//! Per-command cooldown management.
//!
//! # Scopes
//!
//! Cooldowns can be enforced at four granularities:
//!
//! | Scope | Description |
//! |---|---|
//! | `Global` | One shared bucket across every invocation |
//! | `User` | Per user, across all guilds |
//! | `Guild` | Per guild, shared among all members |
//! | `Channel` | Per channel |
//! | `Member` | Per `(guild_id, user_id)` pair |

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use oxcord_model::id::{ChannelId, GuildId, UserId};
use parking_lot::Mutex;

/// The granularity at which a cooldown is enforced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CooldownScope {
    /// One bucket shared by everyone.
    Global,
    /// Per user (across guilds).
    User,
    /// Per guild.
    Guild,
    /// Per channel.
    Channel,
    /// Per `(guild, user)` pair — the strictest scope.
    Member,
}

/// Cooldown configuration for a single command.
#[derive(Debug, Clone)]
pub struct CooldownConfig {
    /// The scope at which to enforce the cooldown.
    pub scope: CooldownScope,
    /// How long a user must wait between invocations.
    pub duration: Duration,
}

impl CooldownConfig {
    /// Create a new config.
    #[must_use]
    pub fn new(scope: CooldownScope, duration: Duration) -> Self {
        Self {
            scope,
            duration,
        }
    }

    /// Per-user, 5-second cooldown — the most common case.
    #[must_use]
    pub fn user_5s() -> Self {
        Self::new(CooldownScope::User, Duration::from_secs(5))
    }
}

/// Internal bucket key derived from scope + IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BucketKey {
    Global,
    User(UserId),
    Guild(GuildId),
    Channel(ChannelId),
    Member(GuildId, UserId),
}

/// Thread-safe cooldown manager, one instance per command.
#[derive(Clone, Debug)]
pub struct CooldownManager {
    config: CooldownConfig,
    buckets: Arc<Mutex<HashMap<BucketKey, Instant>>>,
}

impl CooldownManager {
    /// Create a new manager with the given config.
    #[must_use]
    pub fn new(config: CooldownConfig) -> Self {
        Self {
            config,
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check whether the caller is on cooldown.
    ///
    /// Returns `Some(remaining)` if the caller must wait, `None` if they may proceed.
    #[must_use]
    pub fn check(
        &self,
        user_id: UserId,
        guild_id: Option<GuildId>,
        channel_id: ChannelId,
    ) -> Option<Duration> {
        let key = self.bucket_key(user_id, guild_id, channel_id);
        let buckets = self.buckets.lock();
        if let Some(&last) = buckets.get(&key) {
            let elapsed = last.elapsed();
            if elapsed < self.config.duration {
                return self.config.duration.checked_sub(elapsed);
            }
        }
        None
    }

    /// Record a command invocation for the given IDs, resetting the bucket.
    pub fn record(&self, user_id: UserId, guild_id: Option<GuildId>, channel_id: ChannelId) {
        let key = self.bucket_key(user_id, guild_id, channel_id);
        let mut buckets = self.buckets.lock();
        buckets.insert(key, Instant::now());
    }

    fn bucket_key(
        &self,
        user_id: UserId,
        guild_id: Option<GuildId>,
        channel_id: ChannelId,
    ) -> BucketKey {
        match self.config.scope {
            CooldownScope::Global => BucketKey::Global,
            CooldownScope::User => BucketKey::User(user_id),
            CooldownScope::Guild => guild_id.map_or(BucketKey::User(user_id), BucketKey::Guild),
            CooldownScope::Channel => BucketKey::Channel(channel_id),
            CooldownScope::Member => {
                guild_id.map(|g| BucketKey::Member(g, user_id)).unwrap_or(BucketKey::User(user_id))
            },
        }
    }

    /// Prune expired buckets to keep memory bounded.
    ///
    /// Call this periodically (e.g., from a background task every minute).
    pub fn prune(&self) {
        let mut buckets = self.buckets.lock();
        buckets.retain(|_, last| last.elapsed() < self.config.duration);
    }
}

#[cfg(test)]
mod tests {
    use oxcord_model::id::{ChannelId, GuildId, UserId};

    use super::*;

    fn ids() -> (UserId, Option<GuildId>, ChannelId) {
        (UserId::new(1), Some(GuildId::new(2)), ChannelId::new(3))
    }

    #[test]
    fn no_cooldown_initially() {
        let mgr = CooldownManager::new(CooldownConfig::user_5s());
        let (user, guild, chan) = ids();
        assert!(mgr.check(user, guild, chan).is_none());
    }

    #[test]
    fn cooldown_after_record() {
        let mgr = CooldownManager::new(CooldownConfig::user_5s());
        let (user, guild, chan) = ids();
        mgr.record(user, guild, chan);
        assert!(mgr.check(user, guild, chan).is_some());
    }

    #[test]
    fn prune_clears_expired() {
        let config = CooldownConfig::new(CooldownScope::User, Duration::from_millis(1));
        let mgr = CooldownManager::new(config);
        let (user, guild, chan) = ids();
        mgr.record(user, guild, chan);
        std::thread::sleep(Duration::from_millis(5));
        mgr.prune();
        let buckets = mgr.buckets.lock();
        assert!(buckets.is_empty());
    }
}
