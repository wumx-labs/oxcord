//! Error types for the broker system.

use std::fmt;

/// Result alias for broker operations.
pub type Result<T> = std::result::Result<T, BrokerError>;

/// Errors that can occur during message broker operations.
#[derive(Debug)]
pub enum BrokerError {
    /// Serialization or deserialization error.
    Serialization(serde_json::Error),
    /// Transport or underlying driver error.
    Transport(String),
    /// Request-response (RPC) timed out waiting for a reply.
    Timeout(String),
    /// Subscription channel was closed or dropped.
    SubscriptionClosed,
    /// No responders available for the requested RPC topic.
    NoResponder(String),
    /// General or custom broker error.
    Other(String),
}

impl fmt::Display for BrokerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(e) => write!(f, "broker serialization error: {e}"),
            Self::Transport(msg) => write!(f, "broker transport error: {msg}"),
            Self::Timeout(msg) => write!(f, "broker RPC timeout: {msg}"),
            Self::SubscriptionClosed => write!(f, "broker subscription channel closed"),
            Self::NoResponder(topic) => write!(f, "no responder available for topic: {topic}"),
            Self::Other(msg) => write!(f, "broker error: {msg}"),
        }
    }
}

impl std::error::Error for BrokerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for BrokerError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}

impl From<tokio::sync::broadcast::error::RecvError> for BrokerError {
    fn from(err: tokio::sync::broadcast::error::RecvError) -> Self {
        match err {
            tokio::sync::broadcast::error::RecvError::Closed => Self::SubscriptionClosed,
            tokio::sync::broadcast::error::RecvError::Lagged(missed) => {
                Self::Transport(format!("subscriber lagged by {missed} messages"))
            },
        }
    }
}
