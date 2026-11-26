# Change Log

All notable changes to this project will be documented in this file.
This project mostly adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## oxcord 0.2.1 / oxcord-framework 0.2.1 / oxcord-builders 0.1.2

- fix(framework): registrasi slash global tidak lagi gagal pada aplikasi yang punya Entry Point
  command (Activity) — command bertipe `PrimaryEntryPoint` ikut dibawa saat bulk overwrite.
- feat(framework): `Reply::components_v2` untuk pesan Components V2 (slash, prefix, dan edit).
- feat(builders): `EditInteractionResponse::components_v2` dan `EditWebhookMessage::components_v2`.
- chore(framework): `serde_json` jadi dependency biasa (sebelumnya dev-dependency).

- Initial restructuring: workspace layout, edition 2024, MSRV 1.85.
- Model convenience methods moved into the `ops` extension-trait layer.
