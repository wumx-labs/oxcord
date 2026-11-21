<p align="center">
  <img src="../../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# oxcord-model

<p align="center">
  <a href="https://crates.io/crates/oxcord-model"><img src="https://img.shields.io/crates/v/oxcord-model.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord-model"><img src="https://img.shields.io/docsrs/oxcord-model?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
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

**Data Models & Discord API Types** — Core Discord API types, robust serde models, and forward-compatible enums with Unknown fallback variants.

Part of the [**OXCORD**](../../README.md) multi-crate ecosystem.

---

## Features

- **Forward-Compatible Enums**: Never crash on new Discord gateway opcodes or event types.
- **Strong Typing**: Type-safe snowflake IDs (`UserId`, `GuildId`, `ChannelId`, `MessageId`).
- **Discord Components V2**: Full data structures for Containers, Sections, Labels, MediaGalleries, and Separators.
- **Serde Optimized**: Zero-allocation parsing where possible with fast deserialization.

## Usage

```toml
[dependencies]
oxcord-model = "0.0.1"
```


---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](../../LICENSE)).
