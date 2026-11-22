//! Core proxy handler — receives inbound HTTP, rate-limits, forwards to Discord.

use std::convert::Infallible;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Request, Response, StatusCode};
use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};
use tracing::{debug, error, warn};

use crate::ratelimit::GlobalLimiter;

/// Shared state threaded into every request handler.
pub struct ProxyState {
    /// Discord Bot token (without `Bot ` prefix). Empty = transparent mode.
    pub token: Arc<String>,
    /// Discord API base URL.
    pub discord_base: &'static str,
    /// Upstream HTTP client.
    pub client: reqwest::Client,
    /// Rate limit coordinator.
    pub limiter: GlobalLimiter,
}

/// Handle a single inbound HTTP request.
pub async fn handle(
    state: Arc<ProxyState>,
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match proxy_request(state, req).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            error!("proxy error: {e}");
            Ok(error_response(StatusCode::BAD_GATEWAY, format!("Proxy error: {e}")))
        },
    }
}

async fn proxy_request(
    state: Arc<ProxyState>,
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let path_and_query = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
    let target_url = format!("{}{path_and_query}", state.discord_base);

    // Determine Discord bucket (if already known) and pre-wait.
    let route_key = extract_route_key(req.uri().path());
    state.limiter.wait_for_bucket(&route_key).await;

    // Acquire a global slot (limits total in-flight requests).
    let _permit = state.limiter.acquire().await;

    // Collect inbound body.
    let (parts, body) = req.into_parts();
    let body_bytes = body.collect().await?.to_bytes();

    // Build the upstream reqwest request.
    let method = reqwest::Method::from_bytes(parts.method.as_str().as_bytes())?;
    let mut builder = state.client.request(method, &target_url);

    // Forward headers, skipping hop-by-hop.
    for (name, value) in &parts.headers {
        let n = name.as_str();
        if matches!(n, "host" | "connection" | "transfer-encoding" | "te" | "trailer" | "upgrade") {
            continue;
        }
        if let (Ok(h), Ok(v)) =
            (HeaderName::from_bytes(n.as_bytes()), HeaderValue::from_bytes(value.as_bytes()))
        {
            builder = builder.header(h, v);
        }
    }

    // Inject Bot token if configured.
    if !state.token.is_empty() {
        let auth = if state.token.starts_with("Bot ") {
            state.token.as_str().to_owned()
        } else {
            format!("Bot {}", state.token)
        };
        if let Ok(val) = HeaderValue::from_str(&auth) {
            builder = builder.header(AUTHORIZATION, val);
        }
    }

    if !body_bytes.is_empty() {
        builder = builder.body(reqwest::Body::from(body_bytes));
    }

    // Send upstream.
    let resp = builder.send().await?;
    let status = resp.status();

    // Extract rate limit headers before consuming the response.
    let bucket =
        resp.headers().get("x-ratelimit-bucket").and_then(|v| v.to_str().ok()).map(str::to_owned);
    let remaining: Option<i64> = resp
        .headers()
        .get("x-ratelimit-remaining")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());
    let reset_after: Option<f64> = resp
        .headers()
        .get("x-ratelimit-reset-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    // Update bucket state.
    state.limiter.update_from_headers(bucket, remaining, reset_after);

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        warn!("Received 429 from Discord — bucket state desynced; backing off");
    }

    // Build the hyper response.
    let mut response_builder = Response::builder().status(status.as_u16());
    for (k, v) in resp.headers() {
        if let (Ok(name), Ok(val)) = (
            hyper::header::HeaderName::from_bytes(k.as_str().as_bytes()),
            hyper::header::HeaderValue::from_bytes(v.as_bytes()),
        ) {
            response_builder = response_builder.header(name, val);
        }
    }

    let resp_bytes = resp.bytes().await?;
    debug!(status = %status.as_u16(), bytes = %resp_bytes.len(), "upstream response");

    Ok(response_builder
        .body(Full::new(resp_bytes))
        .map_err(|e| format!("response build error: {e}"))?)
}

/// Derive a stable bucket key from the URL path for pre-request lookup.
///
/// Discord uses major params (guild ID, channel ID, webhook ID) as part of the
/// bucket hash. We use the path with snowflake IDs normalized to `{id}` as a
/// lookup key into our local bucket map.
fn extract_route_key(path: &str) -> String {
    // Normalize 17-19 digit snowflakes to `{id}`.
    let mut key = String::with_capacity(path.len());
    for segment in path.split('/') {
        if !key.is_empty() {
            key.push('/');
        }
        if segment.len() >= 17 && segment.bytes().all(|b| b.is_ascii_digit()) {
            key.push_str("{id}");
        } else {
            key.push_str(segment);
        }
    }
    key
}

fn error_response(status: StatusCode, body: String) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain")
        .body(Full::new(Bytes::from(body)))
        .expect("static error response is valid")
}
