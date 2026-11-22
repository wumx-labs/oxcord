# Migrating from Serenity to OXCORD

OXCORD's API is designed to feel familiar to Serenity users while delivering modern architecture, modularity, and superior performance.

---

## 🔄 Dependency Mapping

Change your `Cargo.toml`:

```toml
# Old:
# serenity = { version = "0.12", features = ["client", "gateway", "rustls_backend", "model"] }

# New:
oxcord = { version = "0.1.0", features = ["client", "gateway", "rustls_backend", "model"] }
```

---

## 🛠️ Code Migrations

### 1. Import Statements
Replace `serenity` with `oxcord`:

```rust
// Old:
// use serenity::prelude::*;
// use serenity::model::prelude::*;
// use serenity::async_trait;

// New:
use oxcord::prelude::*;
use oxcord::model::prelude::*;
use oxcord::async_trait;
```

### 2. Modern Voice Setup
Serenity relies on `songbird` with legacy ciphers. OXCORD provides built-in Voice Gateway v8 with modern AEAD ciphers:

```rust
// Joining a channel with oxcord:
ctx.join_voice_channel(guild_id, channel_id, false, false).await?;

// Leaving a channel:
ctx.leave_voice_channel(guild_id).await?;
```

### 3. State & Cache Engine
If you need distributed caching (e.g. across multiple nodes), Serenity required custom forks. OXCORD supports pluggable backends out of the box:

```rust
use oxcord::cache::{InMemoryBackend, KeyValueStateBackend, StateEngine};

// In-Memory:
let state = StateEngine::new(InMemoryBackend::new());

// Or Redis/Dragonfly:
let state = StateEngine::new(KeyValueStateBackend::with_prefix(my_redis_store, "bot"));
```

### 4. Serverless Webhook Endpoint
In Serenity, receiving interactions over HTTP required extensive boilerplate. In OXCORD:

```rust
use oxcord::interactions_endpoint::{process_interaction, InteractionHandler};

// Handled with zero-alloc Ed25519 verification:
let output = process_interaction(&verifier, &handler, signature, timestamp, body).await?;
```
