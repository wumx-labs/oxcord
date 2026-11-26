//! Key-Value state backend adapter.
//!
//! Enables running Discord state caching on top of any Key-Value store
//! (such as Redis, Dragonfly, KeyDB, RocksDB, or memory KV).

use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use oxcord_model::channel::{GuildChannel, Message};
use oxcord_model::guild::{Guild, Member, Role};
use oxcord_model::id::{ChannelId, GuildId, MessageId, RoleId, UserId};
use oxcord_model::user::{CurrentUser, User};

use super::{StateBackend, StateBackendError};
use crate::wrappers::BuildHasher;

/// Abstraction over a raw byte-oriented key-value store.
#[async_trait]
pub trait KeyValueStore: Send + Sync {
    /// Retrieves raw bytes for the specified key, if present.
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, StateBackendError>;

    /// Stores raw bytes under the specified key, with an optional TTL in seconds.
    async fn set_raw(
        &self,
        key: &str,
        value: &[u8],
        ttl_secs: Option<u64>,
    ) -> Result<(), StateBackendError>;

    /// Deletes a key and returns `true` if it existed.
    async fn del_raw(&self, key: &str) -> Result<bool, StateBackendError>;

    /// Lists all keys matching the specified prefix.
    async fn keys_with_prefix(&self, prefix: &str) -> Result<Vec<String>, StateBackendError>;

    /// Counts all keys matching the specified prefix.
    async fn count_prefix(&self, prefix: &str) -> Result<usize, StateBackendError> {
        let keys = self.keys_with_prefix(prefix).await?;
        Ok(keys.len())
    }

    /// Deletes all keys matching the specified prefix and returns the count deleted.
    async fn clear_prefix(&self, prefix: &str) -> Result<usize, StateBackendError> {
        let keys = self.keys_with_prefix(prefix).await?;
        let count = keys.len();
        for key in keys {
            self.del_raw(&key).await?;
        }
        Ok(count)
    }
}

/// An in-memory [`KeyValueStore`] implementation for testing and embedded storage.
#[derive(Debug, Default)]
pub struct MemoryKeyValueStore {
    data: DashMap<String, Vec<u8>, BuildHasher>,
}

impl MemoryKeyValueStore {
    /// Creates a new, empty in-memory key-value store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KeyValueStore for MemoryKeyValueStore {
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, StateBackendError> {
        Ok(self.data.get(key).map(|v| v.clone()))
    }

    async fn set_raw(
        &self,
        key: &str,
        value: &[u8],
        _ttl_secs: Option<u64>,
    ) -> Result<(), StateBackendError> {
        self.data.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn del_raw(&self, key: &str) -> Result<bool, StateBackendError> {
        Ok(self.data.remove(key).is_some())
    }

    async fn keys_with_prefix(&self, prefix: &str) -> Result<Vec<String>, StateBackendError> {
        Ok(self
            .data
            .iter()
            .filter_map(|entry| {
                if entry.key().starts_with(prefix) { Some(entry.key().clone()) } else { None }
            })
            .collect())
    }

    async fn count_prefix(&self, prefix: &str) -> Result<usize, StateBackendError> {
        Ok(self.data.iter().filter(|entry| entry.key().starts_with(prefix)).count())
    }
}

/// A [`StateBackend`] implementation that stores serialized JSON entities in an underlying
/// [`KeyValueStore`].
///
/// Keys are namespaced with the given prefix (defaulting to `"oxcord"`):
/// - `{prefix}:guild:{guild_id}`
/// - `{prefix}:channel:{channel_id}`
/// - `{prefix}:user:{user_id}`
/// - `{prefix}:member:{guild_id}:{user_id}`
/// - `{prefix}:role:{guild_id}:{role_id}`
/// - `{prefix}:msg:{channel_id}:{message_id}`
/// - `{prefix}:current_user`
#[derive(Debug, Clone)]
pub struct KeyValueStateBackend<KV> {
    store: Arc<KV>,
    prefix: String,
}

impl<KV: KeyValueStore> KeyValueStateBackend<KV> {
    /// Creates a new `KeyValueStateBackend` with default `"oxcord"` prefix.
    pub fn new(store: KV) -> Self {
        Self::with_prefix(store, "oxcord")
    }

    /// Creates a new `KeyValueStateBackend` with a custom namespace prefix.
    pub fn with_prefix(store: KV, prefix: impl Into<String>) -> Self {
        Self {
            store: Arc::new(store),
            prefix: prefix.into(),
        }
    }

    fn key_guild(&self, id: GuildId) -> String {
        format!("{}:guild:{}", self.prefix, id)
    }

    fn key_channel(&self, id: ChannelId) -> String {
        format!("{}:channel:{}", self.prefix, id)
    }

    fn key_user(&self, id: UserId) -> String {
        format!("{}:user:{}", self.prefix, id)
    }

    fn key_member(&self, guild_id: GuildId, user_id: UserId) -> String {
        format!("{}:member:{}:{}", self.prefix, guild_id, user_id)
    }

    fn key_role(&self, guild_id: GuildId, role_id: RoleId) -> String {
        format!("{}:role:{}:{}", self.prefix, guild_id, role_id)
    }

    fn key_msg(&self, channel_id: ChannelId, msg_id: MessageId) -> String {
        format!("{}:msg:{}:{}", self.prefix, channel_id, msg_id)
    }

    fn key_current_user(&self) -> String {
        format!("{}:current_user", self.prefix)
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let bytes = self.store.get_raw(key).await.ok()??;
        serde_json::from_slice(&bytes).ok()
    }

    async fn set_json<T: serde::Serialize>(&self, key: &str, val: &T, ttl: Option<u64>) {
        if let Ok(bytes) = serde_json::to_vec(val) {
            let _ = self.store.set_raw(key, &bytes, ttl).await;
        }
    }
}

#[async_trait]
impl<KV: KeyValueStore> StateBackend for KeyValueStateBackend<KV> {
    async fn get_guild(&self, guild_id: GuildId) -> Option<Guild> {
        self.get_json(&self.key_guild(guild_id)).await
    }

    async fn set_guild(&self, guild: Guild) {
        self.set_json(&self.key_guild(guild.id), &guild, None).await;
    }

    async fn remove_guild(&self, guild_id: GuildId) -> Option<Guild> {
        let key = self.key_guild(guild_id);
        let existing = self.get_json(&key).await;
        let _ = self.store.del_raw(&key).await;
        existing
    }

    async fn guild_count(&self) -> usize {
        let prefix = format!("{}:guild:", self.prefix);
        self.store.count_prefix(&prefix).await.unwrap_or(0)
    }

    async fn guild_ids(&self) -> Vec<GuildId> {
        let prefix = format!("{}:guild:", self.prefix);
        let keys = self.store.keys_with_prefix(&prefix).await.unwrap_or_default();
        keys.into_iter()
            .filter_map(|k| {
                let id_str = k.strip_prefix(&prefix)?;
                id_str.parse::<u64>().ok().map(GuildId::new)
            })
            .collect()
    }

    async fn get_channel(&self, channel_id: ChannelId) -> Option<GuildChannel> {
        self.get_json(&self.key_channel(channel_id)).await
    }

    async fn set_channel(&self, channel: GuildChannel) {
        self.set_json(&self.key_channel(channel.id), &channel, None).await;
    }

    async fn remove_channel(&self, channel_id: ChannelId) -> Option<GuildChannel> {
        let key = self.key_channel(channel_id);
        let existing = self.get_json(&key).await;
        let _ = self.store.del_raw(&key).await;
        existing
    }

    async fn channel_ids(&self) -> Vec<ChannelId> {
        let prefix = format!("{}:channel:", self.prefix);
        let keys = self.store.keys_with_prefix(&prefix).await.unwrap_or_default();
        keys.into_iter()
            .filter_map(|k| {
                let id_str = k.strip_prefix(&prefix)?;
                id_str.parse::<u64>().ok().map(ChannelId::new)
            })
            .collect()
    }

    async fn get_user(&self, user_id: UserId) -> Option<User> {
        self.get_json(&self.key_user(user_id)).await
    }

    async fn set_user(&self, user: User) {
        self.set_json(&self.key_user(user.id), &user, None).await;
    }

    async fn remove_user(&self, user_id: UserId) -> Option<User> {
        let key = self.key_user(user_id);
        let existing = self.get_json(&key).await;
        let _ = self.store.del_raw(&key).await;
        existing
    }

    async fn user_count(&self) -> usize {
        let prefix = format!("{}:user:", self.prefix);
        self.store.count_prefix(&prefix).await.unwrap_or(0)
    }

    async fn get_member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member> {
        self.get_json(&self.key_member(guild_id, user_id)).await
    }

    async fn set_member(&self, guild_id: GuildId, member: Member) {
        self.set_json(&self.key_member(guild_id, member.user.id), &member, None).await;
    }

    async fn remove_member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member> {
        let key = self.key_member(guild_id, user_id);
        let existing = self.get_json(&key).await;
        let _ = self.store.del_raw(&key).await;
        existing
    }

    async fn get_role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role> {
        self.get_json(&self.key_role(guild_id, role_id)).await
    }

    async fn set_role(&self, guild_id: GuildId, role: Role) {
        self.set_json(&self.key_role(guild_id, role.id), &role, None).await;
    }

    async fn remove_role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role> {
        let key = self.key_role(guild_id, role_id);
        let existing = self.get_json(&key).await;
        let _ = self.store.del_raw(&key).await;
        existing
    }

    async fn get_message(&self, channel_id: ChannelId, message_id: MessageId) -> Option<Message> {
        self.get_json(&self.key_msg(channel_id, message_id)).await
    }

    async fn set_message(&self, message: Message) {
        self.set_json(&self.key_msg(message.channel_id, message.id), &message, None).await;
    }

    async fn remove_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Option<Message> {
        let key = self.key_msg(channel_id, message_id);
        let existing = self.get_json(&key).await;
        let _ = self.store.del_raw(&key).await;
        existing
    }

    async fn get_current_user(&self) -> Option<CurrentUser> {
        self.get_json(&self.key_current_user()).await
    }

    async fn set_current_user(&self, user: CurrentUser) {
        self.set_json(&self.key_current_user(), &user, None).await;
    }

    async fn clear(&self) {
        let prefix = format!("{}:", self.prefix);
        let _ = self.store.clear_prefix(&prefix).await;
    }
}
