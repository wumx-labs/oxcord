//! High-level State Engine that processes Discord events and coordinates state queries.

use std::sync::Arc;

use oxcord_model::channel::{GuildChannel, Message};
use oxcord_model::event::Event;
use oxcord_model::guild::{Guild, Member, Role};
use oxcord_model::id::{ChannelId, GuildId, MessageId, RoleId, UserId};
use oxcord_model::user::{CurrentUser, User};

use super::{InMemoryBackend, StateBackend};

/// Snapshot statistics of state currently held in the backend.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateStats {
    /// Total number of cached guilds.
    pub guilds: usize,
    /// Total number of cached channels.
    pub channels: usize,
    /// Total number of cached users.
    pub users: usize,
}

/// The central state engine for managing Discord cached entities.
///
/// Designed to be decoupled from the Gateway, allowing events received over
/// Gateway, Webhooks, or message brokers (Kafka/RabbitMQ) to update state uniformly.
#[derive(Clone)]
pub struct StateEngine {
    backend: Arc<dyn StateBackend>,
}

impl Default for StateEngine {
    fn default() -> Self {
        Self::new(InMemoryBackend::new())
    }
}

impl StateEngine {
    /// Creates a new `StateEngine` with the specified [`StateBackend`].
    pub fn new<B: StateBackend + 'static>(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
        }
    }

    /// Creates a new `StateEngine` wrapped with an `Arc<dyn StateBackend>`.
    pub fn from_arc(backend: Arc<dyn StateBackend>) -> Self {
        Self {
            backend,
        }
    }

    /// Returns a reference to the underlying [`StateBackend`].
    #[must_use]
    pub fn backend(&self) -> &Arc<dyn StateBackend> {
        &self.backend
    }

    // --- State Queries ---

    /// Retrieves a guild by its ID.
    pub async fn get_guild(&self, guild_id: GuildId) -> Option<Guild> {
        self.backend.get_guild(guild_id).await
    }

    /// Retrieves a channel by its ID.
    pub async fn get_channel(&self, channel_id: ChannelId) -> Option<GuildChannel> {
        self.backend.get_channel(channel_id).await
    }

    /// Retrieves a user by their ID.
    pub async fn get_user(&self, user_id: UserId) -> Option<User> {
        self.backend.get_user(user_id).await
    }

    /// Retrieves a guild member by guild and user ID.
    pub async fn get_member(&self, guild_id: GuildId, user_id: UserId) -> Option<Member> {
        self.backend.get_member(guild_id, user_id).await
    }

    /// Retrieves a role by guild and role ID.
    pub async fn get_role(&self, guild_id: GuildId, role_id: RoleId) -> Option<Role> {
        self.backend.get_role(guild_id, role_id).await
    }

    /// Retrieves a message by channel and message ID.
    pub async fn get_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Option<Message> {
        self.backend.get_message(channel_id, message_id).await
    }

    /// Retrieves the current bot user.
    pub async fn get_current_user(&self) -> Option<CurrentUser> {
        self.backend.get_current_user().await
    }

    /// Checks if a user is a member of the given guild.
    pub async fn is_guild_member(&self, guild_id: GuildId, user_id: UserId) -> bool {
        self.backend.get_member(guild_id, user_id).await.is_some()
    }

    /// Gathers high-level entity counts from the backend.
    pub async fn stats(&self) -> StateStats {
        StateStats {
            guilds: self.backend.guild_count().await,
            channels: self.backend.channel_ids().await.len(),
            users: self.backend.user_count().await,
        }
    }

    // --- Event Ingestion ---

    /// Ingests a Discord [`Event`] and updates the backend accordingly.
    pub async fn handle_event(&self, event: &Event) {
        match event {
            Event::GuildCreate(e) => {
                let guild = &e.guild;
                self.backend.set_guild(guild.clone()).await;

                for channel in guild.channels.values() {
                    self.backend.set_channel(channel.clone()).await;
                }
                for role in guild.roles.values() {
                    self.backend.set_role(guild.id, role.clone()).await;
                }
                for member in guild.members.values() {
                    self.backend.set_member(guild.id, member.clone()).await;
                    self.backend.set_user(member.user.clone()).await;
                }
            },
            Event::GuildUpdate(e) => {
                if let Some(mut guild) = self.backend.get_guild(e.guild.id).await {
                    guild.afk_metadata.clone_from(&e.guild.afk_metadata);
                    guild.banner.clone_from(&e.guild.banner);
                    guild.description.clone_from(&e.guild.description);
                    guild.discovery_splash.clone_from(&e.guild.discovery_splash);
                    guild.emojis.clone_from(&e.guild.emojis);
                    guild.features.clone_from(&e.guild.features);
                    guild.icon.clone_from(&e.guild.icon);
                    guild.icon_hash.clone_from(&e.guild.icon_hash);
                    guild.name.clone_from(&e.guild.name);
                    guild.owner_id.clone_from(&e.guild.owner_id);
                    guild.preferred_locale.clone_from(&e.guild.preferred_locale);
                    guild.roles.clone_from(&e.guild.roles);
                    guild.splash.clone_from(&e.guild.splash);
                    guild.stickers.clone_from(&e.guild.stickers);
                    guild.vanity_url_code.clone_from(&e.guild.vanity_url_code);
                    guild.welcome_screen.clone_from(&e.guild.welcome_screen);
                    self.backend.set_guild(guild).await;
                }
            },
            Event::GuildDelete(e) => {
                self.backend.remove_guild(e.guild.id).await;
            },
            Event::ChannelCreate(e) => {
                self.backend.set_channel(e.channel.clone()).await;
            },
            Event::ChannelUpdate(e) => {
                self.backend.set_channel(e.channel.clone()).await;
            },
            Event::ChannelDelete(e) => {
                self.backend.remove_channel(e.channel.id).await;
            },
            Event::GuildRoleCreate(e) => {
                self.backend.set_role(e.role.guild_id, e.role.clone()).await;
            },
            Event::GuildRoleUpdate(e) => {
                self.backend.set_role(e.role.guild_id, e.role.clone()).await;
            },
            Event::GuildRoleDelete(e) => {
                self.backend.remove_role(e.guild_id, e.role_id).await;
            },
            Event::GuildMemberAdd(e) => {
                self.backend.set_member(e.member.guild_id, e.member.clone()).await;
                self.backend.set_user(e.member.user.clone()).await;
            },
            Event::GuildMemberUpdate(e) => {
                if let Some(mut member) = self.backend.get_member(e.guild_id, e.user.id).await {
                    member.user.clone_from(&e.user);
                    member.roles.clone_from(&e.roles);
                    member.nick.clone_from(&e.nick);
                    member.avatar.clone_from(&e.avatar);
                    member.banner.clone_from(&e.banner);
                    member.premium_since.clone_from(&e.premium_since);
                    member.pending.clone_from(&e.pending);
                    member.deaf.clone_from(&e.deaf);
                    member.mute.clone_from(&e.mute);
                    member.communication_disabled_until.clone_from(&e.communication_disabled_until);
                    member.unusual_dm_activity_until.clone_from(&e.unusual_dm_activity_until);
                    self.backend.set_member(e.guild_id, member).await;
                }
                self.backend.set_user(e.user.clone()).await;
            },
            Event::GuildMemberRemove(e) => {
                self.backend.remove_member(e.guild_id, e.user.id).await;
            },
            Event::MessageCreate(e) => {
                self.backend.set_message(e.message.clone()).await;
            },
            Event::MessageDelete(e) => {
                self.backend.remove_message(e.channel_id, e.message_id).await;
            },
            Event::Ready(e) => {
                self.backend.set_current_user(e.ready.user.clone()).await;
            },
            Event::UserUpdate(e) => {
                self.backend.set_current_user(e.current_user.clone()).await;
            },
            _ => {},
        }
    }
}
