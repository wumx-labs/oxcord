# Pluggable State & Cache Engine

OXCORD decouples state caching into a modular, pluggable engine: a zero-lock-contention in-memory DashMap backend for single-node bots, and the `KeyValueStateBackend` trait for distributed stores (e.g., Redis, Dragonfly, or RocksDB). For distributed pub/sub messaging and cross-shard event routing, native Redis, NATS, and Kafka drivers are provided by [`oxcord-broker`](/guide/broker).

---

## 🏗️ StateBackend Trait

The [`StateBackend`] trait abstracts all state operations:

```rust
use oxcord::async_trait;
use oxcord::cache::StateBackend;
use oxcord::model::prelude::*;

#[async_trait]
pub trait StateBackend: Send + Sync {
    async fn get_guild(&self, guild_id: GuildId) -> Option<Guild>;
    async fn set_guild(&self, guild: Guild);
    async fn remove_guild(&self, guild_id: GuildId) -> Option<Guild>;
    
    async fn get_channel(&self, channel_id: ChannelId) -> Option<GuildChannel>;
    async fn set_channel(&self, channel: GuildChannel);
    
    async fn get_user(&self, user_id: UserId) -> Option<User>;
    async fn set_user(&self, user: User);
    
    async fn get_member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member>;
    async fn set_member(&self, guild_id: GuildId, member: Member);
    
    async fn clear(&self);
}
```

---

## ⚡ In-Memory Backend (DashMap)

For standalone bots, [`InMemoryBackend`] provides lock-sharded concurrent hash maps with sub-millisecond latencies:

```rust
use oxcord::cache::{InMemoryBackend, StateEngine};

let backend = InMemoryBackend::new();
let state = StateEngine::new(backend);

// Automatically update state from events:
state.handle_event(&event).await;

// Query cached state:
let guild = state.get_guild(guild_id).await;
let is_member = state.is_guild_member(guild_id, user_id).await;
```

---

## 🌐 Distributed Key-Value Backend (Redis / Dragonfly)

For bots scaled across multiple processes or Kubernetes clusters, [`KeyValueStateBackend`] maps state into any key-value store:

```rust
use oxcord::cache::{KeyValueStateBackend, StateEngine};

// Any store implementing KeyValueStore (e.g. Redis, KeyDB, RocksDB)
let kv_store = MyRedisClient::new("redis://127.0.0.1:6379");
let backend = KeyValueStateBackend::with_prefix(kv_store, "mybot");
let state = StateEngine::new(backend);
```

### Key Schema:
- `mybot:guild:{guild_id}`
- `mybot:channel:{channel_id}`
- `mybot:user:{user_id}`
- `mybot:member:{guild_id}:{user_id}`
- `mybot:role:{guild_id}:{role_id}`
- `mybot:msg:{channel_id}:{message_id}`
- `mybot:current_user`
