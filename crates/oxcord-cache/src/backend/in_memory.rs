//! In-memory state backend implementation using concurrent hash maps.

use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use oxcord_model::channel::{GuildChannel, Message};
use oxcord_model::guild::{Guild, Member, Role};
use oxcord_model::id::{ChannelId, GuildId, MessageId, RoleId, UserId};
use oxcord_model::user::{CurrentUser, User};
use parking_lot::RwLock;

use super::StateBackend;
use crate::wrappers::BuildHasher;

/// A high-performance, concurrent, in-memory state backend.
///
/// Backed by [`DashMap`] for fine-grained sharded locking across threads,
/// suitable for standalone bots and shards running within a single process.
#[derive(Debug, Default)]
pub struct InMemoryBackend {
    guilds: DashMap<GuildId, Guild, BuildHasher>,
    channels: DashMap<ChannelId, GuildChannel, BuildHasher>,
    users: DashMap<UserId, User, BuildHasher>,
    members: DashMap<(GuildId, UserId), Member, BuildHasher>,
    roles: DashMap<(GuildId, RoleId), Role, BuildHasher>,
    messages: DashMap<(ChannelId, MessageId), Message, BuildHasher>,
    current_user: Arc<RwLock<Option<CurrentUser>>>,
}

impl InMemoryBackend {
    /// Creates a new, empty in-memory backend.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl StateBackend for InMemoryBackend {
    async fn get_guild(&self, guild_id: GuildId) -> Option<Guild> {
        self.guilds.get(&guild_id).map(|g| g.clone())
    }

    async fn set_guild(&self, guild: Guild) {
        self.guilds.insert(guild.id, guild);
    }

    async fn remove_guild(&self, guild_id: GuildId) -> Option<Guild> {
        self.guilds.remove(&guild_id).map(|(_, g)| g)
    }

    async fn guild_count(&self) -> usize {
        self.guilds.len()
    }

    async fn guild_ids(&self) -> Vec<GuildId> {
        self.guilds.iter().map(|entry| *entry.key()).collect()
    }

    async fn get_channel(&self, channel_id: ChannelId) -> Option<GuildChannel> {
        self.channels.get(&channel_id).map(|c| c.clone())
    }

    async fn set_channel(&self, channel: GuildChannel) {
        self.channels.insert(channel.id, channel);
    }

    async fn remove_channel(&self, channel_id: ChannelId) -> Option<GuildChannel> {
        self.channels.remove(&channel_id).map(|(_, c)| c)
    }

    async fn channel_ids(&self) -> Vec<ChannelId> {
        self.channels.iter().map(|entry| *entry.key()).collect()
    }

    async fn get_user(&self, user_id: UserId) -> Option<User> {
        self.users.get(&user_id).map(|u| u.clone())
    }

    async fn set_user(&self, user: User) {
        self.users.insert(user.id, user);
    }

    async fn remove_user(&self, user_id: UserId) -> Option<User> {
        self.users.remove(&user_id).map(|(_, u)| u)
    }

    async fn user_count(&self) -> usize {
        self.users.len()
    }

    async fn get_member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member> {
        self.members.get(&(guild_id, user_id)).map(|m| m.clone())
    }

    async fn set_member(&self, guild_id: GuildId, member: Member) {
        self.members.insert((guild_id, member.user.id), member);
    }

    async fn remove_member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member> {
        self.members.remove(&(guild_id, user_id)).map(|(_, m)| m)
    }

    async fn get_role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role> {
        self.roles.get(&(guild_id, role_id)).map(|r| r.clone())
    }

    async fn set_role(&self, guild_id: GuildId, role: Role) {
        self.roles.insert((guild_id, role.id), role);
    }

    async fn remove_role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role> {
        self.roles.remove(&(guild_id, role_id)).map(|(_, r)| r)
    }

    async fn get_message(&self, channel_id: ChannelId, message_id: MessageId) -> Option<Message> {
        self.messages.get(&(channel_id, message_id)).map(|m| m.clone())
    }

    async fn set_message(&self, message: Message) {
        self.messages.insert((message.channel_id, message.id), message);
    }

    async fn remove_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Option<Message> {
        self.messages.remove(&(channel_id, message_id)).map(|(_, m)| m)
    }

    async fn get_current_user(&self) -> Option<CurrentUser> {
        self.current_user.read().clone()
    }

    async fn set_current_user(&self, user: CurrentUser) {
        *self.current_user.write() = Some(user);
    }

    async fn clear(&self) {
        self.guilds.clear();
        self.channels.clear();
        self.users.clear();
        self.members.clear();
        self.roles.clear();
        self.messages.clear();
        *self.current_user.write() = None;
    }
}
