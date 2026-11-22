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
      text: 🤖 AI Docs (llms.txt)
      link: /llms.txt
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
    details: High-concurrency lock-free DashMap state cache with pluggable KeyValueStateBackend trait for distributed external stores.
  - icon: 🦀
    title: Pure Rust 2024 & Rustls
    details: Built on modern Rust Edition 2024 (1.85+) with 100% memory-safe pure Rustls TLS — no OpenSSL / pkg-config headaches.
---

<div class="tip custom-block" style="padding: 1.25rem 1.5rem; margin: 2rem 0; border-radius: 8px; border-left: 4px solid var(--vp-c-brand-1); background-color: var(--vp-c-bg-soft);">
  <p class="custom-block-title" style="font-weight: 600; font-size: 1.1rem; margin-bottom: 0.5rem;">
    🤖 First-Class AI Agent & LLM Documentation
  </p>
  <p style="margin: 0.25rem 0;">
    Building your bot with <b>Claude, Cursor, Copilot, Codex, or ChatGPT</b>? OXCORD provides official single-file machine-readable documentation endpoints:
  </p>
  <ul style="margin: 0.5rem 0 0.5rem 1.25rem;">
    <li><b>Standard Index:</b> <a href="/llms.txt" target="_blank"><code>https://oxcord.noxlydev.xyz/llms.txt</code></a> — Lightweight directory of all guides, architecture summaries, and crate references.</li>
    <li><b>Full AI Context:</b> <a href="/llms-full.txt" target="_blank"><code>https://oxcord.noxlydev.xyz/llms-full.txt</code></a> — Complete uncompressed single-file context (~95 KB) containing all guides, Cargo.toml manifests, and crate API signatures.</li>
  </ul>
</div>

<div style="text-align: center; margin: 3rem 0 1rem;">
  <p style="font-size: 0.95rem; color: var(--vp-c-text-2);">
    Open source under Apache-2.0 License. Maintained by <a href="https://github.com/NoxlyDev" target="_blank">NoxlyDev</a> & <a href="https://github.com/wumx-labs" target="_blank">wumx-labs</a>.
  </p>
</div>

