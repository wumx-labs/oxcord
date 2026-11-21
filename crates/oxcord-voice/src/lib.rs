//! # oxcord-voice
//!
//! Native Discord Voice Gateway client, RTP audio packetizer, AEAD audio encryption,
//! and real-time voice streaming driver for OXCORD.
//!
//! ## Architecture
//! - [`VoiceConnection`]: Orchestrates Voice Gateway v4/v8 WebSocket handshake, IP discovery, and
//!   session negotiation.
//! - [`VoiceDriver`]: Real-time audio sender over UDP, sequencing RTP headers and encrypting Opus
//!   frames.
//! - [`VoiceCryptor`]: Modern Discord AEAD encryption (`aead_aes256_gcm_rtpsize` and
//!   `aead_xchacha20_poly1305_rtpsize`).
//! - [`RtpHeader`] & [`RtpPacketizer`]: RFC 3550 RTP packetization with 48 kHz Opus audio frames.
//! - [`ip_discovery`]: NAT hole-punching UDP IP discovery for Discord voice calls.

pub mod connection;
pub mod crypto;
pub mod driver;
pub mod error;
pub mod ip_discovery;
pub mod rtp;
pub mod ws;

pub use connection::VoiceConnection;
pub use crypto::{AEAD_TAG_SIZE, CipherAlgorithm, NONCE_COUNTER_SIZE, VoiceCryptor};
pub use driver::VoiceDriver;
pub use error::{Result, VoiceError};
pub use ip_discovery::{
    IP_DISCOVERY_PACKET_SIZE,
    create_ip_discovery_request,
    parse_ip_discovery_response,
    perform_ip_discovery,
};
// Re-export voice model definitions
pub use oxcord_voice_model::constants::{GATEWAY_VERSION, GATEWAY_VERSION_V8};
pub use oxcord_voice_model::{
    CloseCode,
    EncryptionMode,
    Event,
    Opcode,
    ProtocolData,
    SpeakingState,
};
pub use rtp::{
    CHANNELS,
    DISCORD_OPUS_PAYLOAD_TYPE,
    FRAME_DURATION_MS,
    OPUS_SILENCE_FRAME,
    RTP_HEADER_SIZE,
    RtpHeader,
    RtpPacketizer,
    SAMPLE_RATE,
    SAMPLES_PER_FRAME,
};
