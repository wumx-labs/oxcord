# Modular Architecture

OXCORD is structured as a Cargo workspace with distinct, specialized crates. This modularity ensures fast compile times, zero bloat, and the flexibility to use individual components independently.

---

## 🏛️ Crates Overview

```
                        ┌──────────────────┐
                        │      oxcord      │  (Unified Facade)
                        └─────────┬────────┘
        ┌──────────────┬──────────┴────────┬──────────────┐
        ▼              ▼                   ▼              ▼
┌──────────────┐ ┌──────────────┐   ┌──────────────┐ ┌──────────────┐
│ oxcord-model │ │ oxcord-rest  │   │oxcord-gateway│ │ oxcord-cache │
└──────────────┘ └──────────────┘   └──────────────┘ └──────────────┘
        ▲              ▲                   ▲              ▲
        └──────────────┴──────────┬────────┴──────────────┘
                                  ▼
                   ┌──────────────────────────────┐
                   │  oxcord-voice / voice-model  │
                   └──────────────────────────────┘
```

---

### 1. `oxcord-model`
- **Role**: Pure data types representing all Discord entities.
- **Dependencies**: No networking dependencies (`reqwest` or `tungstenite` are NOT included here). Only `serde`, `bitflags`, and lightweight utility libraries.
- **Use Cases**: Useful if you want to parse Discord payloads in custom microservices, event queues, or frontend bridges.

### 2. `oxcord-rest`
- **Role**: HTTP client for interacting with the Discord REST API v10.
- **Features**:
  - Connection pooling with keep-alive tuning.
  - Per-route ratelimiting adhering strictly to Discord headers (`X-RateLimit-*`).
  - Multipart uploads for attachments, files, and avatars.

### 3. `oxcord-gateway`
- **Role**: WebSocket shard runner managing connection lifecycles.
- **Features**:
  - Discord Gateway Resume URL v10 support for fast shard reconnects.
  - Full-jitter exponential backoff to eliminate server thundering herd.
  - Modern Zstandard (`zstd`) stream and packet decompression alongside zlib-inflate.
  - Heartbeat manager maintaining keep-alive intervals with sequence tracking.

### 4. `oxcord-cache`
- **Role**: State and cache engine.
- **Features**:
  - `InMemoryBackend`: Lock-sharded `DashMap` for concurrent sub-millisecond lookups.
  - `KeyValueStateBackend`: Trait-based backend supporting Redis, KeyDB, and distributed KV stores.
  - `StateEngine`: Event dispatcher orchestrating state updates from Gateway or Webhook events.

### 5. `oxcord-voice` & `oxcord-voice-model`
- **Role**: Voice gateway connection and audio streaming driver.
- **Features**:
  - Voice Gateway v8 handshake.
  - Modern AEAD cipher modes: `aead_xchacha20_poly1305_rtpsize` and `aead_aes256_gcm_rtpsize` using `ring`.
  - 74-byte UDP IP discovery packet.
  - Real-time RTP packetizer transmitting 48 kHz Opus frames at 20ms intervals.

### 6. `oxcord-builders`
- **Role**: Type-safe, ergonomic fluent builders.
- **Features**: `CreateMessage`, `CreateInteractionResponse`, `CreateAutocompleteResponse`, `CreateModal`, and Components V2.
