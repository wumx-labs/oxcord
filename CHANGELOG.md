# Change Log

All notable changes to this project will be documented in this file.
This project mostly adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## oxcord-framework 0.2.2

- fix(framework): `Context::send` setelah `Context::defer` pada slash command sekarang mengedit
  respons asli (state "thinking…") alih-alih membuat respons baru yang ditolak Discord
  (`Interaction has already been acknowledged`).
- feat(framework): `PrefixOptions::noprefix` (`NoPrefixFn`) — callback "boleh tanpa prefix" per
  pengguna, dicoba terakhir setelah semua prefix lain gagal cocok (setara `stripped_dynamic_prefix`
  poise yang mengembalikan string kosong). Catatan: menambah field publik ke `PrefixOptions`;
  pemakai yang membuat struct itu tanpa `..Default::default()` perlu menambahkan field ini.

## oxcord 0.2.1 / oxcord-framework 0.2.1 / oxcord-builders 0.1.2

- fix(framework): registrasi slash global tidak lagi gagal pada aplikasi yang punya Entry Point
  command (Activity) — command bertipe `PrimaryEntryPoint` ikut dibawa saat bulk overwrite.
- feat(framework): `Reply::components_v2` untuk pesan Components V2 (slash, prefix, dan edit).
- feat(builders): `EditInteractionResponse::components_v2` dan `EditWebhookMessage::components_v2`.
- chore(framework): `serde_json` jadi dependency biasa (sebelumnya dev-dependency).

- Initial restructuring: workspace layout, edition 2024, MSRV 1.85.
- Model convenience methods moved into the `ops` extension-trait layer.
