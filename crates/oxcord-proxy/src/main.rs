//! # oxcord-proxy
//!
//! Centralized Discord REST API Rate Limiting Proxy — built on **hyper 1.0**.
//!
//! ## Why
//!
//! When running hundreds of shards across multiple worker processes (T3/T4), each
//! process makes independent Discord REST calls and each has its own view of the
//! rate limit windows. Without coordination, concurrent 429s pile up. This proxy
//! acts as the single chokepoint:
//!
//! - All worker containers send REST requests **here** instead of directly to Discord.
//! - Rate limit buckets are tracked in-process (`DashMap`) with per-route token
//!   bucket scheduling.
//! - The Discord Bot token is injected here, so individual shard containers never
//!   hold the token.
//! - On overload the proxy queues requests and drains in bucket order, preventing
//!   Cloudflare 429 bans at enterprise scale.
//!
//! ## Configuration
//!
//! | Env var | Default | Description |
//! |---|---|---|
//! | `PORT` | `8080` | Local listen port |
//! | `DISCORD_TOKEN` | *(required)* | Bot token (without `Bot ` prefix) |
//! | `GLOBAL_RATE_LIMIT` | `50` | Requests-per-second global cap |
//!
//! ## Running
//!
//! ```bash
//! DISCORD_TOKEN=your_token cargo run -p oxcord-proxy
//! ```

mod proxy;
mod ratelimit;

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use proxy::ProxyState;
use ratelimit::GlobalLimiter;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info, warn};

const DISCORD_API_BASE: &str = "https://discord.com";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "oxcord_proxy=info".parse().expect("valid filter")),
        )
        .init();

    let port: u16 = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);

    let token = match env::var("DISCORD_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => {
            warn!(
                "DISCORD_TOKEN not set; running in transparent forwarding mode (no auth injection)"
            );
            String::new()
        },
    };

    let global_rps: u32 =
        env::var("GLOBAL_RATE_LIMIT").ok().and_then(|v| v.parse().ok()).unwrap_or(50);

    let http_client = reqwest::Client::builder().timeout(Duration::from_secs(30)).build()?;

    let state = Arc::new(ProxyState {
        token: Arc::new(token),
        discord_base: DISCORD_API_BASE,
        client: http_client,
        limiter: GlobalLimiter::new(global_rps),
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!("OXCORD Central REST Rate Limiter Proxy listening on http://{addr}");
    info!("Global rate limit cap: {global_rps} req/s");

    loop {
        tokio::select! {
            conn = listener.accept() => {
                match conn {
                    Ok((stream, remote)) => {
                        let io = TokioIo::new(stream);
                        let state = Arc::clone(&state);

                        tokio::spawn(async move {
                            let svc = service_fn(move |req| {
                                let state = Arc::clone(&state);
                                proxy::handle(state, req)
                            });

                            if let Err(e) = http1::Builder::new()
                                .keep_alive(true)
                                .serve_connection(io, svc)
                                .await
                            {
                                error!(remote = %remote, "connection error: {e}");
                            }
                        });
                    },
                    Err(e) => error!("accept error: {e}"),
                }
            }
            _ = shutdown_signal() => {
                info!("Shutdown signal received — stopping proxy");
                break;
            }
        }
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    {
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler")
                .recv()
                .await;
        };
        tokio::select! {
            () = ctrl_c => {},
            () = terminate => {},
        }
    }

    #[cfg(not(unix))]
    ctrl_c.await;
}
