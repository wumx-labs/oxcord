<p align="center">
  <img src="../../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# oxcord-broker

<p align="center">
  <a href="https://crates.io/crates/oxcord-broker"><img src="https://img.shields.io/crates/v/oxcord-broker.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord-broker"><img src="https://img.shields.io/docsrs/oxcord-broker?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
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

**Distributed Message Broker & RPC** — Pluggable message broker abstraction supporting InProcess, Redis, NATS, and Kafka for million-guild scale.

Part of the [**OXCORD**](../../README.md) multi-crate ecosystem.

---

## Features

- **`Broker` trait**: Unified async publish/subscribe and RPC (request-reply) interface.
- **`InProcessBroker`**: Zero-dependency broadcast broker using Tokio channels for single-process architectures.
- **`RedisBroker`** (feature `redis`): Redis Pub/Sub backend for containerized deployments.
- **`NatsBroker`** (feature `nats`): Ultra-low-latency pure Rust NATS backend.
- **`KafkaBroker`** (feature `kafka`): 100% pure Rust Kafka backend (`rskafka`) for high-throughput enterprise event streaming.
- **`RpcDispatcher`**: Cross-shard request-response dispatching across microservices.

## Usage

```toml
[dependencies]
oxcord-broker = { version = "0.1.0", features = ["nats"] } # or "redis", "kafka"
```


---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](../../LICENSE)).
