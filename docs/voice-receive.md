# Voice receive: design note

Status: research only, no code. Written for the owner to decide whether receive support is worth doing.

## Goal

Let a bot record a voice channel (the Alya `record` command). `oxcord-voice` today only sends audio.

## What exists today

`crates/oxcord-voice` (about 1100 lines): voice websocket (`ws.rs`), UDP IP discovery, RTP header
parse and build (`rtp.rs`), AEAD transport encryption (`crypto.rs`, AES-256-GCM and XChaCha20-Poly1305,
`rtpsize` modes) and a send-only `VoiceDriver`. `VoiceCryptor::decrypt_rtp_packet` already exists, and
`oxcord-voice-model` already parses the `Speaking` event and defines the DAVE opcodes 21 to 31.
Nothing in `oxcord-voice` or in the bridge (`crates/oxcord/src/gateway/bridge/voice.rs`) handles DAVE.

## What Discord requires

1. **UDP receive.** After IP discovery and Select Protocol, the same UDP socket also receives RTP
   packets from other users. Each packet has an RTP header with an SSRC, then transport-encrypted
   data. The existing `decrypt_rtp_packet` covers this layer.
2. **SSRC to user mapping.** Voice `Speaking` events (opcode 5) carry `ssrc` and `user_id` (sent by the
   server when a user starts speaking; a bot must also remember the mapping from Client Connect events).
   Without it, packets cannot be attributed to a user. Keep a `HashMap<u32, UserId>` per connection.
3. **Jitter handling.** Packets arrive out of order or lost; use the RTP sequence number and timestamp
   to reorder and to insert silence for gaps, otherwise recorded files drift out of sync.
4. **Opus decode.** Payload type 120, 48 kHz, 20 ms frames, stereo. Needs a decoder (libopus) and, for
   a file, a container or a raw PCM/WAV writer. Opus passthrough into an Ogg container avoids decoding
   (no libopus), at the cost of mixing channels later.
5. **DAVE end-to-end encryption.** Discord made DAVE mandatory in 2026: voice and video calls
   (including bots) are end-to-end encrypted, stage channels excluded
   ([Discord blog](https://discord.com/blog/every-voice-and-video-call-on-discord-is-now-end-to-end-encrypted),
   [bots and platforms](https://discord.com/blog/bringing-dave-to-all-discord-platforms)); clients
   without DAVE support are reported to be refused from calls since March 2026. DAVE adds an MLS
   group on top of the transport encryption: each Opus frame is encrypted with a per-sender key, so after
   removing the transport layer the receiver still has to decrypt each frame with the sender key
   derived from the MLS group. The voice gateway side is opcodes 21 to 31 (prepare transition, execute
   transition, ready, prepare epoch, MLS external sender, key package, proposals, commit/welcome,
   announce commit, welcome, invalid commit/welcome) plus the `max_dave_protocol_version` field in
   Identify. Reference library: [discord/libdave](https://github.com/discord/libdave) (C++), protocol
   whitepaper in `discord/dave-protocol`.

## Important finding for the existing send path

`oxcord-voice` has no DAVE handling at all (no `max_dave_protocol_version`, no MLS). If the reports
above hold for bots, **even playback is likely refused or silent today** (voice websocket closed with a
DAVE-related code, or audio the other side cannot decrypt). This was not tested here (no live voice
test, no token). It should be checked first with one live playback test (example `e21_voice_playback`),
because DAVE is a prerequisite for receive and for send.

## Size of the change for `oxcord-voice`

| Piece | Work |
|---|---|
| DAVE: MLS group, key ratchet, frame encrypt/decrypt, transitions and epochs | Large. Either bind libdave (C++, build toolchain, FFI) or use a Rust MLS stack. This is the bulk of the work and is needed for send too. |
| Receive loop: socket read, decrypt, SSRC map, per-user stream, jitter buffer | Medium. Transport decrypt and RTP parse exist. |
| Opus decode and file output | Small to medium. |
| Public API (a `VoiceReceiver` trait or channel of `(UserId, pcm/opus)`) in `oxcord-voice` and the `oxcord` voice bridge | Small to medium. |
| Tests | Receive path can use captured or synthetic packets. DAVE needs a live server or libdave test vectors. |

## Crates

| Need | Candidate | License | Note |
|---|---|---|---|
| DAVE reference | `discord/libdave` (C++) via FFI | not verified here | check before adding; needs a C++ build in CI |
| DAVE, Rust | `davey` crate | MIT (from its Cargo.toml on docs.rs) | built on `openmls` pinned to `=0.7.2`; maturity not assessed |
| DAVE, Rust | `dave` crate | AGPL-3.0-only (docs.rs) | **do not use**: not in the allowed list |
| MLS | `openmls` | not verified here | check |
| Opus decode | `audiopus` or `opus` crates over libopus | wrapper licenses not verified; libopus itself is BSD-3-Clause ([opus-codec.org](https://opus-codec.org/license/)) | needs `libopus` at build time, or the bundled build feature |
| Opus passthrough | `ogg` crate | not verified here | avoids decoding |
| WAV output | `hound` | not verified here | |

All licenses marked "not verified" must be confirmed against `cargo deny` (allowed: MIT, Apache-2.0,
BSD-2/3, ISC, Unicode-3.0, Zlib, MPL-2.0, CC0-1.0, OpenSSL, BSL-1.0, CDLA-Permissive-2.0) before any
dependency is added.

## Effort estimate (rough)

- Verify the current send path against Discord with DAVE: half a day.
- DAVE (via `davey` or libdave bindings), including epoch and transition handling: 1 to 2 weeks,
  most of the risk, testable only live.
- Receive loop, SSRC map, jitter buffer: 2 to 4 days.
- Opus decode, WAV or Ogg writer, API, docs, tests: 3 to 5 days.

Total: about 2 to 4 weeks of focused work, dominated by DAVE.

## Recommendation

Do the DAVE work first, as a separate task, because it also fixes playback if playback is broken.
Only after DAVE works for sending, add receive; at that point it is a moderate extension. Doing receive
alone is not possible. For Alya, the `record` command alone does not justify the DAVE work; the
playback fix does. If the owner wants a cheaper path for `record`, a bot can stay on the old JS stack
for that single command until DAVE is in `oxcord-voice`.

## Sources

- [Discord: every voice and video call is now end-to-end encrypted](https://discord.com/blog/every-voice-and-video-call-on-discord-is-now-end-to-end-encrypted)
- [Discord: bringing DAVE to all Discord platforms](https://discord.com/blog/bringing-dave-to-all-discord-platforms)
- [Discord: meet DAVE](https://discord.com/blog/meet-dave-e2ee-for-audio-video)
- [`davey` on docs.rs](https://docs.rs/crate/davey/0.1.2/source/Cargo.toml), [`dave` on docs.rs](https://docs.rs/crate/dave/latest)
