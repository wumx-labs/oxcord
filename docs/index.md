---
layout: home

hero:
  name: "OXCORD"
  text: "Next-Gen Rust Discord Library"
  tagline: "Modular multi-crate architecture, Zstandard compression, AEAD voice v8, zero-alloc webhooks, and pluggable distributed caching."
  image:
    light: /logo-light-transparent.png
    dark: /logo-transparent.png
    alt: OXCORD Logo
  actions:
    - theme: brand
      text: Get Started →
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/wumx-labs/oxcord

features:
  - icon: 🧩
    title: Modular Multi-Crate
    details: Pick only what you need. Use `oxcord-model` standalone, or combine `oxcord-gateway`, `oxcord-cache`, `oxcord-voice`, and `oxcord-rest`.
  - icon: ⚡
    title: Zstd & Fast Gateway
    details: Official v10 Resume URL specification, modern Zstandard payload decompression, and full-jitter exponential reconnect backoff.
  - icon: 🎙️
    title: Modern Voice v8 & AEAD
    details: Discord Voice Gateway v8 with AEAD ciphers (`aead_xchacha20_poly1305_rtpsize`, `aead_aes256_gcm_rtpsize`) and DAVE protocol ready.
  - icon: 🔒
    title: Zero-Alloc Webhooks
    details: Gateway-less serverless bots with stack-accelerated Ed25519 signature verification, Autocomplete, and Modal handling.
  - icon: 🗄️
    title: Pluggable State Engine
    details: Seamlessly switch between concurrent in-memory DashMap and distributed Redis / Key-Value stores for million-guild scale.
  - icon: 🦀
    title: Pure Rust 2024 & Rustls
    details: Built on modern Rust Edition 2024 (1.85+) with 100% memory-safe pure Rustls TLS — no OpenSSL / pkg-config headaches.
---
