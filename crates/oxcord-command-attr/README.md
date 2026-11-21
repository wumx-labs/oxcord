<p align="center">
  <img src="../../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# oxcord-command-attr

<p align="center">
  <a href="https://crates.io/crates/oxcord-command-attr"><img src="https://img.shields.io/crates/v/oxcord-command-attr.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord-command-attr"><img src="https://img.shields.io/docsrs/oxcord-command-attr?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
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

**Command Framework Macros** — Procedural attribute macros for declaring standard bot commands, hooks, and permission checks.

Part of the [**OXCORD**](../../README.md) multi-crate ecosystem.

---

## Features

- `#[command]` macro: Annotate functions as bot commands with aliases, descriptions, and rate limits.
- `#[group]` macro: Group related commands with shared prefixes and checks.
- Compile-time argument parsing and signature verification.

## Usage

```toml
[dependencies]
oxcord-command-attr = "0.5.4"
```


---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](../../LICENSE)).
