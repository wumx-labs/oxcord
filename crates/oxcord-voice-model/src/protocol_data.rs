use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Discord voice encryption modes.
///
/// Discord requires AEAD cipher modes (`aead_xchacha20_poly1305_rtpsize` or
/// `aead_aes256_gcm_rtpsize`). Older legacy `xsalsa20_poly1305*` modes have been
/// deprecated and discontinued by Discord.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum EncryptionMode {
    /// Modern AEAD mode using XChaCha20-Poly1305 with standard RTP size header handling.
    ///
    /// This is the required baseline mode for all Discord voice clients.
    AeadXChaCha20Poly1305RtpSize,
    /// Modern AEAD mode using AES-256-GCM with standard RTP size header handling.
    ///
    /// Preferred mode when hardware AES acceleration is available.
    AeadAes256GcmRtpSize,
    /// Legacy XSalsa20-Poly1305 mode.
    #[deprecated(since = "0.0.1", note = "Discontinued by Discord; use AEAD modes")]
    XSalsa20Poly1305,
    /// Legacy XSalsa20-Poly1305 suffix mode.
    #[deprecated(since = "0.0.1", note = "Discontinued by Discord; use AEAD modes")]
    XSalsa20Poly1305Suffix,
    /// Legacy XSalsa20-Poly1305 lite mode.
    #[deprecated(since = "0.0.1", note = "Discontinued by Discord; use AEAD modes")]
    XSalsa20Poly1305Lite,
    /// Unknown or future encryption mode for forward-compatibility.
    Unknown(String),
}

impl EncryptionMode {
    /// Returns the string representation of this encryption mode as expected by Discord.
    #[must_use]
    pub fn as_str(&self) -> &str {
        #[allow(deprecated)]
        match self {
            Self::AeadXChaCha20Poly1305RtpSize => "aead_xchacha20_poly1305_rtpsize",
            Self::AeadAes256GcmRtpSize => "aead_aes256_gcm_rtpsize",
            Self::XSalsa20Poly1305 => "xsalsa20_poly1305",
            Self::XSalsa20Poly1305Suffix => "xsalsa20_poly1305_suffix",
            Self::XSalsa20Poly1305Lite => "xsalsa20_poly1305_lite",
            Self::Unknown(s) => s.as_str(),
        }
    }

    /// Returns `true` if this encryption mode is an AEAD mode (`aead_*`).
    #[must_use]
    pub fn is_aead(&self) -> bool {
        matches!(self, Self::AeadXChaCha20Poly1305RtpSize | Self::AeadAes256GcmRtpSize)
    }

    /// Returns `true` if this mode is currently active and supported by modern Discord voice.
    #[must_use]
    pub fn is_supported(&self) -> bool {
        self.is_aead()
    }
}

impl fmt::Display for EncryptionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EncryptionMode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[allow(deprecated)]
        Ok(match s {
            "aead_xchacha20_poly1305_rtpsize" => Self::AeadXChaCha20Poly1305RtpSize,
            "aead_aes256_gcm_rtpsize" => Self::AeadAes256GcmRtpSize,
            "xsalsa20_poly1305" => Self::XSalsa20Poly1305,
            "xsalsa20_poly1305_suffix" => Self::XSalsa20Poly1305Suffix,
            "xsalsa20_poly1305_lite" => Self::XSalsa20Poly1305Lite,
            other => Self::Unknown(other.to_string()),
        })
    }
}

impl From<&str> for EncryptionMode {
    fn from(s: &str) -> Self {
        s.parse().expect("Infallible")
    }
}

impl From<String> for EncryptionMode {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<EncryptionMode> for String {
    fn from(mode: EncryptionMode) -> Self {
        mode.as_str().to_string()
    }
}

impl Serialize for EncryptionMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EncryptionMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(s.into())
    }
}

/// The client's response to a connection offer.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ProtocolData {
    /// IP address of the client as seen by the server (*e.g.*, after using [IP Discovery] for NAT
    /// hole-punching).
    ///
    /// [IP Discovery]: https://docs.rs/discortp/discord/struct.IpDiscovery.html
    pub address: IpAddr,
    /// The client's chosen encryption mode (from those offered by the server).
    pub mode: String,
    /// UDP source port of the client as seen by the server, as above.
    pub port: u16,
}

impl ProtocolData {
    /// Creates a new `ProtocolData` instance.
    pub fn new(address: IpAddr, port: u16, mode: impl Into<String>) -> Self {
        Self {
            address,
            mode: mode.into(),
            port,
        }
    }

    /// Returns the parsed [`EncryptionMode`].
    #[must_use]
    pub fn encryption_mode(&self) -> EncryptionMode {
        self.mode.as_str().into()
    }
}
