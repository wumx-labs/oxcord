//! Live smoke test: opens the real Discord gateway (through `HTTPS_PROXY` when set) and expects
//! the first event. Run with `cargo test -p oxcord-gateway --test live_connect -- --ignored`.
//! It checks the TLS stack (one rustls provider) and the websocket upgrade end to end.

use oxcord_gateway::WsClient;
use url::Url;

#[tokio::test]
#[ignore = "needs internet access"]
async fn connects_to_the_gateway_and_receives_hello() {
    let url = Url::parse("wss://gateway.discord.gg/?v=10&encoding=json").unwrap();
    let mut ws = WsClient::connect(url).await.expect("TLS + websocket handshake");
    let event = ws.recv_json().await.expect("read the first event");
    assert!(event.is_some(), "Discord sends Hello (opcode 10) first");
}
