//! HTTP `CONNECT` tunnel for the gateway websocket.
//!
//! REST goes through `reqwest`, which honours `HTTPS_PROXY`; the websocket used to connect directly,
//! so behind a proxy-only network the REST calls worked while every shard failed with
//! "failed to lookup address information". This module reads the usual environment variables and
//! opens the TCP stream through the proxy; TLS and the websocket handshake then run on top of it.

use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Where to send the `CONNECT` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proxy {
    pub host: String,
    pub port: u16,
    /// `user:password` of a `http://user:password@host:port` URL, for `Proxy-Authorization: Basic`.
    pub credentials: Option<String>,
}

fn env_value(get: &impl Fn(&str) -> Option<String>, upper: &str) -> Option<String> {
    get(upper)
        .or_else(|| get(&upper.to_lowercase()))
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

/// `NO_PROXY`: `*`, exact hosts, and domain suffixes (`example.com` or `.example.com`).
fn bypassed(no_proxy: &str, host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    no_proxy.split(',').map(|e| e.trim().to_ascii_lowercase()).filter(|e| !e.is_empty()).any(|e| {
        if e == "*" {
            return true;
        }
        let e = e.trim_start_matches('.');
        host == e || host.ends_with(&format!(".{e}"))
    })
}

fn parse_proxy_url(raw: &str) -> Option<Proxy> {
    let rest = raw.split_once("://").map_or(raw, |(_, r)| r);
    let rest = rest.split('/').next()?;
    let (credentials, hostport) = match rest.rsplit_once('@') {
        Some((c, h)) => (Some(c.to_owned()), h),
        None => (None, rest),
    };
    let (host, port) = match hostport.rsplit_once(':') {
        Some((h, p)) => (h, p.parse().ok()?),
        None => (hostport, 80),
    };
    (!host.is_empty()).then(|| Proxy {
        host: host.trim_matches(['[', ']']).to_owned(),
        port,
        credentials,
    })
}

/// Proxy to use for `target_host`, from `HTTPS_PROXY`/`https_proxy` and `NO_PROXY`/`no_proxy`.
/// `get` is injected so the rules can be tested without touching the process environment.
pub fn proxy_for(get: impl Fn(&str) -> Option<String>, target_host: &str) -> Option<Proxy> {
    let raw = env_value(&get, "HTTPS_PROXY")?;
    if env_value(&get, "NO_PROXY").is_some_and(|n| bypassed(&n, target_host)) {
        return None;
    }
    parse_proxy_url(&raw)
}

/// Basic auth value (base64 of `user:password`).
fn basic(credentials: &str) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let b = credentials.as_bytes();
    let mut out = String::new();
    for c in b.chunks(3) {
        let n = (u32::from(c[0]) << 16)
            | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
            | u32::from(*c.get(2).unwrap_or(&0));
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

/// Open a TCP stream to `host:port` through `proxy` (`CONNECT`).
pub async fn connect_tunnel(proxy: &Proxy, host: &str, port: u16) -> io::Result<TcpStream> {
    let mut stream = TcpStream::connect((proxy.host.as_str(), proxy.port)).await?;
    let mut request = format!("CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\n");
    if let Some(c) = &proxy.credentials {
        request.push_str(&format!("Proxy-Authorization: Basic {}\r\n", basic(c)));
    }
    request.push_str("\r\n");
    stream.write_all(request.as_bytes()).await?;

    // Read until the blank line that ends the response head (never past it: the next bytes belong
    // to the TLS handshake).
    let mut head = Vec::with_capacity(256);
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        if head.len() > 8192 || stream.read(&mut byte).await? == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "proxy closed the connection",
            ));
        }
        head.push(byte[0]);
    }
    let status = std::str::from_utf8(&head)
        .ok()
        .and_then(|h| h.lines().next())
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    if status == 200 {
        Ok(stream)
    } else {
        Err(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            format!("proxy refused CONNECT with status {status}"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use tokio::net::TcpListener;

    use super::*;

    fn env(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |k| pairs.iter().find(|(a, _)| *a == k).map(|(_, v)| (*v).to_owned())
    }

    #[test]
    fn proxy_url_parsing() {
        let p = parse_proxy_url("http://127.0.0.1:38185").unwrap();
        assert_eq!((p.host.as_str(), p.port, p.credentials), ("127.0.0.1", 38185, None));
        let p = parse_proxy_url("http://user:p%40ss@proxy.local:3128/").unwrap();
        assert_eq!((p.host.as_str(), p.port), ("proxy.local", 3128));
        assert_eq!(p.credentials.as_deref(), Some("user:p%40ss"));
        assert_eq!(parse_proxy_url("proxy:8080").unwrap().port, 8080);
        assert_eq!(parse_proxy_url("http://proxy").unwrap().port, 80);
        assert!(parse_proxy_url("http://:80").is_none());
    }

    #[test]
    fn environment_rules() {
        let host = "gateway.discord.gg";
        assert!(proxy_for(env(&[]), host).is_none());
        assert!(proxy_for(env(&[("HTTPS_PROXY", "http://p:1")]), host).is_some());
        assert!(proxy_for(env(&[("https_proxy", "http://p:1")]), host).is_some());
        assert!(
            proxy_for(
                env(&[("HTTPS_PROXY", "http://p:1"), ("NO_PROXY", "localhost,.discord.gg")]),
                host
            )
            .is_none()
        );
        assert!(
            proxy_for(env(&[("HTTPS_PROXY", "http://p:1"), ("no_proxy", "*")]), host).is_none()
        );
        assert!(
            proxy_for(env(&[("HTTPS_PROXY", "http://p:1"), ("NO_PROXY", "other.com")]), host)
                .is_some()
        );
        assert!(proxy_for(env(&[("HTTPS_PROXY", "  ")]), host).is_none());
    }

    #[test]
    fn basic_auth_matches_rfc_examples() {
        assert_eq!(basic("Aladdin:open sesame"), "QWxhZGRpbjpvcGVuIHNlc2FtZQ==");
        assert_eq!(basic("a"), "YQ==");
        assert_eq!(basic("ab"), "YWI=");
        assert_eq!(basic("abc"), "YWJj");
    }

    async fn fake_proxy(reply: &'static str) -> (u16, tokio::task::JoinHandle<String>) {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = l.local_addr().unwrap().port();
        let h = tokio::spawn(async move {
            let (mut s, _) = l.accept().await.unwrap();
            let mut buf = vec![0u8; 1024];
            let n = s.read(&mut buf).await.unwrap();
            s.write_all(reply.as_bytes()).await.unwrap();
            String::from_utf8_lossy(&buf[..n]).into_owned()
        });
        (port, h)
    }

    #[tokio::test]
    async fn connect_sends_the_request_and_accepts_200() {
        let (port, seen) = fake_proxy("HTTP/1.1 200 Connection established\r\n\r\n").await;
        let proxy = Proxy {
            host: "127.0.0.1".into(),
            port,
            credentials: Some("u:p".into()),
        };
        connect_tunnel(&proxy, "gateway.discord.gg", 443).await.unwrap();
        let request = seen.await.unwrap();
        assert!(request.starts_with("CONNECT gateway.discord.gg:443 HTTP/1.1\r\n"), "{request}");
        assert!(request.contains("Proxy-Authorization: Basic dTpw\r\n"));
    }

    #[tokio::test]
    async fn connect_reports_a_refusal() {
        let (port, _) = fake_proxy("HTTP/1.1 403 Forbidden\r\n\r\n").await;
        let proxy = Proxy {
            host: "127.0.0.1".into(),
            port,
            credentials: None,
        };
        let e = connect_tunnel(&proxy, "x", 443).await.unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::ConnectionRefused);
        assert!(e.to_string().contains("403"));
    }
}
