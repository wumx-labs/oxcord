//! Live smoke test: one unauthenticated REST call over TLS (through `HTTPS_PROXY` when set).
//! Run with `cargo test -p oxcord-rest --test live_rest -- --ignored`. It checks that reqwest
//! finds exactly one rustls provider when only this crate is built.

use oxcord_rest::Http;

#[tokio::test]
#[ignore = "needs internet access"]
async fn gets_the_gateway_url() {
    let http = Http::new("");
    let gateway = http.get_gateway().await.expect("TLS + REST call");
    assert!(gateway.url.starts_with("wss://"));
}
