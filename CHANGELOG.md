# Change Log

All notable changes to this project will be documented in this file.
This project mostly adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

- feat(client): `guild_emojis_update_with_old` and `guild_stickers_update_with_old`
  (`FullEvent::GuildEmojisUpdateWithOld` / `GuildStickersUpdateWithOld`) are dispatched together with the
  existing update events and add `old_if_available`, the previous map from the cache (`None` without the
  `cache` feature or when the guild is not cached). Compare it with `current_state` to tell created,
  deleted and changed items apart. The existing handlers are unchanged.

## oxcord 0.3.0 / oxcord-framework 0.3.0 / oxcord-builders 0.2.0 / oxcord-rest 0.2.0 / oxcord-gateway 0.2.0 / oxcord-voice 0.2.0 / oxcord-proxy 0.2.0 / oxcord-gateway-daemon 0.2.0

TLS stack upgrade (breaking). The old stack (reqwest 0.11, hyper 0.14, h2 0.3, rustls 0.21 and 0.22,
tokio-tungstenite 0.21) carried nine open security advisories (RUSTSEC-2026-0049, -0098, -0099,
-0104, -0258, -0346, -0349, -0381, -0382); none of them applies to the new versions.

- **breaking:** `reqwest` 0.11 to 0.12 in `oxcord-rest`, `oxcord-proxy` and `oxcord`. `reqwest` types
  that appear in the public API (`StatusCode`, `Method`, `HeaderMap`, the error type) now come from
  `http` 1.x.
- **breaking:** `tokio-tungstenite` 0.21 to 0.26 in `oxcord-gateway`, `oxcord-voice` and `oxcord`.
  `CloseFrame` has no lifetime parameter any more (`GatewayError::Closed(Option<CloseFrame>)`), and
  `Message::Text` / the close reason hold `Utf8Bytes` instead of `String`.
- feat(gateway): the websocket connects through `HTTPS_PROXY` / `https_proxy` (HTTP CONNECT tunnel,
  Basic auth, `NO_PROXY` respected). Without a proxy nothing changes.
- fix: with `rustls_backend` the crates now select exactly one rustls crypto provider (`ring`). Without
  it, rustls 0.23 panics at the first TLS handshake ("Could not automatically determine the
  process-level CryptoProvider") when no other crate in the build picks one. Applications that
  install their own provider with `CryptoProvider::install_default` keep working.
- fix(example): `e26_declarative_slash_bot` sets the new `choices` field.
- test: ignored live smoke tests `oxcord-gateway/tests/live_connect.rs` and
  `oxcord-rest/tests/live_rest.rs` (run with `-- --ignored`; they need internet access).

Publish order (each crate must be on crates.io before the ones that depend on it): `oxcord-rest`,
`oxcord-gateway`, `oxcord-voice`, `oxcord-builders`, `oxcord-proxy`, `oxcord-framework`, `oxcord`,
`oxcord-gateway-daemon`.

## oxcord 0.2.2 / oxcord-framework 0.2.3 / oxcord-macros 0.2.1 / oxcord-rest 0.1.1

- feat(macros): atribut `param("nama", "deskripsi"[, "required"|"optional"])` pada `#[command]` untuk
  menimpa deskripsi opsi slash dan status wajibnya (sebelumnya deskripsi selalu "x parameter" dan
  `Option<T>` selalu tidak wajib).
- feat(framework): pilihan tetap (choices) kini dikirim ke Discord. `FrameworkArgument::choices()`
  (bawaan kosong), `CommandOptionSpec::choices`, dan `#[derive(ChoiceParameter)]` mengisinya; varian
  boleh diberi label dengan `#[choice("Label")]` (nilai yang dikirim balik tetap nama varian).
  `ChoiceParameterInfo` mendapat field `value`. Catatan: menambah field publik ke
  `CommandOptionSpec` dan `ChoiceParameterInfo`; pemakai yang membuat struct itu dengan literal
  perlu menambahkan field baru (kode hasil makro otomatis ikut).
- feat(framework): `ReplyHandle::delete()` (slash: respons asli; prefix: pesan balasan) dan
  `Context::message_id()` (ID pesan pemanggil untuk prefix command, mis. untuk membaca balasan).
- feat(rest): `Http::ban_user_seconds` menghapus pesan pengguna per detik (Discord menerima
  0–604800); `ban_user` tetap ada dan memanggilnya.

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
