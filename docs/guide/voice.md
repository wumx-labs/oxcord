# Modern Voice Architecture (v8 & AEAD)

OXCORD introduces a completely modernized Voice architecture engineered for Discord's Voice Gateway v8 and modern AEAD encryption standards.

---

## 🔒 Supported AEAD Encryption Modes

Discord has deprecated legacy `xsalsa20_poly1305` ciphers. OXCORD only uses modern, cryptographically validated AEAD ciphers:

1. **`aead_xchacha20_poly1305_rtpsize`** (Recommended default)
   - XChaCha20 stream cipher with Poly1305 authenticator.
   - 4-byte big-endian incremental nonce counter appended to the RTP packet.
2. **`aead_aes256_gcm_rtpsize`**
   - Hardware-accelerated AES-256 in Galois/Counter Mode.

Both encryption modes are powered by the hardware-accelerated `ring` cryptographic engine (using AES-NI and ARM NEON instructions).

---

## 🎙️ Joining and Leaving Voice Channels

OXCORD makes voice channel management intuitive through [`VoiceStateTracker`]:

```rust
use oxcord::prelude::*;

// Joining a channel:
ctx.join_voice_channel(guild_id, channel_id, false, false).await?;

// Leaving a channel:
ctx.leave_voice_channel(guild_id).await?;
```

---

## ⚡ Voice Handshake Lifecycle

```
[ Bot ]                                                  [ Discord Gateway ]
   │                                                             │
   │─── Opcode 4: Update Voice State (guild_id, channel_id) ────▶│
   │                                                             │
   │◀── Event: VOICE_STATE_UPDATE (session_id) ──────────────────│
   │◀── Event: VOICE_SERVER_UPDATE (endpoint, token) ────────────│
   │                                                             │
[ Bot ]                                              [ Discord Voice Server ]
   │                                                             │
   │─── Connect Voice WebSocket (wss://{endpoint}/?v=8) ────────▶│
   │─── Opcode 0: Identify (server_id, user_id, session, token) ─▶│
   │◀── Opcode 2: Ready (ssrc, ip, port, modes) ─────────────────│
   │                                                             │
   │─── UDP IP Discovery (74-byte packet) ──────────────────────▶│
   │◀── UDP IP Discovery Response (external IP & port) ──────────│
   │                                                             │
   │─── Opcode 1: Select Protocol (aead_xchacha20_poly1305_rtp) ─▶│
   │◀── Opcode 4: Session Description (secret_key) ──────────────│
   │                                                             │
   │═══ Stream RTP Encrypted Opus Audio (20ms frames) ══════════▶│
```

---

## 🎵 RTP Audio Pipeline

Audio streaming transmits 48 kHz Opus frames (960 samples per frame at 20ms intervals):
- **RTP Header**: Payload type `120`, monotonically increasing sequence number and timestamp.
- **Payload**: Opus frame encrypted with `VoiceCryptor`.
- **Nonce Counter**: 4-byte counter appended to the packet tail.

---

## 🛡️ Discord DAVE (E2EE) Protocol Status

> [!NOTE] DAVE Protocol Roadmap
> Discord is rolling out the **DAVE (Discord Audio & Video End-to-End Encryption)** protocol based on Messaging Layer Security (MLS). 
> In OXCORD, DAVE protocol support is currently **in development / planned on the roadmap** as the upstream specification stabilizes. Voice currently operates securely over Discord Voice Gateway v8 with transport-layer AEAD ciphers (`aead_xchacha20_poly1305_rtpsize` and `aead_aes256_gcm_rtpsize`).

