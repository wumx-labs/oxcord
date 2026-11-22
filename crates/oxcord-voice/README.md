<p align="center">
  <img src="../../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# oxcord-voice

<p align="center">
  <a href="https://crates.io/crates/oxcord-voice"><img src="https://img.shields.io/crates/v/oxcord-voice.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord-voice"><img src="https://img.shields.io/docsrs/oxcord-voice?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
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

**Discord Voice Gateway v8 & Encryption** — Voice client with modern AEAD ciphers (AES-256-GCM, XChaCha20-Poly1305), IP discovery, and modular audio pipeline.

Part of the [**OXCORD**](../../README.md) multi-crate ecosystem.

---

## Features

- **Modern AEAD Encryption**: Supports `aead_xchacha20_poly1305_rtpsize` and `aead_aes256_gcm_rtpsize`.
- **IP Discovery**: Built-in UDP hole punching and endpoint resolution.
- **Voice Gateway v8**: Full voice handshake, heartbeat, and speaking status events.
- **DAVE Protocol Status**: In active development / roadmap (Discord's MLS-based end-to-end encryption specification; not yet available for production audio streams).

## Usage

```toml
[dependencies]
oxcord-voice = "0.0.1"
```


---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](../../LICENSE)).
