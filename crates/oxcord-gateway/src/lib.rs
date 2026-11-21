//! The gateway module contains the pieces - primarily the [`Shard`] - responsible for maintaining a
//! WebSocket connection with Discord.
//!
//! A shard is an interface for the lower-level receiver and sender. A shard represents a single
//! connection to Discord. You can make use of a method named "sharding" to have multiple shards,
//! potentially offloading some server load to another server(s).

#[macro_use]
extern crate serde;

pub(crate) mod internal;

pub mod backoff;
mod error;
mod shard;
mod ws;

use std::fmt;

use url::Url;

pub use self::backoff::Backoff;
pub use self::error::GatewayError;
pub use self::shard::Shard;
pub use self::ws::WsClient;

/// The error type used in oxcord-gateway.
pub type Error = GatewayError;
/// The result type used in oxcord-gateway.
pub type Result<T, E = GatewayError> = std::result::Result<T, E>;

use oxcord_model::gateway::{Activity, ActivityType};
use oxcord_model::id::UserId;
use oxcord_model::user::OnlineStatus;

/// Presence data of the current user.
#[derive(Clone, Debug, Default)]
pub struct PresenceData {
    /// The current activity, if present
    pub activity: Option<ActivityData>,
    /// The current online status
    pub status: OnlineStatus,
}

/// Activity data of the current user.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ActivityData {
    /// The name of the activity
    pub name: String,
    /// The type of the activity
    #[serde(rename = "type")]
    pub kind: ActivityType,
    /// The state of the activity, if the type is [`ActivityType::Custom`]
    pub state: Option<String>,
    /// The url of the activity, if the type is [`ActivityType::Streaming`]
    pub url: Option<Url>,
}

impl ActivityData {
    /// Creates an activity that appears as `Playing <name>`.
    #[must_use]
    pub fn playing(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ActivityType::Playing,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `Streaming <name>`.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL parsing fails.
    pub fn streaming(name: impl Into<String>, url: impl AsRef<str>) -> Result<Self> {
        let parsed_url = Url::parse(url.as_ref())?;
        Ok(Self {
            name: name.into(),
            kind: ActivityType::Streaming,
            state: None,
            url: Some(parsed_url),
        })
    }

    /// Creates an activity that appears as `Listening to <name>`.
    #[must_use]
    pub fn listening(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ActivityType::Listening,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `Watching <name>`.
    #[must_use]
    pub fn watching(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ActivityType::Watching,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `Competing in <name>`.
    #[must_use]
    pub fn competing(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ActivityType::Competing,
            state: None,
            url: None,
        }
    }

    /// Creates an activity that appears as `<state>`.
    #[must_use]
    pub fn custom(state: impl Into<String>) -> Self {
        Self {
            // discord seems to require a name for custom activities
            // even though it's not displayed
            name: "~".to_string(),
            kind: ActivityType::Custom,
            state: Some(state.into()),
            url: None,
        }
    }
}

impl From<Activity> for ActivityData {
    fn from(activity: Activity) -> Self {
        Self {
            name: activity.name,
            kind: activity.kind,
            state: activity.state,
            url: activity.url,
        }
    }
}

/// Indicates the current connection stage of a [`Shard`].
///
/// This can be useful for knowing which shards are currently "down"/"up".
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ConnectionStage {
    /// Indicator that the [`Shard`] is normally connected and is not in, e.g., a resume phase.
    Connected,
    /// Indicator that the [`Shard`] is connecting and is in, e.g., a resume phase.
    Connecting,
    /// Indicator that the [`Shard`] is fully disconnected and is not in a reconnecting phase.
    Disconnected,
    /// Indicator that the [`Shard`] is currently initiating a handshake.
    Handshake,
    /// Indicator that the [`Shard`] has sent an IDENTIFY packet and is awaiting a READY packet.
    Identifying,
    /// Indicator that the [`Shard`] has sent a RESUME packet and is awaiting a RESUMED packet.
    Resuming,
}

impl ConnectionStage {
    /// Whether the stage is a form of connecting.
    #[must_use]
    pub fn is_connecting(self) -> bool {
        use self::ConnectionStage::{Connecting, Handshake, Identifying, Resuming};
        matches!(self, Connecting | Handshake | Identifying | Resuming)
    }
}

impl fmt::Display for ConnectionStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::Connected => "connected",
            Self::Connecting => "connecting",
            Self::Disconnected => "disconnected",
            Self::Handshake => "handshaking",
            Self::Identifying => "identifying",
            Self::Resuming => "resuming",
        })
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ShardAction {
    Heartbeat,
    Identify,
    Reconnect(ReconnectType),
}

/// The type of reconnection that should be performed.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReconnectType {
    /// Indicator that a new connection should be made by sending an IDENTIFY.
    Reidentify,
    /// Indicator that a new connection should be made by sending a RESUME.
    Resume,
}

/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#request-guild-members).
#[derive(Clone, Debug)]
pub enum ChunkGuildFilter {
    /// Returns all members of the guilds specified. Requires GUILD_MEMBERS intent.
    None,
    /// A common username prefix filter for the members returned.
    ///
    /// Will return a maximum of 100 members.
    Query(String),
    /// A set of exact user IDs to query for.
    ///
    /// Will return a maximum of 100 members.
    UserIds(Vec<UserId>),
}
