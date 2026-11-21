# What is OXCORD?

**OXCORD** is a high-performance, modular Discord API library written in modern Rust (Edition 2024, Rust 1.85+). It is engineered for maximum throughput, memory safety, and developer ergonomics, scaling from single-shard enthusiast bots to distributed million-guild enterprise deployments.

---

## 🎯 Motivation

Traditional monolithic Discord libraries in Rust often suffer from:
1. **Monolithic Bloat**: Forcing users to compile the entire Discord API, Gateway, Cache, and HTTP client even if they only needed data models or a webhook interaction handler.
2. **Outdated Voice Protocols**: Legacy voice implementations relying on deprecated `xsalsa20_poly1305` ciphers and older voice gateway handshakes.
3. **Rigid In-Memory Caching**: State engines tightly coupled to a single machine's RAM, making multi-node horizontal scaling across Kubernetes or container clusters difficult.
4. **Heavy TLS Dependencies**: Relying on OpenSSL / `pkg-config` system binaries that break easily in minimal Docker containers and cross-compilation environments.

**OXCORD solves every single one of these challenges.**

---

## 🚀 Key Advantages

### 1. Granular Multi-Crate Workspace
OXCORD is broken down into clean, independent crates:
- **`oxcord-model`**: Pure data structures and Serde deserializers with zero networking.
- **`oxcord-rest`**: Connection-pooled, rate-limited Discord REST client.
- **`oxcord-gateway`**: High-throughput WebSocket Gateway runner supporting Zstandard (`zstd`).
- **`oxcord-cache`**: Pluggable state engine supporting both concurrent DashMap and distributed Redis.
- **`oxcord-voice` & `oxcord-voice-model`**: Voice Gateway v8, modern AEAD encryption, IP discovery, and RTP driver.
- **`oxcord-builders`**: Ergonomic type-safe message and interaction response builders.
- **`oxcord`**: The unified top-level facade crate tying everything together.

### 2. Modern Discord Voice v8 & AEAD Encryption
Discord has deprecated older Salsa20 voice ciphers in favor of AEAD (Authenticated Encryption with Associated Data). OXCORD implements:
- `aead_xchacha20_poly1305_rtpsize`
- `aead_aes256_gcm_rtpsize`
- 74-byte UDP IP discovery packet parsing
- Preparation for Discord's end-to-end encrypted DAVE (MLS) protocol.

### 3. Gateway-less Serverless Interactions
With Discord's Interactions Endpoint URL, bots can receive Slash Commands over HTTP POST without maintaining a 24/7 WebSocket connection. OXCORD provides:
- **Zero-allocation Ed25519 verification** using stack buffers for payloads under 4 KiB.
- Built-in Ping/Pong acknowledgment.
- Direct dispatching to Autocomplete, Slash Commands, and Modals.

### 4. Zero-Friction Pure Rustls
By default, OXCORD uses `rustls` for all TLS connections. You don't need `libssl-dev`, `openssl`, or `pkg-config` installed on your host system. It compiles anywhere: Linux (musl/glibc), macOS, Windows, and scratch Docker containers.
