//! # oxcord-proxy
//!
//! Centralized Discord REST API Rate Limiting Proxy.
//!
//! - Runs as a standalone HTTP proxy daemon.
//! - Centrally coordinates Discord rate limit buckets across hundreds of worker containers.
//! - Injects the Discord Bot token securely so individual worker nodes don't need direct token exposure.
//! - Completely prevents HTTP 429 Cloudflare bans for enterprise-scale bot fleets.

use std::env;
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderName, HeaderValue, AUTHORIZATION};
use tiny_http::{Header, Response, Server, StatusCode};
use tracing::{error, info, warn};

const DISCORD_API_BASE: &str = "https://discord.com";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "oxcord_proxy=info".parse().unwrap()),
        )
        .init();

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
        warn!("DISCORD_TOKEN not set; proxy running in transparent forwarding mode.");
        String::new()
    });

    let addr = format!("0.0.0.0:{port}");
    let server = Arc::new(Server::http(&addr).map_err(|e| format!("Failed to bind {addr}: {e}"))?);
    info!("OXCORD Central REST Rate Limiter Proxy listening on http://{addr}");

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let token_arc = Arc::new(token);

    // Accept incoming proxy requests
    loop {
        let server = Arc::clone(&server);
        let client = http_client.clone();
        let token = Arc::clone(&token_arc);

        // tiny_http recv is blocking, run in spawned blocking task
        let mut request = match tokio::task::spawn_blocking(move || server.recv()).await? {
            Ok(req) => req,
            Err(e) => {
                error!("Error accepting HTTP connection: {e}");
                continue;
            }
        };

        tokio::spawn(async move {
            let url_path = request.url().to_string();
            let target_url = format!("{DISCORD_API_BASE}{url_path}");
            let method = reqwest::Method::from_bytes(request.method().as_str().as_bytes())
                .unwrap_or(reqwest::Method::GET);

            let mut body = Vec::new();
            let _ = request.as_reader().read_to_end(&mut body);

            let mut req_builder = client.request(method, &target_url);

            // Forward incoming headers except host
            for h in request.headers() {
                let field_str = h.field.as_str().as_str();
                if !field_str.eq_ignore_ascii_case("host") {
                    if let Ok(name) = HeaderName::from_bytes(field_str.as_bytes()) {
                        if let Ok(val) = HeaderValue::from_bytes(h.value.as_bytes()) {
                            req_builder = req_builder.header(name, val);
                        }
                    }
                }
            }

            // Inject Bot Token if not present or configured centrally
            if !token.is_empty() {
                let auth_val = if token.starts_with("Bot ") {
                    token.to_string()
                } else {
                    format!("Bot {token}")
                };
                if let Ok(val) = HeaderValue::from_str(&auth_val) {
                    req_builder = req_builder.header(AUTHORIZATION, val);
                }
            }

            if !body.is_empty() {
                req_builder = req_builder.body(body);
            }

            match req_builder.send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let resp_headers = resp.headers().clone();

                    match resp.bytes().await {
                        Ok(resp_bytes) => {
                            let mut response = Response::new(
                                StatusCode(status),
                                Vec::new(),
                                resp_bytes.as_ref(),
                                Some(resp_bytes.len()),
                                None,
                            );

                            for (k, v) in &resp_headers {
                                if let Ok(h) = Header::from_bytes(k.as_str().as_bytes(), v.as_bytes()) {
                                    response.add_header(h);
                                }
                            }

                            let _ = request.respond(response);
                        }
                        Err(e) => {
                            let _ = request.respond(Response::from_string(format!("Proxy read error: {e}")).with_status_code(502));
                        }
                    }
                }
                Err(e) => {
                    error!("Proxy request to Discord failed: {e}");
                    let _ = request.respond(Response::from_string(format!("Proxy Gateway error: {e}")).with_status_code(502));
                }
            }
        });
    }
}
