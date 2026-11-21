<p align="center">
  <img src="../../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# oxcord-gateway-daemon

<p align="center">
  <a href="https://crates.io/crates/oxcord-gateway-daemon"><img src="https://img.shields.io/crates/v/oxcord-gateway-daemon.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord-gateway-daemon"><img src="https://img.shields.io/docsrs/oxcord-gateway-daemon?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
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

**Standalone Gateway Daemon** — Independent standalone binary service connecting to Discord Gateway and streaming raw events to message brokers.

Part of the [**OXCORD**](../../README.md) multi-crate ecosystem.

---

## Features

- **Zero-Downtime Bot Deploys**: Deploy bot workers and command handlers without dropping Discord Gateway connections.
- **Pluggable Broker Integration**: Streams raw gateway events directly into NATS, Redis, Kafka, or Tokio channels.
- **Graceful SIGINT Shutdown**: Proper gateway disconnect and session preservation.

## Usage

```bash
cargo run -p oxcord-gateway-daemon -- --token "$DISCORD_TOKEN"
```


---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](../../LICENSE)).
