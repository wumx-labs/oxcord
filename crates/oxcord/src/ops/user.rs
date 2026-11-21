#[cfg(feature = "model")]
use crate::builder::{Builder, CreateMessage, EditProfile};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::{Cache, UserRef};
#[cfg(feature = "collector")]
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "collector")]
use crate::gateway::ShardMessenger;
#[cfg(feature = "model")]
use crate::http::CacheHttp;
#[cfg(feature = "model")]
use crate::json::json;
use crate::model::prelude::*;
#[cfg(feature = "model")]
use crate::model::user::avatar_decoration_url;
#[cfg(feature = "model")]
use crate::model::user::default_avatar_url;
#[cfg(all(feature = "unstable_discord_api", feature = "model"))]
use crate::model::user::nameplate_url;
#[cfg(feature = "model")]
use crate::model::user::primary_guild_badge_url;
#[cfg(feature = "model")]
use crate::model::user::static_avatar_url;
#[cfg(all(feature = "unstable_discord_api", feature = "model"))]
#[cfg(feature = "model")]
use crate::model::user::static_nameplate_url;
#[cfg(feature = "model")]
use crate::model::user::tag;
#[cfg(feature = "model")]
use crate::model::utils::{avatar_url, user_banner_url};
#[allow(async_fn_in_trait)]
pub trait AvatarDecorationDataExt: Sized {
    #[cfg(feature = "model")]
    #[must_use]
    /// Returns the formatted URL of the decoration.
    fn decoration_url(&self) -> String;
}
impl AvatarDecorationDataExt for AvatarDecorationData {
    #[cfg(feature = "model")]
    fn decoration_url(&self) -> String {
        avatar_decoration_url(&self.asset)
    }
}
#[allow(async_fn_in_trait)]
pub trait CurrentUserExt: Sized {
    #[cfg(feature = "model")]
    /// Edits the current user's profile settings.
    ///
    /// This mutates the current user in-place.
    ///
    /// Refer to [`EditProfile`]'s documentation for its methods.
    ///
    /// # Examples
    ///
    /// Change the avatar:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::builder::{EditProfile, CreateAttachment};
    /// # use oxcord::http::Http;
    /// # use oxcord::model::user::CurrentUser;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let mut user = CurrentUser::default();
    /// let avatar = CreateAttachment::path("./avatar.png").await?;
    /// user.edit(&http, EditProfile::new().avatar(&avatar)).await;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if an invalid value is set. May also return an [`Error::Json`]
    /// if there is an error in deserializing the API response.
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditProfile) -> Result<()>;
}
impl CurrentUserExt for CurrentUser {
    #[cfg(feature = "model")]
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditProfile) -> Result<()> {
        *self = builder.execute(cache_http, ()).await?;
        Ok(())
    }
}
#[allow(async_fn_in_trait)]
pub trait NameplateExt: Sized {
    #[cfg(all(feature = "unstable_discord_api", feature = "model"))]
    /// Gets the static version of the nameplate's url.
    #[must_use]
    fn static_url(&self) -> String;
    #[cfg(all(feature = "unstable_discord_api", feature = "model"))]
    /// Gets the animated version of the nameplate's url.
    #[must_use]
    fn url(&self) -> String;
}
impl NameplateExt for Nameplate {
    #[cfg(all(feature = "unstable_discord_api", feature = "model"))]
    fn static_url(&self) -> String {
        static_nameplate_url(&self.asset)
    }
    #[cfg(all(feature = "unstable_discord_api", feature = "model"))]
    fn url(&self) -> String {
        nameplate_url(&self.asset)
    }
}
#[allow(async_fn_in_trait)]
pub trait PrimaryGuildExt: Sized {
    #[cfg(feature = "model")]
    #[must_use]
    /// Returns the formatted URL of the badge's icon, if one exists.
    fn badge_url(&self) -> Option<String>;
}
impl PrimaryGuildExt for PrimaryGuild {
    #[cfg(feature = "model")]
    fn badge_url(&self) -> Option<String> {
        primary_guild_badge_url(self.identity_guild_id, self.badge.as_ref())
    }
}
#[allow(async_fn_in_trait)]
pub trait UserExt: Sized {
    #[cfg(feature = "model")]
    /// Returns the formatted URL of the user's icon, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the user has a GIF avatar.
    #[must_use]
    fn avatar_url(&self) -> Option<String>;
    #[cfg(feature = "model")]
    /// Returns the formatted URL of the user's banner, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the user has a GIF banner.
    #[must_use]
    fn banner_url(&self) -> Option<String>;
    #[cfg(feature = "model")]
    /// Creates a direct message channel between the [current user] and the user. This can also
    /// retrieve the channel if one already exists.
    ///
    /// [current user]: CurrentUser
    ///
    /// # Errors
    ///
    /// See [`UserId::create_dm_channel`] for what errors may be returned.
    async fn create_dm_channel(&self, cache_http: impl CacheHttp) -> Result<PrivateChannel>;
    #[cfg(feature = "model")]
    /// Retrieves the time that this user was created at.
    #[must_use]
    fn created_at(&self) -> Timestamp;
    #[cfg(feature = "model")]
    /// Returns the formatted URL to the user's default avatar URL.
    ///
    /// This will produce a PNG URL.
    #[must_use]
    fn default_avatar_url(&self) -> String;
    #[cfg(feature = "model")]
    /// Sends a message to a user through a direct message channel. This is a channel that can only
    /// be accessed by you and the recipient.
    ///
    /// # Examples
    ///
    /// See [`UserId::direct_message`] for examples.
    ///
    /// # Errors
    ///
    /// See [`UserId::direct_message`] for errors.
    async fn direct_message(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateMessage,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Calculates the user's display name.
    ///
    /// The global name takes priority over the user's username if it exists.
    ///
    /// Note: Guild specific information is not included as this is only available on the [Member].
    #[must_use]
    fn display_name(&self) -> &str;
    #[cfg(feature = "model")]
    /// This is an alias of [`Self::direct_message`].
    #[allow(clippy::missing_errors_doc)]
    async fn dm(&self, cache_http: impl CacheHttp, builder: CreateMessage) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Retrieves the URL to the user's avatar, falling back to the default avatar if needed.
    ///
    /// This will call [`Self::avatar_url`] first, and if that returns [`None`], it then falls back
    /// to [`Self::default_avatar_url`].
    #[must_use]
    fn face(&self) -> String;
    #[cfg(feature = "model")]
    /// Retrieves the URL to the static version of the user's avatar, falling back to the default
    /// avatar if needed.
    ///
    /// This will call [`Self::static_avatar_url`] first, and if that returns [`None`], it then
    /// falls back to [`Self::default_avatar_url`].
    #[must_use]
    fn static_face(&self) -> String;
    #[cfg(feature = "model")]
    /// Check if a user has a [`Role`]. This will retrieve the [`Guild`] from the [`Cache`] if it
    /// is available, and then check if that guild has the given [`Role`].
    ///
    /// # Examples
    ///
    /// Check if a guild has a [`Role`] by Id:
    ///
    /// ```rust,ignore
    /// // Assumes a 'guild_id' and `role_id` have already been bound
    /// let _ = message.author.has_role(guild_id, role_id);
    /// ```
    ///
    /// [`Cache`]: crate::cache::Cache
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the given [`Guild`] is unavailable, if that [`Role`] does not
    /// exist in the given [`Guild`], or if the given [`User`] is not in that [`Guild`].
    ///
    /// May also return an [`Error::Json`] if there is an error in deserializing the API response.
    async fn has_role(
        &self,
        cache_http: impl CacheHttp,
        guild_id: impl Into<GuildId>,
        role: impl Into<RoleId>,
    ) -> Result<bool>;
    #[cfg(feature = "model")]
    /// Refreshes the information about the user.
    ///
    /// Replaces the instance with the data retrieved over the REST API.
    ///
    /// # Errors
    ///
    /// See [`UserId::to_user`] for what errors may be returned.
    async fn refresh(&mut self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// Returns a static formatted URL of the user's icon, if one exists.
    ///
    /// This will always produce a WEBP image URL.
    #[must_use]
    fn static_avatar_url(&self) -> Option<String>;
    #[cfg(feature = "model")]
    /// Returns the "tag" for the user.
    ///
    /// The "tag" is defined as "username#discriminator", such as "zeyla#5479".
    ///
    /// # Examples
    ///
    /// Make a command to tell the user what their tag is:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::model::prelude::*;
    /// # struct Handler;
    ///
    /// #[oxcord::async_trait]
    /// # #[cfg(feature = "client")]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, msg: Message) {
    ///         if msg.content == "!mytag" {
    ///             let content = format!("Your tag is: {}", msg.author.tag());
    ///             let _ = msg.channel_id.say(&context.http, &content).await;
    ///         }
    ///     }
    /// }
    /// ```
    #[must_use]
    fn tag(&self) -> String;
    #[cfg(feature = "model")]
    /// Returns the user's nickname in the given `guild_id`.
    ///
    /// If none is used, it returns [`None`].
    async fn nick_in(
        &self,
        cache_http: impl CacheHttp,
        guild_id: impl Into<GuildId>,
    ) -> Option<String>;
    #[cfg(feature = "model")]
    /// Returns a builder which can be awaited to obtain a message or stream of messages sent by
    /// this user.
    #[cfg(feature = "collector")]
    fn await_reply(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector;
    #[cfg(feature = "model")]
    /// Same as [`Self::await_reply`].
    #[cfg(feature = "collector")]
    fn await_replies(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector;
    #[cfg(feature = "model")]
    /// Returns a builder which can be awaited to obtain a reaction or stream of reactions sent by
    /// this user.
    #[cfg(feature = "collector")]
    fn await_reaction(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector;
    #[cfg(feature = "model")]
    /// Same as [`Self::await_reaction`].
    #[cfg(feature = "collector")]
    fn await_reactions(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector;
}
impl UserExt for User {
    #[cfg(feature = "model")]
    #[inline]
    fn avatar_url(&self) -> Option<String> {
        avatar_url(None, self.id, self.avatar.as_ref())
    }
    #[cfg(feature = "model")]
    #[inline]
    fn banner_url(&self) -> Option<String> {
        user_banner_url(None, self.id, self.banner.as_ref())
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn create_dm_channel(&self, cache_http: impl CacheHttp) -> Result<PrivateChannel> {
        if self.bot {
            return Err(Error::Model(ModelError::MessagingBot));
        }
        self.id.create_dm_channel(cache_http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    fn created_at(&self) -> Timestamp {
        self.id.created_at()
    }
    #[cfg(feature = "model")]
    #[inline]
    fn default_avatar_url(&self) -> String {
        default_avatar_url(self)
    }
    #[cfg(feature = "model")]
    async fn direct_message(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateMessage,
    ) -> Result<Message> {
        self.id.direct_message(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    #[inline]
    fn display_name(&self) -> &str {
        self.global_name.as_deref().unwrap_or(&self.name)
    }
    #[cfg(feature = "model")]
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    async fn dm(&self, cache_http: impl CacheHttp, builder: CreateMessage) -> Result<Message> {
        self.direct_message(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    fn face(&self) -> String {
        self.avatar_url().unwrap_or_else(|| self.default_avatar_url())
    }
    #[cfg(feature = "model")]
    fn static_face(&self) -> String {
        self.static_avatar_url().unwrap_or_else(|| self.default_avatar_url())
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn has_role(
        &self,
        cache_http: impl CacheHttp,
        guild_id: impl Into<GuildId>,
        role: impl Into<RoleId>,
    ) -> Result<bool> {
        guild_id.into().member(cache_http, self).await.map(|m| m.roles.contains(&role.into()))
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn refresh(&mut self, cache_http: impl CacheHttp) -> Result<()> {
        *self = self.id.to_user(cache_http).await?;
        Ok(())
    }
    #[cfg(feature = "model")]
    #[inline]
    fn static_avatar_url(&self) -> Option<String> {
        static_avatar_url(self.id, self.avatar.as_ref())
    }
    #[cfg(feature = "model")]
    #[inline]
    fn tag(&self) -> String {
        tag(&self.name, self.discriminator)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn nick_in(
        &self,
        cache_http: impl CacheHttp,
        guild_id: impl Into<GuildId>,
    ) -> Option<String> {
        let guild_id = guild_id.into();
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = guild_id.to_guild_cached(cache) {
                    if let Some(member) = guild.members.get(&self.id) {
                        return member.nick.clone();
                    }
                }
            }
        }
        guild_id.member(cache_http, &self.id).await.ok().and_then(|member| member.nick)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reply(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector {
        MessageCollector::new(shard_messenger).author_id(self.id)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_replies(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector {
        self.await_reply(shard_messenger)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reaction(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).author_id(self.id)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reactions(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector {
        self.await_reaction(shard_messenger)
    }
}
#[allow(async_fn_in_trait)]
pub trait UserIdExt: Sized {
    #[cfg(feature = "model")]
    /// Creates a direct message channel between the [current user] and the user. This can also
    /// retrieve the channel if one already exists.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a [`User`] with that [`UserId`] does not exist, or is otherwise
    /// unavailable.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserializing the channel data
    /// returned by the Discord API.
    ///
    /// [current user]: CurrentUser
    async fn create_dm_channel(self, cache_http: impl CacheHttp) -> Result<PrivateChannel>;
    #[cfg(feature = "model")]
    /// Sends a message to a user through a direct message channel. This is a channel that can only
    /// be accessed by you and the recipient.
    ///
    /// # Examples
    ///
    /// When a user sends a message with a content of `"~help"`, DM the author a help message
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::model::prelude::*;
    /// # struct Handler;
    /// use oxcord::builder::CreateMessage;
    ///
    /// #[oxcord::async_trait]
    /// # #[cfg(feature = "client")]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, ctx: Context, msg: Message) {
    ///         if msg.content == "~help" {
    ///             let builder = CreateMessage::new().content("Helpful info here.");
    ///
    ///             if let Err(why) = msg.author.id.direct_message(&ctx, builder).await {
    ///                 println!("Err sending help: {why:?}");
    ///                 let _ = msg.reply(&ctx, "There was an error DMing you help.").await;
    ///             };
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessagingBot`] if the user being direct messaged is a bot user.
    ///
    /// May also return an [`Error::Http`] if the user cannot be sent a direct message.
    ///
    /// Returns an [`Error::Json`] if there is an error deserializing the API response.
    async fn direct_message(
        self,
        cache_http: impl CacheHttp,
        builder: CreateMessage,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// This is an alias of [`Self::direct_message`].
    #[allow(clippy::missing_errors_doc)]
    async fn dm(self, cache_http: impl CacheHttp, builder: CreateMessage) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Attempts to find a [`User`] by its Id in the cache.
    #[cfg(feature = "cache")]
    fn to_user_cached(self, cache: &impl AsRef<Cache>) -> Option<UserRef<'_>>;
    #[cfg(feature = "model")]
    /// First attempts to find a [`User`] by its Id in the cache, upon failure requests it via the
    /// REST API.
    ///
    /// **Note**: If the cache is not enabled, REST API will be used only.
    ///
    /// **Note**: If the cache is enabled, you might want to enable the `temp_cache` feature to
    /// cache user data retrieved by this function for a short duration.
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if a [`User`] with that [`UserId`] does not exist, or
    /// otherwise cannot be fetched.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserializing the user.
    async fn to_user(self, cache_http: impl CacheHttp) -> Result<User>;
}
impl UserIdExt for UserId {
    #[cfg(feature = "model")]
    async fn create_dm_channel(self, cache_http: impl CacheHttp) -> Result<PrivateChannel> {
        #[cfg(feature = "temp_cache")]
        if let Some(cache) = cache_http.cache() {
            if let Some(private_channel) = cache.temp_private_channels.get(&self) {
                return Ok(PrivateChannel::clone(&private_channel));
            }
        }
        let map = json!({ "recipient_id" : self, });
        let channel = cache_http.http().create_private_channel(&map).await?;
        #[cfg(feature = "temp_cache")]
        if let Some(cache) = cache_http.cache() {
            use crate::cache::MaybeOwnedArc;
            let cached_channel = MaybeOwnedArc::new(channel.clone());
            cache.temp_private_channels.insert(self, cached_channel);
        }
        Ok(channel)
    }
    #[cfg(feature = "model")]
    async fn direct_message(
        self,
        cache_http: impl CacheHttp,
        builder: CreateMessage,
    ) -> Result<Message> {
        self.create_dm_channel(&cache_http).await?.send_message(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    async fn dm(self, cache_http: impl CacheHttp, builder: CreateMessage) -> Result<Message> {
        self.direct_message(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    #[inline]
    fn to_user_cached(self, cache: &impl AsRef<Cache>) -> Option<UserRef<'_>> {
        cache.as_ref().user(self)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn to_user(self, cache_http: impl CacheHttp) -> Result<User> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(user) = cache.user(self) {
                    return Ok(user.clone());
                }
            }
        }
        let user = cache_http.http().get_user(self).await?;
        #[cfg(all(feature = "cache", feature = "temp_cache"))]
        {
            if let Some(cache) = cache_http.cache() {
                use crate::cache::MaybeOwnedArc;
                let cached_user = MaybeOwnedArc::new(user.clone());
                cache.temp_users.insert(cached_user.id, cached_user);
            }
        }
        Ok(user)
    }
}
