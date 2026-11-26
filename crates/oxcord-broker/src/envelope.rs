//! Message envelope format for cross-process event broadcasting and RPC.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Header key for reply topic in RPC calls.
pub const HEADER_REPLY_TO: &str = "oxcord-reply-to";
/// Header key for correlation ID in RPC calls.
pub const HEADER_CORRELATION_ID: &str = "oxcord-correlation-id";
/// Header key for Discord shard ID.
pub const HEADER_SHARD_ID: &str = "oxcord-shard-id";
/// Header key for Discord event name.
pub const HEADER_EVENT_NAME: &str = "oxcord-event-name";

/// A standardized message envelope passed across broker topics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrokerMessage {
    /// Target or source topic of the message.
    pub topic: String,
    /// Raw binary or JSON payload.
    pub payload: Vec<u8>,
    /// Metadata headers (e.g., shard ID, correlation ID for RPC).
    pub headers: HashMap<String, String>,
}

impl BrokerMessage {
    /// Creates a new message envelope with empty headers.
    pub fn new(topic: impl Into<String>, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            topic: topic.into(),
            payload: payload.into(),
            headers: HashMap::new(),
        }
    }

    /// Serializes a Rust type into JSON as the message payload.
    ///
    /// # Errors
    /// Returns [`BrokerError::Serialization`](crate::BrokerError::Serialization) if serialization fails.
    pub fn from_json<T: Serialize>(topic: impl Into<String>, value: &T) -> Result<Self> {
        let payload = serde_json::to_vec(value)?;
        Ok(Self::new(topic, payload))
    }

    /// Deserializes the message payload from JSON into `T`.
    ///
    /// # Errors
    /// Returns [`BrokerError::Serialization`](crate::BrokerError::Serialization) if deserialization fails.
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        Ok(serde_json::from_slice(&self.payload)?)
    }

    /// Attaches a metadata header to the message.
    #[must_use]
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Attaches a Discord shard ID header.
    #[must_use]
    pub fn with_shard(self, shard_id: u32) -> Self {
        self.with_header(HEADER_SHARD_ID, shard_id.to_string())
    }

    /// Attaches an event name header (e.g., `MESSAGE_CREATE`, `GUILD_CREATE`).
    #[must_use]
    pub fn with_event_name(self, event_name: impl Into<String>) -> Self {
        self.with_header(HEADER_EVENT_NAME, event_name)
    }

    /// Attaches an RPC reply-to topic header.
    #[must_use]
    pub fn with_reply_to(self, reply_to: impl Into<String>) -> Self {
        self.with_header(HEADER_REPLY_TO, reply_to)
    }

    /// Attaches an RPC correlation ID header.
    #[must_use]
    pub fn with_correlation_id(self, correlation_id: impl Into<String>) -> Self {
        self.with_header(HEADER_CORRELATION_ID, correlation_id)
    }

    /// Returns the reply topic if this message is an RPC request.
    #[must_use]
    pub fn reply_to(&self) -> Option<&str> {
        self.headers.get(HEADER_REPLY_TO).map(String::as_str)
    }

    /// Returns the correlation ID if present.
    #[must_use]
    pub fn correlation_id(&self) -> Option<&str> {
        self.headers.get(HEADER_CORRELATION_ID).map(String::as_str)
    }

    /// Returns the shard ID if present and valid.
    #[must_use]
    pub fn shard_id(&self) -> Option<u32> {
        self.headers.get(HEADER_SHARD_ID).and_then(|s| s.parse().ok())
    }

    /// Returns the event name if present.
    #[must_use]
    pub fn event_name(&self) -> Option<&str> {
        self.headers.get(HEADER_EVENT_NAME).map(String::as_str)
    }

    /// Checks if this message represents an RPC request expecting a reply.
    #[must_use]
    pub fn is_rpc_request(&self) -> bool {
        self.headers.contains_key(HEADER_REPLY_TO)
            && self.headers.contains_key(HEADER_CORRELATION_ID)
    }
}
