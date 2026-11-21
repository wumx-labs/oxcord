<p align="center">
  <img src="../../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# oxcord

<p align="center">
  <a href="https://crates.io/crates/oxcord"><img src="https://img.shields.io/crates/v/oxcord.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord"><img src="https://img.shields.io/docsrs/oxcord?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg?logo=apache&logoColor=white" alt="License"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rustc-1.85%2B%20(2024)-orange.svg?logo=rust&logoColor=white" alt="Rust Version"></a>
  <a href="https://discord.com/developers/docs"><img src="https://img.shields.io/badge/Discord%20API-v10-5865F2.svg?logo=discord&logoColor=white" alt="Discord API"></a>
</p>

<p align="center">
  <a href="../../README.md"><b>Workspace Root</b></a> •
  <a href="https://oxcord.noxlydev.xyz"><b>Documentation</b></a> •
  <a href="../../examples/"><b>Examples</b></a> •
  <a href="https://discord.gg/JnTRems5CT"><b>Discord Community</b></a>
</p>

**Facade Crate** — The unified facade crate for OXCORD — modular, high-performance, and forward-compatible Discord library for Rust.

Part of the [**OXCORD**](../../README.md) multi-crate ecosystem.

---

## Overview

`oxcord` re-exports all modular sub-crates behind convenient Cargo feature flags. For typical bots, this is the only dependency you need.

## Quick Start

```toml
[dependencies]
oxcord = { version = "0.0.1", features = ["client", "gateway", "rustls_backend", "model"] }
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

```rust,no_run
use oxcord::prelude::*;

struct Handler;

#[oxcord::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            let _ = msg.channel_id.say(&ctx.http, "Pong!").await;
        }
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("client creation failed");

    client.start().await.expect("client runtime error");
}
```


---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](../../LICENSE)).
