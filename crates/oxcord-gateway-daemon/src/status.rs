//! Lightweight health and metrics HTTP server for the Gateway Daemon.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::session::SessionStore;

/// Shared daemon metrics tracked during runtime.
#[derive(Debug)]
pub struct DaemonMetrics {
    /// Number of events forwarded to the broker.
    pub events_forwarded: AtomicU64,
    /// Daemon start instant.
    pub start_time: Instant,
}

impl Default for DaemonMetrics {
    fn default() -> Self {
        Self {
            events_forwarded: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }
}

/// Start the background status server on the requested port.
pub fn start_status_server(port: u16, metrics: Arc<DaemonMetrics>, session_store: SessionStore) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tokio::spawn(async move {
        let listener = match TcpListener::bind(addr).await {
            Ok(l) => {
                info!(port = port, "Gateway Daemon status/health server listening");
                l
            },
            Err(e) => {
                error!(port = port, error = %e, "Failed to bind status server; status endpoint disabled");
                return;
            },
        };

        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(_) => continue,
            };

            let metrics = Arc::clone(&metrics);
            let sessions = session_store.clone();

            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n > 0 => n,
                    _ => return,
                };

                let req = String::from_utf8_lossy(&buf[..n]);
                let first_line = req.lines().next().unwrap_or("");
                let path = first_line.split_whitespace().nth(1).unwrap_or("/");

                let (status, body) = match path {
                    "/health" | "/live" => (
                        "200 OK",
                        serde_json::json!({
                            "status": "healthy",
                            "uptime_secs": metrics.start_time.elapsed().as_secs()
                        })
                        .to_string(),
                    ),
                    "/metrics" | "/status" => {
                        let active_sessions = sessions.all();
                        (
                            "200 OK",
                            serde_json::json!({
                                "status": "running",
                                "uptime_secs": metrics.start_time.elapsed().as_secs(),
                                "events_forwarded": metrics.events_forwarded.load(Ordering::Relaxed),
                                "active_shards": active_sessions.len(),
                                "sessions": active_sessions,
                            })
                            .to_string(),
                        )
                    },
                    _ => ("404 Not Found", r#"{"error":"Not Found"}"#.to_string()),
                };

                let response = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status,
                    body.len(),
                    body
                );

                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    });
}
