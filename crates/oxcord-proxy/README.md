<p align="center">
  <img src="../../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# oxcord-proxy

<p align="center">
  <a href="https://crates.io/crates/oxcord-proxy"><img src="https://img.shields.io/crates/v/oxcord-proxy.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord-proxy"><img src="https://img.shields.io/docsrs/oxcord-proxy?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
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

**Centralized REST Proxy** — High-performance centralized HTTP proxy for Discord REST API with rate limiting and bot token injection.

Part of the [**OXCORD**](../../README.md) multi-crate ecosystem.

---

## Features

- **Centralized Rate Limiting**: All microservices route through one proxy, completely eliminating 429 rate limit storms.
- **Token Injection**: Workers don't need raw bot tokens; the proxy injects authentication headers safely.
- **Zero-Copy Forwarding**: Minimal latency overhead on REST requests.

## Usage

```bash
cargo run -p oxcord-proxy -- --bind 127.0.0.1:8080 --token "$DISCORD_TOKEN"
```


---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](../../LICENSE)).
