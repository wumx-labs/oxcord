# Distributed Cache Bot (e22)

This example demonstrates OXCORD's pluggable **`StateEngine`** with an `InMemoryBackend` — the backbone for building bots that query Discord state without hitting the REST API on every event.

> [!NOTE]
> Source code: [`examples/e22_distributed_cache_bot/`](https://github.com/wumx-labs/oxcord/tree/main/examples/e22_distributed_cache_bot)

## Overview

```
Discord Gateway
      │
      ▼ Events (GuildCreate, MessageCreate, …)
┌─────────────────┐
│  EventHandler   │
│  (raw handler)  │
└────────┬────────┘
         │ handle_event(&event)
         ▼
┌─────────────────┐
│  StateEngine    │◄── Any backend: InMemory, Redis, SQL
│                 │
│  get_guild()    │
│  get_member()   │
│  get_channel()  │
│  stats()        │
└─────────────────┘
```

## Commands

| Command | Description |
|---------|-------------|
| `~stats` | Print live cache statistics |
| `~guild` | Show current guild info from cache |
| `~member <id>` | Look up a guild member by user ID |
| `~channel <id>` | Look up a channel by ID |

## Running

```sh
DISCORD_TOKEN=your_token cargo run --example e22_distributed_cache_bot
```

## Key Code Walkthrough

### 1. Create the StateEngine

```rust
use oxcord_cache::{InMemoryBackend, StateEngine};

let state = StateEngine::new(InMemoryBackend::new());
```

The `InMemoryBackend` uses `DashMap` internally — zero-lock-contention concurrent reads.

### 2. Share State via TypeMap

```rust
struct BotStateKey;
impl TypeMapKey for BotStateKey {
    type Value = Arc<RwLock<BotState>>;
}

// Register on client start:
let mut data = client.data.write().await;
data.insert::<BotStateKey>(Arc::new(RwLock::new(BotState::new())));
```

### 3. Seed Cache on Ready

```rust
async fn ready(&self, ctx: Context, ready: Ready) {
    let data = ctx.data.read().await;
    if let Some(bot_state) = data.get::<BotStateKey>() {
        let state = bot_state.read().await;
        state.state.backend().set_current_user(ready.user.clone()).await;
    }
}
```

### 4. Query Without REST

```rust
// In a message handler:
let guild = state.state.get_guild(guild_id).await;
let member = state.state.get_member(guild_id, user_id).await;
let stats  = state.state.stats().await;
```

## Switching to a Distributed Backend

Replace `InMemoryBackend` with `KeyValueStateBackend` to use any K/V store (Redis, Dragonfly, etc.):

```rust
use oxcord_cache::{KeyValueStateBackend, MemoryKeyValueStore};

// Drop-in replacement — same StateEngine API:
let kv = MemoryKeyValueStore::default();
let state = StateEngine::new(KeyValueStateBackend::new(kv, "oxcord"));
```

For Redis, implement the `KeyValueStore` trait on your Redis client.

## See Also

- [Pluggable State & Cache Guide](/guide/cache)
- [Gateway & Compression](/guide/gateway)
