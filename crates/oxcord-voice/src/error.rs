use std::{fmt, io};

/// Results returned by operations in `oxcord-voice`.
pub type Result<T> = std::result::Result<T, VoiceError>;

/// Errors that can occur during Voice Gateway operations or audio packet processing.
#[derive(Debug)]
pub enum VoiceError {
    /// An I/O error occurred (e.g. UDP socket error).
    Io(io::Error),
    /// A WebSocket error occurred.
    WebSocket(tokio_tungstenite::tungstenite::Error),
    /// A JSON serialization or deserialization error occurred.
    Json(serde_json::Error),
    /// Cryptographic error during encryption or decryption.
    Crypto(&'static str),
    /// Protocol error during Voice Gateway handshake.
    Protocol(String),
    /// IP discovery failed or timed out.
    IpDiscovery(String),
    /// The voice connection was closed or invalidated.
    ConnectionClosed(String),
}

impl fmt::Display for VoiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "Voice I/O error: {err}"),
            Self::WebSocket(err) => write!(f, "Voice WebSocket error: {err}"),
            Self::Json(err) => write!(f, "Voice JSON error: {err}"),
            Self::Crypto(msg) => write!(f, "Voice crypto error: {msg}"),
            Self::Protocol(msg) => write!(f, "Voice protocol error: {msg}"),
            Self::IpDiscovery(msg) => write!(f, "Voice IP discovery error: {msg}"),
            Self::ConnectionClosed(msg) => write!(f, "Voice connection closed: {msg}"),
        }
    }
}

impl std::error::Error for VoiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::WebSocket(err) => Some(err),
            Self::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for VoiceError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for VoiceError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(err)
    }
}

impl From<serde_json::Error> for VoiceError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}
