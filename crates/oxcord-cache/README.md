<p align="center">
  <img src="../../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# oxcord-cache

<p align="center">
  <a href="https://crates.io/crates/oxcord-cache"><img src="https://img.shields.io/crates/v/oxcord-cache.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord-cache"><img src="https://img.shields.io/docsrs/oxcord-cache?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
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

**State Engine & Cache Sweeper** — Concurrent in-memory cache powered by DashMap with TTL expiration sweeper and memory safeguards.

Part of the [**OXCORD**](../../README.md) multi-crate ecosystem.

---

## Features

- **Concurrent DashMap**: Lock-free parallel reads for guilds, channels, roles, and members.
- **Cache Sweeper**: Background sweeper task with configurable TTL to prevent memory bloat.
- **Pluggable State Engine**: Trait-based abstraction allowing in-memory or Redis key-value storage.

## Usage

```toml
[dependencies]
oxcord-cache = "0.0.1"
```


---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](../../LICENSE)).
