<p align="center">
  <img src="../../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# oxcord-gateway

<p align="center">
  <a href="https://crates.io/crates/oxcord-gateway"><img src="https://img.shields.io/crates/v/oxcord-gateway.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord-gateway"><img src="https://img.shields.io/docsrs/oxcord-gateway?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
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

**WebSocket Gateway & Sharding** — Production-grade Discord Gateway WebSocket client with Zstandard compression and full-jitter reconnect backoff.

Part of the [**OXCORD**](../../README.md) multi-crate ecosystem.

---

## Features

- **Zstandard (`zstd`) & zlib**: Automatic stream decompression for minimal bandwidth and maximum throughput.
- **Discord v10 Resume URL**: Automatic dynamic session resumption.
- **Full Jitter Exponential Backoff**: Prevents reconnect stampedes during Discord incidents.
- **Shard Manager**: Built-in horizontal sharding across CPU cores or tasks.

## Usage

```toml
[dependencies]
oxcord-gateway = "0.0.1"
```


---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](../../LICENSE)).
