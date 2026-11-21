# Distributed Message Broker (`oxcord-broker`)

The `oxcord-broker` crate provides a unified, backend-agnostic message broker and RPC abstraction designed for enterprise-scale Discord bots (T3/T4 scale — tens of thousands to millions of servers).

Inspired by `@discordjs/brokers`, it allows you to decouple your bot into isolated microservices without vendor lock-in.

---

## Architecture Overview

```
                      ┌───────────────────────────────────────┐
                      │    Gateway Cluster (Daemon Shards)    │
                      │  - Shard 0..99                        │
                      │  - Maintains WebSocket to Discord     │
                      └──────────────────┬────────────────────┘
                                         │ BrokerMessage
                                         ▼
                      ┌───────────────────────────────────────┐
                      │      Message Broker (Pluggable)       │
                      │   [In-Process] / [NATS] / [Redis]     │
                      └──────────────────┬────────────────────┘
                                         │
                   ┌─────────────────────┴─────────────────────┐
                   ▼                                           ▼
┌──────────────────────────────────────┐    ┌──────────────────────────────────────┐
│        Worker Node 1 (Commands)      │    │        Worker Node 2 (Cache)         │
│  - Executes slash & text commands    │    │  - Ingests events into StateEngine   │
│  - Calls Gateway via RPC for actions │    │  - Answers state queries via RPC     │
└──────────────────────────────────────┘    └──────────────────────────────────────┘
```

---

## Key Benefits

1. **Zero-Downtime Bot Updates**: When you redeploy or update your command handling code, the Gateway WebSocket connection stays alive in the Sharder. Discord never detects a disconnect.
2. **Horizontal Scalability**: Add as many worker instances as needed to handle command spikes.
3. **Cross-Shard RPC**: Workers can query shard latency, fetch guild counts, or trigger actions on shards managed by other processes.
4. **Zero Vendor Lock-In**: Choose between in-process testing, Redis Pub/Sub, or NATS via simple feature flags.

---

## Supported Broker Drivers

| Driver | Feature Flag | Description |
|---|---|---|
| `InProcessBroker` | *(Default)* | Concurrent broadcast channels via Tokio. 0 external dependencies. Perfect for testing. |
| `RedisBroker` | `redis` | Redis Pub/Sub backend for multi-container deployments. |
| `NatsBroker` | `nats` | Ultra-high-throughput, sub-millisecond cloud microservices. Pure async Rust. |
| `KafkaBroker` | `kafka` | Apache Kafka driver backed by `rskafka` (100% pure Rust async client, 0 C-dependencies). |

---

## Basic Usage

### 1. Publishing Gateway Events (Publisher)

```rust
use oxcord::broker::{Broker, BrokerMessage, InProcessBroker};

let broker = InProcessBroker::new();

let msg = BrokerMessage::from_json("discord.events.message_create", &event)?
    .with_shard(0)
    .with_event_name("MESSAGE_CREATE");

broker.publish_message(&msg).await?;
```

### 2. Subscribing to Events in a Worker (Subscriber)

```rust
use oxcord::broker::{Broker, InProcessBroker};

let broker = InProcessBroker::new();
let mut sub = broker.subscribe("discord.events.message_create").await?;

while let Some(msg) = sub.next_message().await {
    println!("Received event from Shard {:?}", msg.shard_id());
}
```

### 3. Cross-Shard RPC (Request-Response)

#### Registering an RPC Responder:
```rust
use oxcord::broker::{RpcDispatcher, InProcessBroker};

let dispatcher = RpcDispatcher::new(InProcessBroker::new());

dispatcher.register("rpc.shard.0.latency", |_req| async move {
    Ok(b"42ms".to_vec())
}).await;

dispatcher.listen("rpc.shard.0.latency").await?;
```

#### Calling the RPC from another Node:
```rust
use std::time::Duration;

let latency_bytes = broker.request("rpc.shard.0.latency", b"ping", Duration::from_secs(2)).await?;
println!("Shard 0 Latency: {}", String::from_utf8_lossy(&latency_bytes));
```

---

## See Also

- [Example: e23_broker_distributed_bot](https://github.com/wumx-labs/oxcord/tree/main/examples/e23_broker_distributed_bot)
- [Pluggable State & Cache](/guide/cache)
- [Modular Architecture](/guide/architecture)
