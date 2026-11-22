//! Pluggable State & Cache Backend Engine for Oxcord.
//!
//! Provides a backend-agnostic abstraction for storing, updating, and querying
//! Discord state. Supports both low-latency concurrent in-memory storage ([`InMemoryBackend`])
//! and distributed key-value engines ([`KeyValueStateBackend`]) for multi-sharded bots.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use oxcord_model::channel::{GuildChannel, Message};
use oxcord_model::guild::{Guild, Member, Role};
use oxcord_model::id::{ChannelId, GuildId, MessageId, RoleId, UserId};
use oxcord_model::user::{CurrentUser, User};

pub mod engine;
pub mod in_memory;
pub mod key_value;
#[cfg(feature = "redis")]
pub mod redis;

pub use self::engine::{StateEngine, StateStats};
pub use self::in_memory::InMemoryBackend;
pub use self::key_value::{KeyValueStateBackend, KeyValueStore, MemoryKeyValueStore};
#[cfg(feature = "redis")]
pub use self::redis::{RedisStateBackend, RedisStore};

/// Errors produced by state storage backends.
#[derive(Debug)]
pub enum StateBackendError {
    /// Serialization or deserialization error when reading/writing state entities.
    Serialization(serde_json::Error),
    /// Underlying storage or network driver error.
    Storage(String),
    /// Requested entity was not found.
    NotFound,
}

impl fmt::Display for StateBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(e) => write!(f, "state serialization error: {e}"),
            Self::Storage(msg) => write!(f, "state storage error: {msg}"),
            Self::NotFound => write!(f, "state entity not found"),
        }
    }
}

impl std::error::Error for StateBackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for StateBackendError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}

/// A pluggable asynchronous interface for Discord state storage.
///
/// Implementations of this trait can be backed by in-memory concurrent hash maps
/// ([`InMemoryBackend`]), external key-value databases like Redis/Dragonfly
/// ([`KeyValueStateBackend`]), or any SQL database.
///
/// # Implementing a Custom Backend
///
/// ```rust,no_run
/// # use async_trait::async_trait;
/// # use oxcord_cache::StateBackend;
/// # use oxcord_model::guild::Guild;
/// # use oxcord_model::id::GuildId;
/// // (other required imports omitted for brevity)
/// struct MyBackend;
///
/// // Implement all required async methods on `MyBackend`...
/// ```
///
/// # Choosing a Backend
///
/// | Backend                  | Use case                                    |
/// |--------------------------|---------------------------------------------|
/// | [`InMemoryBackend`]      | Single-process bots, lowest latency         |
/// | [`KeyValueStateBackend`] | Multi-shard / distributed bots, e.g. Redis  |
///
/// # Thread Safety
///
/// All implementations **must** be `Send + Sync` and safe to share across
/// asynchronous tasks without external locking.
#[async_trait]
pub trait StateBackend: Send + Sync {
    // --- Guilds ---
    /// Retrieves a guild by its ID.
    async fn get_guild(&self, guild_id: GuildId) -> Option<Guild>;
    /// Inserts or updates a guild.
    async fn set_guild(&self, guild: Guild);
    /// Removes a guild and returns the previous value if it existed.
    async fn remove_guild(&self, guild_id: GuildId) -> Option<Guild>;
    /// Returns the total number of cached guilds.
    async fn guild_count(&self) -> usize;
    /// Returns a list of all cached guild IDs.
    async fn guild_ids(&self) -> Vec<GuildId>;

    // --- Channels ---
    /// Retrieves a guild channel by its ID.
    async fn get_channel(&self, channel_id: ChannelId) -> Option<GuildChannel>;
    /// Inserts or updates a guild channel.
    async fn set_channel(&self, channel: GuildChannel);
    /// Removes a guild channel and returns the previous value if it existed.
    async fn remove_channel(&self, channel_id: ChannelId) -> Option<GuildChannel>;
    /// Returns a list of all cached channel IDs.
    async fn channel_ids(&self) -> Vec<ChannelId>;

    // --- Users ---
    /// Retrieves a user by their ID.
    async fn get_user(&self, user_id: UserId) -> Option<User>;
    /// Inserts or updates a user.
    async fn set_user(&self, user: User);
    /// Removes a user and returns the previous value if it existed.
    async fn remove_user(&self, user_id: UserId) -> Option<User>;
    /// Returns the total number of cached users.
    async fn user_count(&self) -> usize;

    // --- Members ---
    /// Retrieves a member of a guild by user ID.
    async fn get_member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member>;
    /// Inserts or updates a guild member.
    async fn set_member(&self, guild_id: GuildId, member: Member);
    /// Removes a guild member and returns the previous value if it existed.
    async fn remove_member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member>;

    // --- Roles ---
    /// Retrieves a role by guild ID and role ID.
    async fn get_role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role>;
    /// Inserts or updates a guild role.
    async fn set_role(&self, guild_id: GuildId, role: Role);
    /// Removes a guild role and returns the previous value if it existed.
    async fn remove_role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role>;

    // --- Messages ---
    /// Retrieves a message by channel ID and message ID.
    async fn get_message(&self, channel_id: ChannelId, message_id: MessageId) -> Option<Message>;
    /// Inserts or updates a message in a channel.
    async fn set_message(&self, message: Message);
    /// Removes a message and returns the previous value if it existed.
    async fn remove_message(&self, channel_id: ChannelId, message_id: MessageId)
    -> Option<Message>;

    // --- Current Bot User ---
    /// Retrieves the current logged in bot user.
    async fn get_current_user(&self) -> Option<CurrentUser>;
    /// Sets the current logged in bot user.
    async fn set_current_user(&self, user: CurrentUser);

    // --- Lifecycle ---
    /// Clears all cached state across all entities.
    async fn clear(&self);
}

#[async_trait]
impl<T: StateBackend + ?Sized> StateBackend for Arc<T> {
    async fn get_guild(&self, guild_id: GuildId) -> Option<Guild> {
        (**self).get_guild(guild_id).await
    }
    async fn set_guild(&self, guild: Guild) {
        (**self).set_guild(guild).await;
    }
    async fn remove_guild(&self, guild_id: GuildId) -> Option<Guild> {
        (**self).remove_guild(guild_id).await
    }
    async fn guild_count(&self) -> usize {
        (**self).guild_count().await
    }
    async fn guild_ids(&self) -> Vec<GuildId> {
        (**self).guild_ids().await
    }

    async fn get_channel(&self, channel_id: ChannelId) -> Option<GuildChannel> {
        (**self).get_channel(channel_id).await
    }
    async fn set_channel(&self, channel: GuildChannel) {
        (**self).set_channel(channel).await;
    }
    async fn remove_channel(&self, channel_id: ChannelId) -> Option<GuildChannel> {
        (**self).remove_channel(channel_id).await
    }
    async fn channel_ids(&self) -> Vec<ChannelId> {
        (**self).channel_ids().await
    }

    async fn get_user(&self, user_id: UserId) -> Option<User> {
        (**self).get_user(user_id).await
    }
    async fn set_user(&self, user: User) {
        (**self).set_user(user).await;
    }
    async fn remove_user(&self, user_id: UserId) -> Option<User> {
        (**self).remove_user(user_id).await
    }
    async fn user_count(&self) -> usize {
        (**self).user_count().await
    }

    async fn get_member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member> {
        (**self).get_member(guild_id, user_id).await
    }
    async fn set_member(&self, guild_id: GuildId, member: Member) {
        (**self).set_member(guild_id, member).await;
    }
    async fn remove_member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member> {
        (**self).remove_member(guild_id, user_id).await
    }

    async fn get_role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role> {
        (**self).get_role(guild_id, role_id).await
    }
    async fn set_role(&self, guild_id: GuildId, role: Role) {
        (**self).set_role(guild_id, role).await;
    }
    async fn remove_role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role> {
        (**self).remove_role(guild_id, role_id).await
    }

    async fn get_message(&self, channel_id: ChannelId, message_id: MessageId) -> Option<Message> {
        (**self).get_message(channel_id, message_id).await
    }
    async fn set_message(&self, message: Message) {
        (**self).set_message(message).await;
    }
    async fn remove_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Option<Message> {
        (**self).remove_message(channel_id, message_id).await
    }

    async fn get_current_user(&self) -> Option<CurrentUser> {
        (**self).get_current_user().await
    }
    async fn set_current_user(&self, user: CurrentUser) {
        (**self).set_current_user(user).await;
    }

    async fn clear(&self) {
        (**self).clear().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_user(id: u64, name: &str) -> User {
        serde_json::from_value(serde_json::json!({
            "id": id.to_string(),
            "username": name,
            "discriminator": "0001",
            "bot": false,
        }))
        .unwrap()
    }

    fn sample_guild(id: u64, name: &str) -> Guild {
        serde_json::from_value(serde_json::json!({
            "id": id.to_string(),
            "name": name,
            "owner_id": "1",
            "roles": [],
            "emojis": [],
            "stickers": [],
            "features": [],
            "channels": [],
            "threads": [],
            "presences": [],
            "stage_instances": [],
            "guild_scheduled_events": [],
            "joined_at": "2026-01-01T00:00:00.000Z",
            "large": false,
            "member_count": 1,
            "voice_states": [],
            "members": [],
            "afk_timeout": 300,
            "verification_level": 0,
            "default_message_notifications": 0,
            "explicit_content_filter": 0,
            "mfa_level": 0,
            "nsfw_level": 0,
            "premium_tier": 0,
            "premium_progress_bar_enabled": false,
            "preferred_locale": "en-US",
            "system_channel_flags": 0,
        }))
        .unwrap()
    }

    fn sample_channel(id: u64, guild_id: u64, name: &str) -> GuildChannel {
        serde_json::from_value(serde_json::json!({
            "id": id.to_string(),
            "guild_id": guild_id.to_string(),
            "name": name,
            "type": 0,
            "position": 0,
            "permission_overwrites": [],
        }))
        .unwrap()
    }

    fn sample_message(id: u64, channel_id: u64, content: &str) -> Message {
        serde_json::from_value(serde_json::json!({
            "id": id.to_string(),
            "channel_id": channel_id.to_string(),
            "author": {
                "id": "100",
                "username": "tester",
                "discriminator": "0001",
                "bot": false,
            },
            "content": content,
            "timestamp": "2026-01-01T00:00:00.000Z",
            "edited_timestamp": null,
            "tts": false,
            "mention_everyone": false,
            "mentions": [],
            "mention_roles": [],
            "attachments": [],
            "embeds": [],
            "pinned": false,
            "type": 0,
        }))
        .unwrap()
    }

    fn sample_role(id: u64, guild_id: u64, name: &str) -> Role {
        serde_json::from_value(serde_json::json!({
            "id": id.to_string(),
            "guild_id": guild_id.to_string(),
            "name": name,
            "color": 0xFF0000,
            "colors": {
                "primary_color": 0xFF0000
            },
            "flags": 0,
            "hoist": true,
            "position": 1,
            "permissions": "8",
            "managed": false,
            "mentionable": true,
        }))
        .unwrap()
    }

    fn sample_member(user: User, guild_id: u64) -> Member {
        let mut m: Member = serde_json::from_value(serde_json::json!({
            "user": user,
            "roles": [],
            "joined_at": "2026-01-01T00:00:00.000Z",
            "deaf": false,
            "mute": false,
            "flags": 0,
        }))
        .unwrap();
        m.guild_id = GuildId::new(guild_id);
        m
    }

    async fn verify_state_backend<B: StateBackend>(backend: &B) {
        let guild_id = GuildId::new(1001);
        let channel_id = ChannelId::new(2001);
        let user_id = UserId::new(3001);
        let role_id = RoleId::new(4001);
        let msg_id = MessageId::new(5001);

        let user = sample_user(3001, "Alice");
        let guild = sample_guild(1001, "Rust Guild");
        let channel = sample_channel(2001, 1001, "general");
        let role = sample_role(4001, 1001, "Admin");
        let member = sample_member(user.clone(), 1001);
        let message = sample_message(5001, 2001, "Hello Oxcord!");

        // Initial empty state
        assert_eq!(backend.guild_count().await, 0);
        assert_eq!(backend.user_count().await, 0);
        assert!(backend.get_guild(guild_id).await.is_none());

        // Insert entities
        backend.set_guild(guild).await;
        backend.set_channel(channel).await;
        backend.set_user(user).await;
        backend.set_role(guild_id, role).await;
        backend.set_member(guild_id, member).await;
        backend.set_message(message).await;

        // Verify counts
        assert_eq!(backend.guild_count().await, 1);
        assert_eq!(backend.user_count().await, 1);
        assert_eq!(backend.guild_ids().await, vec![guild_id]);
        assert_eq!(backend.channel_ids().await, vec![channel_id]);

        // Verify retrieval
        let fetched_guild = backend.get_guild(guild_id).await.unwrap();
        assert_eq!(fetched_guild.name, "Rust Guild");

        let fetched_chan = backend.get_channel(channel_id).await.unwrap();
        assert_eq!(fetched_chan.name, "general");

        let fetched_user = backend.get_user(user_id).await.unwrap();
        assert_eq!(fetched_user.name, "Alice");

        let fetched_role = backend.get_role(guild_id, role_id).await.unwrap();
        assert_eq!(fetched_role.name, "Admin");

        let fetched_member = backend.get_member(guild_id, user_id).await.unwrap();
        assert_eq!(fetched_member.user.name, "Alice");

        let fetched_msg = backend.get_message(channel_id, msg_id).await.unwrap();
        assert_eq!(fetched_msg.content, "Hello Oxcord!");

        // Remove operations
        let removed_msg = backend.remove_message(channel_id, msg_id).await;
        assert!(removed_msg.is_some());
        assert!(backend.get_message(channel_id, msg_id).await.is_none());

        let removed_member = backend.remove_member(guild_id, user_id).await;
        assert!(removed_member.is_some());
        assert!(backend.get_member(guild_id, user_id).await.is_none());

        // Clear all
        backend.clear().await;
        assert_eq!(backend.guild_count().await, 0);
        assert_eq!(backend.user_count().await, 0);
        assert!(backend.get_guild(guild_id).await.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_backend() {
        let backend = InMemoryBackend::new();
        verify_state_backend(&backend).await;
    }

    #[tokio::test]
    async fn test_key_value_state_backend() {
        let kv = MemoryKeyValueStore::new();
        let backend = KeyValueStateBackend::new(kv);
        verify_state_backend(&backend).await;
    }

    #[tokio::test]
    async fn test_state_engine_lifecycle_and_queries() {
        let backend = InMemoryBackend::new();
        let engine = StateEngine::new(backend);

        let guild_id = GuildId::new(999);
        let user_id = UserId::new(888);

        let user = sample_user(888, "Bob");
        let guild = sample_guild(999, "Engine Guild");
        let member = sample_member(user.clone(), 999);

        engine.backend().set_guild(guild).await;
        engine.backend().set_user(user).await;
        engine.backend().set_member(guild_id, member).await;

        assert!(engine.is_guild_member(guild_id, user_id).await);
        assert!(!engine.is_guild_member(guild_id, UserId::new(12345)).await);

        let stats = engine.stats().await;
        assert_eq!(stats.guilds, 1);
        assert_eq!(stats.users, 1);
    }
}
