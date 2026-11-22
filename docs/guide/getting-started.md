# Getting Started

Build your first high-performance Discord bot with **OXCORD** in less than 5 minutes!

---

## 📦 Installation

Add `oxcord` and `tokio` to your `Cargo.toml`:

```toml
[dependencies]
oxcord = { version = "0.1.0", features = ["client", "gateway", "model", "rustls_backend"] }
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

> **Tip**: OXCORD uses `rustls` by default, meaning no system OpenSSL libraries are required!

---

## 🤖 Writing Your First Bot (`ping` / `pong`)

Create `src/main.rs`:

```rust
use oxcord::async_trait;
use oxcord::model::channel::Message;
use oxcord::model::gateway::Ready;
use oxcord::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Fired when the bot connects and successfully logs into Discord
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected and online!", ready.user.name);
    }

    // Fired whenever a message is created in a guild channel or DM
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong! 🏓").await {
                eprintln!("Error sending message: {why:?}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Retrieve bot token from environment variable
    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    // Configure gateway intents (MessageContent is required to read !ping)
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Build and start the client
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}
```

---

## 🚀 Running Your Bot

Set your Discord bot token and run:

```bash
export DISCORD_TOKEN="your_bot_token_here"
cargo run --release
```

Type `!ping` in your server and your bot will immediately respond with `Pong! 🏓`!

---

## 🛠️ Feature Flags

OXCORD provides fine-grained feature flags so you only compile what your bot uses:

| Feature Flag | Description |
|---|---|
| `client` | High-level `Client` wrapper and connection manager |
| `gateway` | WebSocket Gateway runner and shard manager |
| `model` | Discord data types and serde models |
| `cache` | In-memory concurrent state caching |
| `voice` | Modern Discord Voice v8 audio driver and RTP pipeline |
| `interactions_endpoint` | Zero-alloc Ed25519 Webhook HTTP server |
| `zstd` | Ultra-fast Zstandard gateway decompression |
| `rustls_backend` | Pure Rust TLS backend (enabled by default) |
| `full` | Enables all major stable features |
