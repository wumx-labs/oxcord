use std::error::Error as StdError;
use std::fmt;

use oxcord_model::json::JsonError;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use url::ParseError as UrlError;

/// An error that occurred while attempting to deal with the gateway.
#[derive(Debug)]
#[non_exhaustive]
pub enum GatewayError {
    /// There was an error building a URL.
    BuildingUrl,
    /// The connection closed, potentially uncleanly.
    Closed(Option<CloseFrame<'static>>),
    /// Expected a Hello during a handshake
    ExpectedHello,
    /// When there was an error sending a heartbeat.
    HeartbeatFailed,
    /// When invalid authentication (a bad token) was sent in the IDENTIFY.
    InvalidAuthentication,
    /// Expected a Ready or an InvalidateSession
    InvalidHandshake,
    /// When invalid sharding data was sent in the IDENTIFY.
    InvalidShardData,
    /// When no authentication was sent in the IDENTIFY.
    NoAuthentication,
    /// When a session Id was expected (for resuming), but was not present.
    NoSessionId,
    /// When a shard would have too many guilds assigned to it.
    OverloadedShard,
    /// Failed to reconnect after a number of attempts.
    ReconnectFailure,
    /// When undocumented gateway intents are provided.
    InvalidGatewayIntents,
    /// When disallowed gateway intents are provided.
    DisallowedGatewayIntents,
    /// Underlying WebSocket error.
    Tungstenite(TungsteniteError),
    /// JSON serialization/deserialization error.
    Json(JsonError),
    /// I/O error (e.g. zlib decompression).
    Io(std::io::Error),
    /// URL parsing error.
    Url(UrlError),
    /// Model error.
    Model(oxcord_model::error::Error),
    /// Other error message.
    Other(String),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildingUrl => f.write_str("Error building url"),
            Self::Closed(_) => f.write_str("Connection closed"),
            Self::ExpectedHello => f.write_str("Expected a Hello"),
            Self::HeartbeatFailed => f.write_str("Failed sending a heartbeat"),
            Self::InvalidAuthentication => f.write_str("Sent invalid authentication"),
            Self::InvalidHandshake => f.write_str("Expected a valid Handshake"),
            Self::InvalidShardData => f.write_str("Sent invalid shard data"),
            Self::NoAuthentication => f.write_str("Sent no authentication"),
            Self::NoSessionId => f.write_str("No Session Id present when required"),
            Self::OverloadedShard => f.write_str("Shard has too many guilds"),
            Self::ReconnectFailure => f.write_str("Failed to Reconnect"),
            Self::InvalidGatewayIntents => f.write_str("Invalid gateway intents were provided"),
            Self::DisallowedGatewayIntents => {
                f.write_str("Disallowed gateway intents were provided")
            },
            Self::Tungstenite(e) => write!(f, "WebSocket error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Url(e) => write!(f, "URL error: {e}"),
            Self::Model(e) => write!(f, "Model error: {e}"),
            Self::Other(e) => write!(f, "{e}"),
        }
    }
}

impl StdError for GatewayError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Tungstenite(inner) => Some(inner),
            Self::Json(inner) => Some(inner),
            Self::Io(inner) => Some(inner),
            Self::Url(inner) => Some(inner),
            Self::Model(inner) => Some(inner),
            _ => None,
        }
    }
}

impl From<TungsteniteError> for GatewayError {
    fn from(error: TungsteniteError) -> Self {
        Self::Tungstenite(error)
    }
}

impl From<JsonError> for GatewayError {
    fn from(error: JsonError) -> Self {
        Self::Json(error)
    }
}

impl From<std::io::Error> for GatewayError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<UrlError> for GatewayError {
    fn from(error: UrlError) -> Self {
        Self::Url(error)
    }
}

impl From<oxcord_model::error::Error> for GatewayError {
    fn from(error: oxcord_model::error::Error) -> Self {
        Self::Model(error)
    }
}
