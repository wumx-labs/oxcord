use std::time::Duration;

/// Settings for the cache.
///
/// # Examples
///
/// Create new settings, specifying the maximum number of messages:
///
/// ```rust,ignore
/// use oxcord::cache::Settings as CacheSettings;
///
/// let mut settings = CacheSettings::default();
/// settings.max_messages = 10;
/// ```
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Settings {
    /// The maximum number of messages to store in a channel's message cache.
    ///
    /// Defaults to 0.
    pub max_messages: usize,
    /// How long temporarily-cached data should be stored before being thrown out.
    ///
    /// Defaults to one hour.
    pub time_to_live: Duration,
    /// Whether to cache guild data received from gateway.
    ///
    /// Defaults to true.
    pub cache_guilds: bool,
    /// Whether to cache channel data received from gateway.
    ///
    /// Defaults to true.
    pub cache_channels: bool,
    /// Whether to cache user data received from gateway.
    ///
    /// Defaults to true.
    pub cache_users: bool,
    /// Maximum number of users to retain in the user cache.
    ///
    /// When the user cache exceeds this limit, the sweeper will remove users that are
    /// not members of any cached guild. Set to `0` for unlimited.
    ///
    /// Defaults to 0 (unlimited).
    pub max_users: usize,
    /// How often the cache sweeper runs.
    ///
    /// When `Some`, a background task will periodically call [`Cache::sweep`] to trim
    /// stale or oversized entries. When `None`, automatic sweeping is disabled (you
    /// can still call [`Cache::sweep`] manually).
    ///
    /// Defaults to `None` (disabled).
    pub sweep_interval: Option<Duration>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_messages: 0,
            time_to_live: Duration::from_secs(60 * 60),
            cache_guilds: true,
            cache_channels: true,
            cache_users: true,
            max_users: 0,
            sweep_interval: None,
        }
    }
}
