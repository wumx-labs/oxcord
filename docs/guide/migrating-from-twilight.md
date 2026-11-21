# Migrating from Twilight to OXCORD

Twilight users appreciate granular crates and zero-copy performance. OXCORD delivers the same modularity with vastly superior ergonomics and unified documentation.

---

## 🔄 Architectural Comparison

| Feature | Twilight | OXCORD |
|---|---|---|
| **Ecosystem** | 10+ fragmented crates | Granular workspace + unified `oxcord` facade |
| **Voice Support** | None built-in | Voice v8 with AEAD & RTP driver built-in |
| **Ergonomics** | Low-level / verbose | Type-safe builders + `Context` helpers |
| **Cache** | In-memory only | Pluggable (InMemory + Redis Key-Value) |
| **TLS** | Manual backend selection | Pure `rustls` by default |

---

## 🛠️ From Twilight Crates to OXCORD

| Twilight Crate | OXCORD Equivalent |
|---|---|
| `twilight-model` | `oxcord-model` |
| `twilight-http` | `oxcord-rest` |
| `twilight-gateway` | `oxcord-gateway` |
| `twilight-cache-inmemory` | `oxcord-cache` (`InMemoryBackend` or `KeyValueStateBackend`) |
| (None) | `oxcord-voice` |
| `twilight-util` | `oxcord-builders` |

You can use the unified `oxcord` facade crate or import individual crates directly just like you did with Twilight!
