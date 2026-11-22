use std::fmt::Display;
#[cfg(all(feature = "cache", feature = "model"))]
use std::fmt::Write;

#[cfg(all(feature = "model", feature = "utils"))]
use crate::builder::{Builder, EditMessage};
#[cfg(feature = "model")]
use crate::builder::{CreateAllowedMentions, CreateMessage};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::{Cache, GuildRef};
#[cfg(feature = "collector")]
use crate::collector::{
    ComponentInteractionCollector,
    ModalInteractionCollector,
    ReactionCollector,
};
#[cfg(feature = "model")]
use crate::constants;
#[cfg(feature = "collector")]
use crate::gateway::ShardMessenger;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;
#[cfg(all(feature = "model", feature = "cache"))]
use crate::utils;
#[allow(async_fn_in_trait)]
pub trait MessageExt: Sized {
    #[cfg(feature = "model")]
    /// Crossposts this message.
    ///
    /// Requires either to be the message author or to have manage [Manage Messages] permissions on
    /// this channel.
    ///
    /// **Note**: Only available on news channels.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have the required permissions.
    ///
    /// Returns a [`ModelError::MessageAlreadyCrossposted`] if the message has already been
    /// crossposted.
    ///
    /// Returns a [`ModelError::CannotCrosspostMessage`] if the message cannot be crossposted.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn crosspost(&self, cache_http: impl CacheHttp) -> Result<Message>;
    #[cfg(feature = "model")]
    /// First attempts to find a [`Channel`] by its Id in the cache, upon failure requests it via
    /// the REST API.
    ///
    /// **Note**: If the `cache`-feature is enabled permissions will be checked and upon owning the
    /// required permissions the HTTP-request will be issued.
    ///
    /// # Errors
    ///
    /// Can return an error if the HTTP request fails.
    async fn channel(&self, cache_http: impl CacheHttp) -> Result<Channel>;
    #[cfg(feature = "model")]
    /// A util function for determining whether this message was sent by someone else, or the bot.
    #[cfg(feature = "cache")]
    #[deprecated(since = "0.0.1", note = "Check Message::author is equal to Cache::current_user")]
    fn is_own(&self, cache: impl AsRef<Cache>) -> bool;
    #[cfg(feature = "model")]
    /// Calculates the permissions of the message author in the current channel.
    ///
    /// This handles the [`Permissions::SEND_MESSAGES_IN_THREADS`] permission for threads, setting
    /// [`Permissions::SEND_MESSAGES`] accordingly if this message was sent in a thread.
    ///
    /// This may return `None` if:
    /// - The [`Cache`] does not have the current [`Guild`]
    /// - The [`Guild`] does not have the current channel cached (should never happen).
    /// - This message is not from [`MessageCreateEvent`] and the author's [`Member`] cannot be
    ///   found in [`Guild#structfield.members`].
    #[cfg(feature = "cache")]
    fn author_permissions(&self, cache: impl AsRef<Cache>) -> Option<Permissions>;
    #[cfg(feature = "model")]
    /// Deletes the message.
    ///
    /// **Note**: The logged in user must either be the author of the message or have the [Manage
    /// Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then returns a [`ModelError::InvalidPermissions`] if the
    /// current user does not have the required permissions.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes all of the [`Reaction`]s associated with the message.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then returns a [`ModelError::InvalidPermissions`] if the
    /// current user does not have the required permissions.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn delete_reactions(&self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes the given [`Reaction`] from the message.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current user did not perform
    /// the reaction.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user did not perform the reaction, or lacks
    /// permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn delete_reaction(
        &self,
        http: impl AsRef<Http>,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes all of the [`Reaction`]s of a given emoji associated with the message.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then returns a [`ModelError::InvalidPermissions`] if the
    /// current user does not have the required permissions.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn delete_reaction_emoji(
        &self,
        cache_http: impl CacheHttp,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits this message, replacing the original content with new content.
    ///
    /// Message editing preserves all unchanged message data, with some exceptions for embeds and
    /// attachments.
    ///
    /// **Note**: In most cases requires that the current user be the author of the message.
    ///
    /// Refer to the documentation for [`EditMessage`] for information regarding content
    /// restrictions and requirements.
    ///
    /// # Examples
    ///
    /// Edit a message with new content:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::builder::EditMessage;
    /// # use oxcord::model::channel::Message;
    /// # use oxcord::model::id::ChannelId;
    /// # use oxcord::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let mut message: Message = unimplemented!();
    /// // assuming a `message` has already been bound
    /// let builder = EditMessage::new().content("new content");
    /// message.edit(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidUser`] if the current user is not
    /// the author. Otherwise returns [`Error::Http`] if the user lacks permission, as well as if
    /// invalid data is given.
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the message contents are too long.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditMessage) -> Result<()>;
    #[cfg(feature = "model")]
    /// Returns message content, but with user and role mentions replaced with
    /// names and everyone/here mentions cancelled.
    #[cfg(feature = "cache")]
    fn content_safe(&self, cache: impl AsRef<Cache>) -> String;
    #[cfg(feature = "model")]
    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different maximum number of
    /// users. The maximum that may be retrieve at a time is `100`, if a greater number is provided
    /// then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain user. This is
    /// useful for pagination.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// **Note**: If the passed reaction_type is a custom guild emoji, it must contain the name.
    /// So, [`Emoji`] or [`EmojiIdentifier`] will always work, [`ReactionType`] only if
    /// [`ReactionType::Custom::name`] is Some, and **[`EmojiId`] will never work**.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    async fn reaction_users(
        &self,
        http: impl AsRef<Http>,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<User>>;
    #[cfg(feature = "model")]
    /// Returns the associated [`Guild`] for the message if one is in the cache.
    ///
    /// Returns [`None`] if the guild's Id could not be found via [`Self::guild_id`] or if the
    /// Guild itself is not cached.
    ///
    /// Requires the `cache` feature be enabled.
    #[cfg(feature = "cache")]
    fn guild<'a>(&self, cache: &'a Cache) -> Option<GuildRef<'a>>;
    #[cfg(feature = "model")]
    /// True if message was sent using direct messages.
    ///
    /// **Only use this for messages from the gateway (event handler)!** Not for returned Message
    /// objects from HTTP requests, like [`ChannelId::send_message`], because [`Self::guild_id`] is
    /// never set for those, which this method relies on.
    #[must_use]
    #[deprecated(
        since = "0.0.1",
        note = "Check if guild_id is None if the message is received from the gateway."
    )]
    fn is_private(&self) -> bool;
    #[cfg(feature = "model")]
    /// Retrieves a clone of the author's Member instance, if this message was sent in a guild.
    ///
    /// If the instance cannot be found in the cache, or the `cache` feature is disabled, a HTTP
    /// request is performed to retrieve it from Discord's API.
    ///
    /// # Errors
    ///
    /// [`ModelError::ItemMissing`] is returned if [`Self::guild_id`] is [`None`].
    async fn member(&self, cache_http: impl CacheHttp) -> Result<Member>;
    #[cfg(feature = "model")]
    /// Checks the length of a message to ensure that it is within Discord's maximum length limit.
    ///
    /// Returns [`None`] if the message is within the limit, otherwise returns [`Some`] with an
    /// inner value of how many unicode code points the message is over.
    #[must_use]
    fn overflow_length(content: &str) -> Option<usize>;
    #[cfg(feature = "model")]
    /// Pins this message to its channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have the required permissions.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn pin(&self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// React to the message with a custom [`Emoji`] or unicode character.
    ///
    /// **Note**: Requires the [Add Reactions] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have the required [permissions].
    ///
    /// [Add Reactions]: Permissions::ADD_REACTIONS
    /// [permissions]: crate::model::permissions
    async fn react(
        &self,
        cache_http: impl CacheHttp,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<Reaction>;
    #[cfg(feature = "model")]
    /// Uses Discord's inline reply to a user without pinging them.
    ///
    /// User mentions are generally around 20 or 21 characters long.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have the required permissions.
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message is over the above
    /// limit, containing the number of unicode code points over the limit.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    async fn reply(
        &self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Uses Discord's inline reply to a user with a ping.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have the required permissions.
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message is over the above
    /// limit, containing the number of unicode code points over the limit.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    async fn reply_ping(
        &self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Replies to the user, mentioning them prior to the content in the form of: `@<USER_ID>
    /// YOUR_CONTENT`.
    ///
    /// User mentions are generally around 20 or 21 characters long.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have the required permissions.
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message is over the above
    /// limit, containing the number of unicode code points over the limit.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    async fn reply_mention(
        &self,
        cache_http: impl CacheHttp,
        content: impl Display,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Checks whether the message mentions passed [`UserId`].
    fn mentions_user_id(&self, id: impl Into<UserId>) -> bool;
    #[cfg(feature = "model")]
    /// Checks whether the message mentions passed [`User`].
    #[must_use]
    fn mentions_user(&self, user: &User) -> bool;
    #[cfg(feature = "model")]
    /// Checks whether the message mentions the current user.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the `cache` feature is not enabled, or if the cache is
    /// otherwise unavailable.
    async fn mentions_me(&self, cache_http: impl CacheHttp) -> Result<bool>;
    #[cfg(feature = "model")]
    /// Unpins the message from its channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have the required permissions.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn unpin(&self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// Ends the [`Poll`] on this message, if there is one.
    ///
    /// # Errors
    ///
    /// See [`ChannelId::end_poll`] for more information.
    async fn end_poll(&self, http: impl AsRef<Http>) -> Result<Self>;
    #[cfg(feature = "model")]
    /// Tries to return author's nickname in the current channel's guild.
    ///
    /// Refer to [`User::nick_in()`] inside and [`None`] outside of a guild.
    async fn author_nick(&self, cache_http: impl CacheHttp) -> Option<String>;
    #[cfg(feature = "model")]
    /// Returns a link referencing this message. When clicked, users will jump to the message. The
    /// link will be valid for messages in either private channels or guilds.
    #[must_use]
    fn link(&self) -> String;
    #[cfg(feature = "model")]
    /// Same as [`Self::link`] but tries to find the [`GuildId`] if Discord does not provide it.
    ///
    /// [`guild_id`]: Self::guild_id
    #[allow(deprecated)]
    #[deprecated(
        since = "0.0.1",
        note = "Use Self::link if Message was recieved via an event, otherwise use MessageId::link to provide the guild_id yourself."
    )]
    async fn link_ensured(&self, cache_http: impl CacheHttp) -> String;
    #[cfg(feature = "model")]
    /// Returns a builder which can be awaited to obtain a reaction or stream of reactions on this
    /// message.
    #[cfg(feature = "collector")]
    fn await_reaction(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector;
    #[cfg(feature = "model")]
    /// Same as [`Self::await_reaction`].
    #[cfg(feature = "collector")]
    fn await_reactions(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector;
    #[cfg(feature = "model")]
    /// Returns a builder which can be awaited to obtain a single component interactions or a
    /// stream of component interactions on this message.
    #[cfg(feature = "collector")]
    fn await_component_interaction(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> ComponentInteractionCollector;
    #[cfg(feature = "model")]
    /// Same as [`Self::await_component_interaction`].
    #[cfg(feature = "collector")]
    fn await_component_interactions(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> ComponentInteractionCollector;
    #[cfg(feature = "model")]
    /// Returns a builder which can be awaited to obtain a model submit interaction or stream of
    /// modal submit interactions on this message.
    #[cfg(feature = "collector")]
    fn await_modal_interaction(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> ModalInteractionCollector;
    #[cfg(feature = "model")]
    /// Same as [`Self::await_modal_interaction`].
    #[cfg(feature = "collector")]
    fn await_modal_interactions(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> ModalInteractionCollector;
    #[cfg(feature = "model")]
    /// Retrieves the message channel's category ID if the channel has one.
    async fn category_id(&self, cache_http: impl CacheHttp) -> Option<ChannelId>;
}
impl MessageExt for Message {
    #[cfg(feature = "model")]
    async fn crosspost(&self, cache_http: impl CacheHttp) -> Result<Message> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.author.id != cache.current_user().id && self.guild_id.is_some() {
                    utils::user_has_perms_cache(
                        cache,
                        self.channel_id,
                        Permissions::MANAGE_MESSAGES,
                    )?;
                }
            }
        }
        if let Some(flags) = self.flags {
            if flags.contains(MessageFlags::CROSSPOSTED) {
                return Err(Error::Model(ModelError::MessageAlreadyCrossposted));
            } else if flags.contains(MessageFlags::IS_CROSSPOST)
                || self.kind != MessageType::Regular
            {
                return Err(Error::Model(ModelError::CannotCrosspostMessage));
            }
        }
        self.channel_id.crosspost(cache_http.http(), self.id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn channel(&self, cache_http: impl CacheHttp) -> Result<Channel> {
        self.channel_id.to_channel(cache_http).await
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn is_own(&self, cache: impl AsRef<Cache>) -> bool {
        self.author.id == cache.as_ref().current_user().id
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn author_permissions(&self, cache: impl AsRef<Cache>) -> Option<Permissions> {
        let Some(guild_id) = self.guild_id else {
            return Some(Permissions::dm_permissions());
        };
        let guild = cache.as_ref().guild(guild_id)?;
        let (channel, is_thread) = if let Some(channel) = guild.channels.get(&self.channel_id) {
            (channel, false)
        } else {
            let thread = guild.threads.iter().find(|th| th.id == self.channel_id)?;
            (thread, true)
        };
        let mut permissions = if let Some(member) = &self.member {
            guild.partial_member_permissions_in(channel, self.author.id, member)
        } else {
            guild.user_permissions_in(channel, guild.members.get(&self.author.id)?)
        };
        if is_thread {
            permissions.set(Permissions::SEND_MESSAGES, permissions.send_messages_in_threads());
        }
        Some(permissions)
    }
    #[cfg(feature = "model")]
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.author.id != cache.current_user().id {
                    utils::user_has_perms_cache(
                        cache,
                        self.channel_id,
                        Permissions::MANAGE_MESSAGES,
                    )?;
                }
            }
        }
        self.channel_id.delete_message(cache_http.http(), self.id).await
    }
    #[cfg(feature = "model")]
    async fn delete_reactions(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                utils::user_has_perms_cache(cache, self.channel_id, Permissions::MANAGE_MESSAGES)?;
            }
        }
        self.channel_id.delete_reactions(cache_http.http(), self.id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete_reaction(
        &self,
        http: impl AsRef<Http>,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        self.channel_id.delete_reaction(http, self.id, user_id, reaction_type).await
    }
    #[cfg(feature = "model")]
    async fn delete_reaction_emoji(
        &self,
        cache_http: impl CacheHttp,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                utils::user_has_perms_cache(cache, self.channel_id, Permissions::MANAGE_MESSAGES)?;
            }
        }
        cache_http
            .http()
            .as_ref()
            .delete_message_reaction_emoji(self.channel_id, self.id, &reaction_type.into())
            .await
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditMessage) -> Result<()> {
        if let Some(flags) = self.flags {
            if flags.contains(MessageFlags::IS_VOICE_MESSAGE) {
                return Err(Error::Model(ModelError::CannotEditVoiceMessage));
            }
        }
        *self =
            builder.execute(cache_http, (self.channel_id, self.id, Some(self.author.id))).await?;
        Ok(())
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn content_safe(&self, cache: impl AsRef<Cache>) -> String {
        let mut result = self.content.clone();
        for u in &self.mentions {
            let mut at_distinct = String::with_capacity(38);
            at_distinct.push('@');
            at_distinct.push_str(&u.name);
            if let Some(discriminator) = u.discriminator {
                at_distinct.push('#');
                write!(at_distinct, "{:04}", discriminator.get())
                    .expect("writing to a string should never fail");
            }
            let mut m = u.mention().to_string();
            if !result.contains(&m) {
                m.insert(2, '!');
            }
            result = result.replace(&m, &at_distinct);
        }
        if let Some(guild_id) = self.guild_id {
            for id in &self.mention_roles {
                let mention = id.mention().to_string();
                if let Some(guild) = cache.as_ref().guild(guild_id) {
                    if let Some(role) = guild.roles.get(id) {
                        result = result.replace(&mention, &format!("@{}", role.name));
                        continue;
                    }
                }
                result = result.replace(&mention, "@deleted-role");
            }
        }
        result.replace("@everyone", "@\u{200B}everyone").replace("@here", "@\u{200B}here")
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn reaction_users(
        &self,
        http: impl AsRef<Http>,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<User>> {
        self.channel_id.reaction_users(http, self.id, reaction_type, limit, after).await
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn guild<'a>(&self, cache: &'a Cache) -> Option<GuildRef<'a>> {
        cache.guild(self.guild_id?)
    }
    #[cfg(feature = "model")]
    #[inline]
    fn is_private(&self) -> bool {
        self.guild_id.is_none()
    }
    #[cfg(feature = "model")]
    async fn member(&self, cache_http: impl CacheHttp) -> Result<Member> {
        match self.guild_id {
            Some(guild_id) => guild_id.member(cache_http, self.author.id).await,
            None => Err(Error::Model(ModelError::ItemMissing)),
        }
    }
    #[cfg(feature = "model")]
    fn overflow_length(content: &str) -> Option<usize> {
        crate::builder::check_overflow(content.chars().count(), constants::MESSAGE_CODE_LIMIT).err()
    }
    #[cfg(feature = "model")]
    async fn pin(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.guild_id.is_some() {
                    utils::user_has_perms_cache(
                        cache,
                        self.channel_id,
                        Permissions::MANAGE_MESSAGES,
                    )?;
                }
            }
        }
        self.channel_id.pin(cache_http.http(), self.id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn react(
        &self,
        cache_http: impl CacheHttp,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<Reaction> {
        self.react_(cache_http, reaction_type.into()).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn reply(
        &self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
    ) -> Result<Message> {
        self.reply_(cache_http, content, Some(false)).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn reply_ping(
        &self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
    ) -> Result<Message> {
        self.reply_(cache_http, content, Some(true)).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn reply_mention(
        &self,
        cache_http: impl CacheHttp,
        content: impl Display,
    ) -> Result<Message> {
        self.reply_(cache_http, format!("{} {content}", self.author.mention()), None).await
    }
    #[cfg(feature = "model")]
    #[inline]
    fn mentions_user_id(&self, id: impl Into<UserId>) -> bool {
        let id = id.into();
        self.mentions.iter().any(|mentioned_user| mentioned_user.id == id)
    }
    #[cfg(feature = "model")]
    #[inline]
    fn mentions_user(&self, user: &User) -> bool {
        self.mentions_user_id(user.id)
    }
    #[cfg(feature = "model")]
    async fn mentions_me(&self, cache_http: impl CacheHttp) -> Result<bool> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                return Ok(self.mentions_user_id(cache.current_user().id));
            }
        }
        let current_user = cache_http.http().get_current_user().await?;
        Ok(self.mentions_user_id(current_user.id))
    }
    #[cfg(feature = "model")]
    async fn unpin(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.guild_id.is_some() {
                    utils::user_has_perms_cache(
                        cache,
                        self.channel_id,
                        Permissions::MANAGE_MESSAGES,
                    )?;
                }
            }
        }
        cache_http.http().unpin_message(self.channel_id, self.id, None).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn end_poll(&self, http: impl AsRef<Http>) -> Result<Self> {
        self.channel_id.end_poll(http, self.id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn author_nick(&self, cache_http: impl CacheHttp) -> Option<String> {
        self.author.nick_in(cache_http, self.guild_id?).await
    }
    #[cfg(feature = "model")]
    #[inline]
    fn link(&self) -> String {
        self.id.link(self.channel_id, self.guild_id)
    }
    #[cfg(feature = "model")]
    #[inline]
    #[allow(deprecated)]
    async fn link_ensured(&self, cache_http: impl CacheHttp) -> String {
        self.id.link_ensured(cache_http, self.channel_id, self.guild_id).await
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reaction(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).message_id(self.id)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reactions(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector {
        self.await_reaction(shard_messenger)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_component_interaction(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> ComponentInteractionCollector {
        ComponentInteractionCollector::new(shard_messenger).message_id(self.id)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_component_interactions(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> ComponentInteractionCollector {
        self.await_component_interaction(shard_messenger)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_modal_interaction(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> ModalInteractionCollector {
        ModalInteractionCollector::new(shard_messenger).message_id(self.id)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_modal_interactions(
        &self,
        shard_messenger: impl AsRef<ShardMessenger>,
    ) -> ModalInteractionCollector {
        self.await_modal_interaction(shard_messenger)
    }
    #[cfg(feature = "model")]
    async fn category_id(&self, cache_http: impl CacheHttp) -> Option<ChannelId> {
        #[cfg(feature = "cache")]
        if let Some(cache) = cache_http.cache() {
            if let Some(guild) = cache.guild(self.guild_id?) {
                let channel = guild.channels.get(&self.channel_id)?;
                return if channel.thread_metadata.is_some() {
                    let thread_parent = guild.channels.get(&channel.parent_id?)?;
                    thread_parent.parent_id
                } else {
                    channel.parent_id
                };
            }
        }
        let channel = self.channel_id.to_channel(&cache_http).await.ok()?.guild()?;
        if channel.thread_metadata.is_some() {
            let thread_parent = channel.parent_id?.to_channel(cache_http).await.ok()?.guild()?;
            thread_parent.parent_id
        } else {
            channel.parent_id
        }
    }
}
#[allow(async_fn_in_trait)]
pub trait MessageIdExt: Sized {
    #[cfg(feature = "model")]
    /// Returns a link referencing this message. When clicked, users will jump to the message. The
    /// link will be valid for messages in either private channels or guilds.
    #[must_use]
    fn link(&self, channel_id: ChannelId, guild_id: Option<GuildId>) -> String;
    #[cfg(feature = "model")]
    /// Same as [`Self::link`] but tries to find the [`GuildId`] if it is not provided.
    #[deprecated(since = "0.0.1", note = "Use GuildChannel::guild_id if you have no GuildId")]
    async fn link_ensured(
        &self,
        cache_http: impl CacheHttp,
        channel_id: ChannelId,
        guild_id: Option<GuildId>,
    ) -> String;
}
impl MessageIdExt for MessageId {
    #[cfg(feature = "model")]
    fn link(&self, channel_id: ChannelId, guild_id: Option<GuildId>) -> String {
        if let Some(guild_id) = guild_id {
            format!("https://discord.com/channels/{guild_id}/{channel_id}/{self}")
        } else {
            format!("https://discord.com/channels/@me/{channel_id}/{self}")
        }
    }
    #[cfg(feature = "model")]
    async fn link_ensured(
        &self,
        cache_http: impl CacheHttp,
        channel_id: ChannelId,
        mut guild_id: Option<GuildId>,
    ) -> String {
        if guild_id.is_none() {
            let found_channel = channel_id.to_channel(cache_http).await;
            if let Ok(channel) = found_channel {
                if let Some(c) = channel.guild() {
                    guild_id = Some(c.guild_id);
                }
            }
        }
        self.link(channel_id, guild_id)
    }
}

#[cfg(feature = "model")]
#[allow(async_fn_in_trait)]
pub(crate) trait MessageHelpers {
    async fn react_(
        &self,
        cache_http: impl CacheHttp,
        reaction_type: ReactionType,
    ) -> Result<Reaction>;

    /// `inlined` decides whether this reply is inlined and whether it pings.
    async fn reply_(
        &self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
        inlined: Option<bool>,
    ) -> Result<Message>;
}

#[cfg(feature = "model")]
impl MessageHelpers for Message {
    async fn react_(
        &self,
        cache_http: impl CacheHttp,
        reaction_type: ReactionType,
    ) -> Result<Reaction> {
        #[cfg_attr(not(feature = "cache"), allow(unused_mut))]
        let mut user_id = None;

        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.guild_id.is_some() {
                    utils::user_has_perms_cache(
                        cache,
                        self.channel_id,
                        Permissions::ADD_REACTIONS,
                    )?;
                }

                user_id = Some(cache.current_user().id);
            }
        }

        cache_http.http().create_reaction(self.channel_id, self.id, &reaction_type).await?;

        Ok(Reaction {
            channel_id: self.channel_id,
            emoji: reaction_type,
            message_id: self.id,
            user_id,
            guild_id: self.guild_id,
            member: self.member.as_deref().map(|member| member.clone().into()),
            message_author_id: None,
            burst: false,
            burst_colours: None,
            reaction_type: ReactionTypes::Normal,
        })
    }

    /// `inlined` decides whether this reply is inlined and whether it pings.
    async fn reply_(
        &self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
        inlined: Option<bool>,
    ) -> Result<Message> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.guild_id.is_some() {
                    utils::user_has_perms_cache(
                        cache,
                        self.channel_id,
                        Permissions::SEND_MESSAGES,
                    )?;
                }
            }
        }

        let mut builder = CreateMessage::new().content(content);
        if let Some(ping_user) = inlined {
            let allowed_mentions = CreateAllowedMentions::new()
                .replied_user(ping_user)
                // By providing allowed_mentions, Discord disabled _all_ pings by default so we
                // need to re-enable them
                .everyone(true)
                .all_users(true)
                .all_roles(true);
            builder = builder.reference_message(self).allowed_mentions(allowed_mentions);
        }
        self.channel_id.send_message(cache_http, builder).await
    }
}

// all tests here require cache, move if non-cache test is added
#[cfg(all(test, feature = "cache"))]
mod tests {
    use std::collections::HashMap;

    use dashmap::DashMap;

    use crate::cache::{Cache, MaybeMap};
    use crate::model::prelude::{
        Guild,
        GuildChannel,
        Member,
        Message,
        PermissionOverwrite,
        PermissionOverwriteType,
        Permissions,
        User,
        UserId,
    };
    use crate::ops::*;

    /// Test that author_permissions checks the permissions in a channel, not just the guild.
    #[test]
    fn author_permissions_respects_overwrites() {
        // Author of the message, with a random ID that won't collide with defaults.
        let author = User {
            id: UserId::new(50778944701071),
            ..Default::default()
        };

        // Channel with the message, with SEND_MESSAGES on.
        let channel = GuildChannel {
            permission_overwrites: vec![PermissionOverwrite {
                allow: Permissions::SEND_MESSAGES,
                deny: Permissions::default(),
                kind: PermissionOverwriteType::Member(author.id),
            }],
            ..Default::default()
        };
        let channel_id = channel.id;

        // Guild with the author and channel cached, default (empty) permissions.
        let guild = Guild {
            channels: HashMap::from([(channel.id, channel)]),
            members: HashMap::from([(author.id, Member {
                user: author.clone(),
                ..Default::default()
            })]),
            ..Default::default()
        };

        // Message, tied to the guild and the channel.
        let message = Message {
            author,
            channel_id,
            guild_id: Some(guild.id),
            ..Default::default()
        };

        // Cache, with the guild setup.
        let mut cache = Cache::new();
        cache.guilds = MaybeMap(Some({
            let guilds = DashMap::default();
            guilds.insert(guild.id, guild);
            guilds
        }));

        // The author should only have the one permission, SEND_MESSAGES.
        assert_eq!(message.author_permissions(&cache), Some(Permissions::SEND_MESSAGES));
    }
}
