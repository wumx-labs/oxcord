# Gateway & Zstandard (zstd) Compression

OXCORD's Gateway runner is tuned for maximum throughput, resilience against disconnects, and minimal bandwidth consumption.

---

## 🗜️ Zstandard (`zstd`) Decompression

Discord supports modern Zstandard compression over Gateway WebSockets, which is significantly faster and achieves higher compression ratios than legacy zlib:

Enable the `zstd` feature in your `Cargo.toml`:

```toml
[dependencies]
oxcord = { version = "0.1.0", features = ["client", "gateway", "zstd"] }
```

OXCORD automatically negotiates `compress=zstd-stream` with Discord Gateway v10, utilizing shared Zstandard dictionary decompression for significantly lower payload overhead and faster decompression cycles compared to legacy zlib streams.

---

## 🔄 Gateway Resume URL v10 Specification

When Discord dispatches a `READY` event, it includes a dedicated `resume_gateway_url`. 

OXCORD records this URL and connects directly to the assigned gateway cluster on disconnects, resulting in:
- Instant session resumption without dropping voice state or guild data.
- Elimination of full `IDENTIFY` re-handshakes that count against Discord's 1000/day daily identify limit.

---

## 🎲 Full-Jitter Exponential Backoff

When network outages or Discord downtime occur, thousands of shards attempting to reconnect simultaneously can overwhelm both local sockets and remote Discord servers (the "Thundering Herd" problem).

OXCORD implements **Full Jitter Exponential Backoff**:

$$\text{sleep} = \text{random}(0, \min(M, B \times 2^{\text{attempts}}))$$

Where $B$ is the base delay and $M$ is the maximum ceiling. This distributes reconnections evenly across time and ensures high reliability.
