<p align="center">
  <img src="assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

<p align="center">
  <a href="https://crates.io/crates/oxcord"><img src="https://img.shields.io/crates/v/oxcord.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord"><img src="https://img.shields.io/docsrs/oxcord?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
  <a href="https://github.com/wumx-labs/oxcord/actions"><img src="https://img.shields.io/badge/tests-passing-brightgreen?logo=github&logoColor=white" alt="CI Tests"></a>
  <a href="https://discord.gg/JnTRems5CT"><img src="https://img.shields.io/badge/join%20us-on%20discord-5865F2?logo=discord&logoColor=white" alt="Discord"></a>
  <br>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rustc-1.85%2B%20%282024%29-orange?logo=rust&logoColor=white" alt="Rust Version"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20ISC-blue?logo=apache&logoColor=white" alt="License"></a>
  <a href="https://github.com/wumx-labs/oxcord/graphs/contributors"><img src="https://img.shields.io/badge/contributors-NoxlyDev-blueviolet?logo=github&logoColor=white" alt="Contributors"></a>
  <a href="crates/"><img src="https://img.shields.io/badge/crates-12%20packages-informational?logo=cargo&logoColor=white" alt="Workspace"></a>
  <br>
  <a href="https://discord.com/developers/docs"><img src="https://img.shields.io/badge/Discord%20API-v10-5865F2?logo=discord&logoColor=white" alt="Discord API"></a>
  <a href="crates/oxcord-voice"><img src="https://img.shields.io/badge/Voice-v8%20AEAD-blueviolet" alt="Voice v8"></a>
  <a href="crates/oxcord-gateway"><img src="https://img.shields.io/badge/Gateway-Zstandard-success" alt="Gateway Zstd"></a>
  <a href="crates/oxcord-broker"><img src="https://img.shields.io/badge/Brokers-Redis%20%7C%20NATS%20%7C%20Kafka-orange" alt="Brokers"></a>
</p>

<p align="center">
  <a href="https://oxcord.noxlydev.xyz"><b>Documentation</b></a> •
  <a href="#-quick-start"><b>Quick Start</b></a> •
  <a href="#-workspace-architecture"><b>Architecture</b></a> •
  <a href="examples/"><b>25 Examples</b></a> •
  <a href="https://discord.gg/JnTRems5CT"><b>Discord Support</b></a>
</p>

**OXCORD** is a modular, high-performance, and forward-compatible Discord library for Rust. Designed from the ground up to scale seamlessly from lightweight single-binary bots to distributed, multi-process architectures serving millions of guilds.

---

## ✨ Key Features

- **📦 Modular Multi-Crate Architecture**: Depend only on what your bot or service needs. Use isolated crates (`oxcord-model`, `oxcord-rest`, `oxcord-gateway`, `oxcord-builders`, `oxcord-cache`) or the unified `oxcord` facade.
- **🎨 Discord Components V2**: First-class support for Discord's next-generation UI system—`Container`, `Section`, `TextDisplay`, `Thumbnail`, `MediaGallery`, `File`, `Separator`, `Label`, and automatic `IS_COMPONENTS_V2` message flags.
- **⚡ High-Performance Gateway Engine**:
  - **Zstandard (`zstd`) & zlib**: Automatic payload decompression detection with fast zero-copy deserialization directly from byte buffers.
  - **Full Jitter Exponential Backoff**: Intelligent PRNG-based backoff to prevent reconnect storms.
  - **Discord Gateway v10 Resume URL**: Proper session resumption via Discord's dynamic resume endpoints.
- **🌐 HTTP Connection Pooling & Keepalive**: Optimized `reqwest` connection pool with configurable TCP keep-alive, idle connection timeouts, and zero redundant TLS handshakes.
- **🛡️ Forward-Compatible Models**: Robust enum structures with `Unknown(u8)` variants ensure your bot never crashes when Discord rolls out new features or opcodes.
- **📊 Zero-Overhead Structured Tracing**: Native `tracing` spans and events for tracking event throughput, heartbeat round-trip latency, and HTTP route timings.

---

## 🏛️ Workspace Architecture

`oxcord` is organized into focused, independent crates within a Cargo workspace:

| Crate | Purpose | Key Responsibilities |
|---|---|---|
| [`oxcord`](crates/oxcord) | **Facade Crate** | Re-exports all sub-crates with convenient feature toggles. Start here for typical bots. |
| [`oxcord-model`](crates/oxcord-model) | **Data Models** | Discord API types, enums with `Unknown` variants, serde logic, snowflake IDs, and event schemas. |
| [`oxcord-rest`](crates/oxcord-rest) | **HTTP & REST** | Discord REST API client, route ratelimiter, connection pooling, and multipart file uploads. |
| [`oxcord-gateway`](crates/oxcord-gateway) | **WebSocket Gateway** | WebSocket stream, sharding, full-jitter reconnect backoff, heartbeat management, and zstd/zlib compression. |
| [`oxcord-voice`](crates/oxcord-voice) | **Voice Client** | Discord Voice Gateway v8, IP Discovery, AEAD ciphers (`aead_xchacha20_poly1305_rtpsize`, `aead_aes256_gcm_rtpsize`). |
| [`oxcord-voice-model`](crates/oxcord-voice-model) | **Voice Models** | Voice gateway opcodes, RTP packet structures, speaking flags, and handshake models. |
| [`oxcord-cache`](crates/oxcord-cache) | **State Cache** | Concurrent DashMap-based in-memory cache with TTL policies and memory sweeper. |
| [`oxcord-broker`](crates/oxcord-broker) | **Distributed Broker** | Pluggable message broker (InProcess, Redis, NATS, Kafka) with cross-shard RPC dispatching. |
| [`oxcord-gateway-daemon`](crates/oxcord-gateway-daemon) | **Gateway Daemon** | Standalone high-throughput gateway daemon that streams raw events into message brokers. |
| [`oxcord-proxy`](crates/oxcord-proxy) | **REST Proxy** | Centralized REST rate-limiting proxy with token injection and zero-copy request forwarding. |
| [`oxcord-builders`](crates/oxcord-builders) | **Type-Safe Builders** | Fluent builders for messages, embeds, modals, polls, and Discord Components V2. |
| [`oxcord-command-attr`](crates/oxcord-command-attr) | **Command Macros** | Procedural macros for standard framework command declarations. |

---

## 🚀 Quick Start

Add `oxcord` to your `Cargo.toml`:

```toml
[dependencies]
oxcord = { version = "0.1.0", features = ["client", "gateway", "rustls_backend", "model"] }
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

### 1. Minimal Ping Bot

```rust
use oxcord::async_trait;
use oxcord::model::channel::Message;
use oxcord::model::gateway::Ready;
use oxcord::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong! 🏓").await {
                eprintln!("Error sending message: {why:?}");
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

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

### 2. Modern Components V2 Card

`oxcord` provides full support for Discord's new **Components V2** interface:

```rust
use oxcord::builder::{
    CreateButton, CreateContainer, CreateMessage, CreateSection, CreateSeparator,
    CreateTextDisplay, CreateThumbnail,
};
use oxcord::model::application::SeparatorSpacing;

// Build a modern card with custom accent color, markdown text, and accessories
let header = CreateTextDisplay::new("## 🚀 Announcement\nWelcome to modern Components V2!");
let separator = CreateSeparator::new().divider(true).spacing(SeparatorSpacing::Small);
let button = CreateButton::new("btn_action").label("Click Here");
let section = CreateSection::new()
    .add_text("Accessory actions are aligned side-by-side:")
    .accessory(button);

let container = CreateContainer::new()
    .accent_color(0x5865F2) // Discord Blurple
    .add_component(header)
    .add_component(separator)
    .add_component(section);

// components_v2() automatically sets the IS_COMPONENTS_V2 flag
let message = CreateMessage::new()
    .content("Here is your Components V2 card:")
    .components_v2(vec![container.into()]);

channel_id.send_message(&ctx.http, message).await?;
```

---

## 🎛️ Feature Flags

| Feature | Description | Default |
|---|---|:---:|
| `client` | High-level `Client` wrapper managing shards, event handlers, and cache. | Yes |
| `gateway` | WebSocket gateway connection and sharding support. | Yes |
| `model` | Discord data structures and serializations. | Yes |
| `cache` | In-memory concurrent state caching. | Yes |
| `builder` | Fluent type-safe builder APIs. | Yes |
| `rustls_backend` | Uses `rustls` for TLS connections (recommended). | Yes |
| `native_tls_backend` | Uses OpenSSL / OS-native TLS backend. | No |
| `zstd` | Enables Zstandard decompression for gateway payloads. | No |
| `simd_json` | Enables SIMD-accelerated JSON parsing via `simd-json`. | No |

---

## 📚 Examples

The repository includes **20 runnable examples** in the [`examples/`](examples) directory:

- [`e01_basic_ping_bot`](examples/e01_basic_ping_bot) — Minimal bot template.
- [`e02_transparent_guild_sharding`](examples/e02_transparent_guild_sharding) — Multi-shard architecture with shared cache.
- [`e14_slash_commands`](examples/e14_slash_commands) — Registering and handling application slash commands.
- [`e17_message_components`](examples/e17_message_components) — Interactive buttons and select menus.
- [`e19_interactions_endpoint`](examples/e19_interactions_endpoint) — Receiving interactions via HTTP webhooks without a Gateway connection.
- [`e20_components_v2`](examples/e20_components_v2) — Discord Components V2 interactive showcase.

To run any example:
```bash
DISCORD_TOKEN="your_token_here" cargo run -p e20_components_v2
```

---

## 🙏 Acknowledgements

OXCORD builds upon foundational architecture, data models, and ergonomics originally established by the [Serenity](https://github.com/serenity-rs/serenity) project (licensed under the ISC License by Serenity Contributors). We express our gratitude to the Serenity community for paving the way for Rust in Discord bots.

OXCORD evolves these concepts into a modern 12-crate modular ecosystem featuring pure async microservices, pluggable message brokers (NATS, Redis, Kafka), modern Voice Gateway v8 with AEAD ciphers, and zero-allocation interaction webhooks.

---

## 📄 License

- Dual-licensed under **Apache License, Version 2.0** ([LICENSE](LICENSE)) and **ISC License** ([NOTICE](NOTICE)).
- For third-party license notices and historical copyright attributions, see [NOTICE](NOTICE).
- **Disclaimer**: `oxcord` is an independent open-source project and is not affiliated with, endorsed by, or sponsored by Discord Inc.
