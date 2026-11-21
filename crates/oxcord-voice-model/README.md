<p align="center">
  <img src="../../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# oxcord-voice-model

<p align="center">
  <a href="https://crates.io/crates/oxcord-voice-model"><img src="https://img.shields.io/crates/v/oxcord-voice-model.svg?logo=rust&logoColor=white&color=brightgreen" alt="Crates.io"></a>
  <a href="https://docs.rs/oxcord-voice-model"><img src="https://img.shields.io/docsrs/oxcord-voice-model?logo=docsdotrs&logoColor=white&color=blue" alt="docs.rs"></a>
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

**Discord Voice Protocol Models** — Protocol data structures, RTP packet layouts, voice gateway opcodes, and speaking flags.

Part of the [**OXCORD**](../../README.md) multi-crate ecosystem.

---

## Features

- Voice Gateway v8 opcode schemas (`Identify`, `SelectProtocol`, `Ready`, `SessionDescription`).
- Speaking state bitflags (`PRIORITY`, `SOUNDSHARE`).
- RTP packet headers and payload type constants.

## Usage

```toml
[dependencies]
oxcord-voice-model = "0.2.1"
```


---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](../../LICENSE)).
