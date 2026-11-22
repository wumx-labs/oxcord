#[cfg(feature = "model")]
use std::sync::Arc;

#[cfg(feature = "model")]
use crate::builder::{
    Builder,
    CreateAttachment,
    CreateForumPost,
    CreateInvite,
    CreateMessage,
    CreateStageInstance,
    CreateThread,
    CreateWebhook,
    EditChannel,
    EditMessage,
    EditStageInstance,
    EditThread,
    EditVoiceState,
    GetMessages,
};
#[cfg(feature = "cache")]
use crate::cache::{self, Cache};
#[cfg(feature = "collector")]
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "collector")]
use crate::gateway::ShardMessenger;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http, Typing};
use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait GuildChannelExt: Sized {
    #[cfg(feature = "model")]
    /// Whether or not this channel is text-based, meaning that it is possible to send messages.
    #[must_use]
    fn is_text_based(&self) -> bool;
    #[cfg(feature = "model")]
    /// Broadcasts to the channel that the current user is typing.
    ///
    /// For bots, this is a good indicator for long-running commands.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have the required permissions.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    async fn broadcast_typing(&self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Creates an invite for the given channel.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// # Examples
    ///
    /// Create an invite that can only be used 5 times:
    ///
    /// ```rust,ignore
    /// let builder = CreateBuilder::default().max_uses(5);
    /// let invite = channel.create_invite(&context, builder).await;
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    #[cfg(feature = "utils")]
    async fn create_invite(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInvite<'_>,
    ) -> Result<RichInvite>;
    #[cfg(feature = "model")]
    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a single [`Member`] or
    /// [`Role`] within a [`Channel`].
    ///
    /// Refer to the documentation for [`PermissionOverwrite`]s for more information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Creating a permission overwrite for a member by specifying the
    /// [`PermissionOverwriteType::Member`] variant, allowing it the [Send Messages] permission,
    /// but denying the [Send TTS Messages] and [Attach Files] permissions:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use oxcord::{cache::Cache, http::Http, model::id::{ChannelId, UserId}};
    /// # use std::sync::Arc;
    /// #
    /// # let http: Arc<Http> = unimplemented!();
    /// # let cache = Cache::default();
    /// # let (channel_id, user_id) = (ChannelId::new(1), UserId::new(1));
    /// use oxcord::model::channel::{PermissionOverwrite, PermissionOverwriteType};
    /// use oxcord::model::{ModelError, Permissions};
    /// let allow = Permissions::SEND_MESSAGES;
    /// let deny = Permissions::SEND_TTS_MESSAGES | Permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow,
    ///     deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    /// // assuming the cache has been unlocked
    /// let channel = cache.channel(channel_id).ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.create_permission(&http, overwrite).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Creating a permission overwrite for a role by specifying the
    /// [`PermissionOverwriteType::Role`] variant, allowing it the [Manage Webhooks]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use oxcord::{cache::Cache, http::Http, model::id::{ChannelId, UserId, RoleId}};
    /// # use std::sync::Arc;
    /// #
    /// # let http: Arc<Http> = unimplemented!();
    /// # let cache = Cache::default();
    /// # let (channel_id, user_id, role_id) = (ChannelId::new(1), UserId::new(1), RoleId::new(1));
    /// use oxcord::model::channel::{Channel, PermissionOverwrite, PermissionOverwriteType};
    /// use oxcord::model::{ModelError, Permissions};
    ///
    /// let allow = Permissions::SEND_MESSAGES;
    /// let deny = Permissions::SEND_TTS_MESSAGES | Permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow,
    ///     deny,
    ///     kind: PermissionOverwriteType::Role(role_id),
    /// };
    ///
    /// let channel = cache.channel(channel_id).ok_or(ModelError::ItemMissing)?;
    ///
    /// channel.create_permission(&http, overwrite).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    /// [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS
    /// [Send Messages]: Permissions::SEND_MESSAGES
    /// [Send TTS Messages]: Permissions::SEND_TTS_MESSAGES
    async fn create_permission(
        &self,
        http: impl AsRef<Http>,
        target: PermissionOverwrite,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes this channel, returning the channel on a successful deletion.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission.
    ///
    /// Otherwise returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<GuildChannel>;
    #[cfg(feature = "model")]
    /// Deletes all messages by Ids from the given vector in the channel.
    ///
    /// The minimum amount of messages is 2 and the maximum amount is 100.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// **Note**: Messages that are older than 2 weeks can't be deleted using this method.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::BulkDeleteAmount`] if an attempt was made to delete either 0 or more
    /// than 100 messages.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn delete_messages<T: AsRef<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_ids: impl IntoIterator<Item = T>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes all permission overrides in the channel from a member or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channel]: Permissions::MANAGE_CHANNELS
    async fn delete_permission(
        &self,
        http: impl AsRef<Http>,
        permission_type: PermissionOverwriteType,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current user did not perform
    /// the reaction.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission. [Manage Messages]:
    /// Permissions::MANAGE_MESSAGES
    async fn delete_reaction(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes all of the [`Reaction`]s associated with the provided message id.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn delete_reactions(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits the channel's settings.
    ///
    /// Refer to the documentation for [`EditChannel`] for a full list of methods.
    ///
    /// **Note**: Requires the [Manage Channels] permission. Modifying permissions via
    /// [`EditChannel::permissions`] also requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Change a voice channels name and bitrate:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::builder::EditChannel;
    /// # use oxcord::http::Http;
    /// # use oxcord::model::id::ChannelId;
    /// # async fn run() {
    /// # let http: Http = unimplemented!();
    /// # let channel = ChannelId::new(1234);
    /// let builder = EditChannel::new().name("test").bitrate(86400);
    /// channel.edit(&http, builder).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditChannel<'_>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits a [`Message`] in the channel given its Id.
    ///
    /// Message editing preserves all unchanged message data, with some exceptions for embeds and
    /// attachments.
    ///
    /// **Note**: In most cases requires that the current user be the author of the message.
    ///
    /// Refer to the documentation for [`EditMessage`] for information regarding content
    /// restrictions and requirements.
    ///
    /// # Errors
    ///
    /// See [`EditMessage::execute`] for a list of possible errors, and their corresponding
    /// reasons.
    async fn edit_message(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
        builder: EditMessage,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Edits a thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    async fn edit_thread(
        &mut self,
        cache_http: impl CacheHttp,
        builder: EditThread<'_>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits the voice state of a given user in a stage channel.
    ///
    /// **Note**: Requires the [Request to Speak] permission. Also requires the [Mute Members]
    /// permission to suppress another user or unsuppress the current user. This is not required if
    /// suppressing the current user.
    ///
    /// # Example
    ///
    /// Invite a user to speak.
    ///
    /// ```rust
    /// # use oxcord::prelude::*;
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::sync::Arc;
    /// # use oxcord::{cache::Cache, http::Http, model::id::{ChannelId, UserId}};
    /// #
    /// # let http: Http = unimplemented!();
    /// # let cache = Cache::default();
    /// # let (channel_id, user_id) = (ChannelId::new(1), UserId::new(1));
    /// use oxcord::builder::EditVoiceState;
    /// use oxcord::model::ModelError;
    ///
    /// // assuming the cache has been unlocked
    /// let channel = cache.channel(channel_id).ok_or(ModelError::ItemMissing)?;
    ///
    /// let builder = EditVoiceState::new().suppress(false);
    /// channel.edit_voice_state(&http, user_id, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if the user lacks permission, or if invalid data is given.
    ///
    /// [Request to Speak]: Permissions::REQUEST_TO_SPEAK
    /// [Mute Members]: Permissions::MUTE_MEMBERS
    async fn edit_voice_state(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
        builder: EditVoiceState,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits the current user's voice state in a stage channel.
    ///
    /// **Note**: Requires the [Request to Speak] permission. The [Mute Members] permission is
    /// **not** required.
    ///
    /// # Example
    ///
    /// Send a request to speak, then clear the request.
    ///
    /// ```rust
    /// # use oxcord::prelude::*;
    /// # #[cfg(feature = "cache")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use std::sync::Arc;
    /// # use oxcord::{cache::Cache, http::Http, model::id::ChannelId};
    /// #
    /// # let http: Http = unimplemented!();
    /// # let cache = Cache::default();
    /// # let channel_id = ChannelId::new(1);
    /// use oxcord::builder::EditVoiceState;
    /// use oxcord::model::ModelError;
    ///
    /// // assuming the cache has been unlocked
    /// let channel = cache.channel(channel_id).ok_or(ModelError::ItemMissing)?;
    ///
    /// // Send a request to speak
    /// let builder = EditVoiceState::new().request_to_speak(true);
    /// channel.edit_own_voice_state(&http, builder.clone()).await?;
    ///
    /// // Clear own request to speak
    /// let builder = builder.request_to_speak(false);
    /// channel.edit_own_voice_state(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidChannelType`] if the channel is
    /// not a stage channel.
    ///
    /// Returns [`Error::Http`] if the user lacks permission, or if invalid data is given.
    ///
    /// [Request to Speak]: Permissions::REQUEST_TO_SPEAK
    /// [Mute Members]: Permissions::MUTE_MEMBERS
    async fn edit_own_voice_state(
        &self,
        cache_http: impl CacheHttp,
        builder: EditVoiceState,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Follows the News Channel
    ///
    /// Requires [Manage Webhook] permissions on the target channel.
    ///
    /// **Note**: Only available on news channels.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn follow(
        &self,
        http: impl AsRef<Http>,
        target_channel_id: impl Into<ChannelId>,
    ) -> Result<FollowedChannel>;
    #[cfg(feature = "model")]
    /// Attempts to find this channel's guild in the Cache.
    #[cfg(feature = "cache")]
    fn guild<'a>(&self, cache: &'a impl AsRef<Cache>) -> Option<cache::GuildRef<'a>>;
    #[cfg(feature = "model")]
    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    async fn invites(&self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>>;
    #[cfg(feature = "model")]
    /// Determines if the channel is NSFW.
    #[must_use]
    #[deprecated(since = "0.0.1", note = "Use the GuildChannel::nsfw field")]
    fn is_nsfw(&self) -> bool;
    #[cfg(feature = "model")]
    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a message with the
    /// given Id does not exist in the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    async fn message(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Gets messages from the channel.
    ///
    /// **Note**: If the user does not have the [Read Message History] permission, returns an empty
    /// [`Vec`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    async fn messages(
        &self,
        cache_http: impl CacheHttp,
        builder: GetMessages,
    ) -> Result<Vec<Message>>;
    #[cfg(feature = "model")]
    /// Returns the name of the guild channel.
    #[must_use]
    fn name(&self) -> &str;
    #[cfg(feature = "model")]
    /// Calculates the permissions of a member.
    ///
    /// The Id of the argument must be a [`Member`] of the [`Guild`] that the channel is in.
    ///
    /// # Examples
    ///
    /// Calculate the permissions of a [`User`] who posted a [`Message`] in a channel:
    ///
    /// ```rust,no_run
    /// # use oxcord::model::prelude::*;
    /// # use oxcord::prelude::*;
    /// # struct Handler;
    ///
    /// #[oxcord::async_trait]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, msg: Message) {
    ///         let channel = match context.cache.channel(msg.channel_id) {
    ///             Some(channel) => channel,
    ///             None => return,
    ///         };
    ///
    ///         if let Ok(permissions) = channel.permissions_for_user(&context.cache, &msg.author) {
    ///             println!("The user's permissions: {:?}", permissions);
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the channel's guild could not be found in the
    /// [`Cache`].
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    /// [Send Messages]: Permissions::SEND_MESSAGES
    #[cfg(feature = "cache")]
    #[deprecated(since = "0.0.1", note = "Use `Guild::user_permissions_in`")]
    fn permissions_for_user(
        &self,
        cache: impl AsRef<Cache>,
        user_id: impl Into<UserId>,
    ) -> Result<Permissions>;
    #[cfg(feature = "model")]
    /// Calculates the permissions of a role.
    ///
    /// The Id of the argument must be a [`Role`] of the [`Guild`] that the channel is in.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the channel's guild could not be found in the
    /// [`Cache`].
    ///
    /// Returns a [`ModelError::RoleNotFound`] if the given role could not be found in the
    /// [`Cache`].
    #[cfg(feature = "cache")]
    #[deprecated(
        since = "0.0.1",
        note = "this function ignores other roles the user may have as well as user-specific permissions; use Guild::user_permissions_in instead"
    )]
    #[allow(deprecated)]
    fn permissions_for_role(
        &self,
        cache: impl AsRef<Cache>,
        role_id: impl Into<RoleId>,
    ) -> Result<Permissions>;
    #[cfg(feature = "model")]
    /// Pins a [`Message`] to the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the channel already has
    /// too many pinned messages.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn pin(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Gets all channel's pins.
    ///
    /// **Note**: If the current user lacks the [Read Message History] permission an empty [`Vec`]
    /// will be returned.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to view the channel.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    async fn pins(&self, http: impl AsRef<Http>) -> Result<Vec<Message>>;
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
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<User>>;
    #[cfg(feature = "model")]
    /// Sends a message with just the given message content in the channel.
    ///
    /// **Note**: Message content must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content length is over the above limit. See
    /// [`CreateMessage::execute`] for more details.
    async fn say(&self, cache_http: impl CacheHttp, content: impl Into<String>) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Sends file(s) along with optional message contents.
    ///
    /// Refer to [`ChannelId::send_files`] for examples and more information.
    ///
    /// # Errors
    ///
    /// See [`CreateMessage::execute`] for a list of possible errors, and their corresponding
    /// reasons.
    async fn send_files(
        self,
        cache_http: impl CacheHttp,
        files: impl IntoIterator<Item = CreateAttachment>,
        builder: CreateMessage,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Sends a message to the channel.
    ///
    /// Refer to the documentation for [`CreateMessage`] for information regarding content
    /// restrictions and requirements.
    ///
    /// # Errors
    ///
    /// See [`CreateMessage::execute`] for a list of possible errors, and their corresponding
    /// reasons.
    async fn send_message(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateMessage,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Starts typing in the channel for an indefinite period of time.
    ///
    /// Returns [`Typing`] that is used to trigger the typing. [`Typing::stop`] must be called on
    /// the returned struct to stop typing. Note that on some clients, typing may persist for a few
    /// seconds after [`Typing::stop`] is called. Typing is also stopped when the struct is
    /// dropped.
    ///
    /// If a message is sent while typing is triggered, the user will stop typing for a brief
    /// period of time and then resume again until either [`Typing::stop`] is called or the struct
    /// is dropped.
    ///
    /// This should rarely be used for bots, although it is a good indicator that a long-running
    /// command is still being processed.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # #[cfg(feature = "cache")]
    /// # async fn run() {
    /// # use oxcord::{cache::Cache, http::Http, model::channel::GuildChannel, Result};
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # let http: Arc<Http> = unimplemented!();
    /// # let cache = Cache::default();
    /// # let channel: GuildChannel = unimplemented!();
    /// // Initiate typing (assuming http is `Arc<Http>` and `channel` is bound)
    /// let typing = channel.start_typing(&http);
    ///
    /// // Run some long-running process
    /// long_process();
    ///
    /// // Stop typing
    /// typing.stop();
    /// # }
    /// ```
    #[allow(clippy::missing_errors_doc)]
    fn start_typing(&self, http: &Arc<Http>) -> Typing;
    #[cfg(feature = "model")]
    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    async fn unpin(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS
    async fn webhooks(&self, http: impl AsRef<Http>) -> Result<Vec<Webhook>>;
    #[cfg(feature = "model")]
    /// Retrieves [`Member`]s from the current channel.
    ///
    /// [`ChannelType::Voice`] and [`ChannelType::Stage`] returns [`Member`]s using the channel.
    ///
    /// [`ChannelType::Text`] and [`ChannelType::News`] return [`Member`]s that can read the
    /// channel.
    ///
    /// # Errors
    ///
    /// Other [`ChannelType`]s lack the concept of [`Member`]s and will return:
    /// [`ModelError::InvalidChannelType`].
    #[cfg(feature = "cache")]
    fn members(&self, cache: impl AsRef<Cache>) -> Result<Vec<Member>>;
    #[cfg(feature = "model")]
    /// Returns a builder which can be awaited to obtain a message or stream of messages sent in
    /// this guild channel.
    #[cfg(feature = "collector")]
    fn await_reply(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector;
    #[cfg(feature = "model")]
    /// Same as [`Self::await_reply`].
    #[cfg(feature = "collector")]
    fn await_replies(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector;
    #[cfg(feature = "model")]
    /// Returns a stream builder which can be awaited to obtain a reaction or stream of reactions
    /// sent by this guild channel.
    #[cfg(feature = "collector")]
    fn await_reaction(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector;
    #[cfg(feature = "model")]
    /// Same as [`Self::await_reaction`].
    #[cfg(feature = "collector")]
    fn await_reactions(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector;
    #[cfg(feature = "model")]
    /// Creates a webhook in the channel.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the corresponding channel is not of type
    /// [`ChannelType::Text`] or [`ChannelType::News`].
    ///
    /// See [`CreateWebhook::execute`] for a detailed list of other
    /// possible errors,
    async fn create_webhook(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateWebhook<'_>,
    ) -> Result<Webhook>;
    #[cfg(feature = "model")]
    /// Gets a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if there is no stage instance currently.
    async fn get_stage_instance(&self, http: impl AsRef<Http>) -> Result<StageInstance>;
    #[cfg(feature = "model")]
    /// Creates a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if there is already a stage instance currently.
    async fn create_stage_instance(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateStageInstance<'_>,
    ) -> Result<StageInstance>;
    #[cfg(feature = "model")]
    /// Edits the stage instance
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if the channel is not a stage channel, or there is no stage
    /// instance currently.
    async fn edit_stage_instance(
        &self,
        cache_http: impl CacheHttp,
        builder: EditStageInstance<'_>,
    ) -> Result<StageInstance>;
    #[cfg(feature = "model")]
    /// Deletes a stage instance.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidChannelType`] if the channel is not a stage channel.
    ///
    /// Returns [`Error::Http`] if there is no stage instance currently.
    async fn delete_stage_instance(&self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Creates a public thread that is connected to a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    async fn create_thread_from_message(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
        builder: CreateThread<'_>,
    ) -> Result<GuildChannel>;
    #[cfg(feature = "model")]
    /// Creates a thread that is not connected to a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    async fn create_thread(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateThread<'_>,
    ) -> Result<GuildChannel>;
    #[cfg(feature = "model")]
    /// Creates a post in a forum channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    async fn create_forum_post(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateForumPost<'_>,
    ) -> Result<GuildChannel>;
}
impl GuildChannelExt for GuildChannel {
    #[cfg(feature = "model")]
    fn is_text_based(&self) -> bool {
        matches!(
            self.kind,
            ChannelType::Text
                | ChannelType::News
                | ChannelType::Voice
                | ChannelType::Stage
                | ChannelType::PublicThread
                | ChannelType::PrivateThread
                | ChannelType::NewsThread
        )
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn broadcast_typing(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.broadcast_typing(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    #[cfg(feature = "utils")]
    async fn create_invite(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInvite<'_>,
    ) -> Result<RichInvite> {
        builder.execute(cache_http, self.id).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn create_permission(
        &self,
        http: impl AsRef<Http>,
        target: PermissionOverwrite,
    ) -> Result<()> {
        self.id.create_permission(http, target).await
    }
    #[cfg(feature = "model")]
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<GuildChannel> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(cache, self.id, Permissions::MANAGE_CHANNELS)?;
            }
        }
        let channel = self.id.delete(cache_http.http()).await?;
        channel.guild().ok_or(Error::Model(ModelError::InvalidChannelType))
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete_messages<T: AsRef<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_ids: impl IntoIterator<Item = T>,
    ) -> Result<()> {
        self.id.delete_messages(http, message_ids).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete_permission(
        &self,
        http: impl AsRef<Http>,
        permission_type: PermissionOverwriteType,
    ) -> Result<()> {
        self.id.delete_permission(http, permission_type).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete_reaction(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        self.id.delete_reaction(http, message_id, user_id, reaction_type).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete_reactions(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
    ) -> Result<()> {
        self.id.delete_reactions(http, message_id).await
    }
    #[cfg(feature = "model")]
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditChannel<'_>) -> Result<()> {
        let channel = builder.execute(cache_http, self.id).await?;
        *self = channel;
        Ok(())
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn edit_message(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
        builder: EditMessage,
    ) -> Result<Message> {
        self.id.edit_message(cache_http, message_id, builder).await
    }
    #[cfg(feature = "model")]
    async fn edit_thread(
        &mut self,
        cache_http: impl CacheHttp,
        builder: EditThread<'_>,
    ) -> Result<()> {
        *self = self.id.edit_thread(cache_http, builder).await?;
        Ok(())
    }
    #[cfg(feature = "model")]
    async fn edit_voice_state(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
        builder: EditVoiceState,
    ) -> Result<()> {
        if self.kind != ChannelType::Stage {
            return Err(Error::from(ModelError::InvalidChannelType));
        }
        builder
            .execute(cache_http, (self.guild_id, self.id, Some(user_id.into())))
            .await
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn edit_own_voice_state(
        &self,
        cache_http: impl CacheHttp,
        builder: EditVoiceState,
    ) -> Result<()> {
        builder.execute(cache_http, (self.guild_id, self.id, None)).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn follow(
        &self,
        http: impl AsRef<Http>,
        target_channel_id: impl Into<ChannelId>,
    ) -> Result<FollowedChannel> {
        self.id.follow(http, target_channel_id).await
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    #[inline]
    fn guild<'a>(&self, cache: &'a impl AsRef<Cache>) -> Option<cache::GuildRef<'a>> {
        cache.as_ref().guild(self.guild_id)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn invites(&self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {
        self.id.invites(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    fn is_nsfw(&self) -> bool {
        self.nsfw
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn message(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
    ) -> Result<Message> {
        self.id.message(cache_http, message_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn messages(
        &self,
        cache_http: impl CacheHttp,
        builder: GetMessages,
    ) -> Result<Vec<Message>> {
        self.id.messages(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    fn name(&self) -> &str {
        &self.name
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    #[inline]
    fn permissions_for_user(
        &self,
        cache: impl AsRef<Cache>,
        user_id: impl Into<UserId>,
    ) -> Result<Permissions> {
        let guild = self.guild(&cache).ok_or(Error::Model(ModelError::GuildNotFound))?;
        let member =
            guild.members.get(&user_id.into()).ok_or(Error::Model(ModelError::MemberNotFound))?;
        Ok(guild.user_permissions_in(self, member))
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    #[inline]
    #[allow(deprecated)]
    fn permissions_for_role(
        &self,
        cache: impl AsRef<Cache>,
        role_id: impl Into<RoleId>,
    ) -> Result<Permissions> {
        let guild = self.guild(&cache).ok_or(Error::Model(ModelError::GuildNotFound))?;
        let role =
            guild.roles.get(&role_id.into()).ok_or(Error::Model(ModelError::RoleNotFound))?;
        guild.role_permissions_in(self, role)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn pin(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()> {
        self.id.pin(http, message_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn pins(&self, http: impl AsRef<Http>) -> Result<Vec<Message>> {
        self.id.pins(http).await
    }
    #[cfg(feature = "model")]
    async fn reaction_users(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<User>> {
        self.id.reaction_users(http, message_id, reaction_type, limit, after).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn say(&self, cache_http: impl CacheHttp, content: impl Into<String>) -> Result<Message> {
        self.id.say(cache_http, content).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn send_files(
        self,
        cache_http: impl CacheHttp,
        files: impl IntoIterator<Item = CreateAttachment>,
        builder: CreateMessage,
    ) -> Result<Message> {
        self.send_message(cache_http, builder.files(files)).await
    }
    #[cfg(feature = "model")]
    async fn send_message(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateMessage,
    ) -> Result<Message> {
        builder.execute(cache_http, (self.id, Some(self.guild_id))).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    #[allow(clippy::missing_errors_doc)]
    fn start_typing(&self, http: &Arc<Http>) -> Typing {
        http.start_typing(self.id)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn unpin(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()> {
        self.id.unpin(http, message_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn webhooks(&self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> {
        self.id.webhooks(http).await
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    #[inline]
    fn members(&self, cache: impl AsRef<Cache>) -> Result<Vec<Member>> {
        let cache = cache.as_ref();
        let guild = cache.guild(self.guild_id).ok_or(ModelError::GuildNotFound)?;
        match self.kind {
            ChannelType::Voice | ChannelType::Stage => Ok(guild
                .voice_states
                .values()
                .filter_map(|v| {
                    v.channel_id.and_then(|c| {
                        if c == self.id { guild.members.get(&v.user_id).cloned() } else { None }
                    })
                })
                .collect()),
            ChannelType::News | ChannelType::Text => Ok(guild
                .members
                .values()
                .filter(|member| {
                    guild.user_permissions_in(self, member).contains(Permissions::VIEW_CHANNEL)
                })
                .cloned()
                .collect::<Vec<Member>>()),
            _ => Err(Error::from(ModelError::InvalidChannelType)),
        }
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reply(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector {
        MessageCollector::new(shard_messenger).channel_id(self.id)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_replies(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector {
        self.await_reply(shard_messenger)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reaction(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).channel_id(self.id)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reactions(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector {
        self.await_reaction(shard_messenger)
    }
    #[cfg(feature = "model")]
    async fn create_webhook(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateWebhook<'_>,
    ) -> Result<Webhook> {
        if !self.is_text_based() && self.kind != ChannelType::Forum {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }
        self.id.create_webhook(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    async fn get_stage_instance(&self, http: impl AsRef<Http>) -> Result<StageInstance> {
        if self.kind != ChannelType::Stage {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }
        self.id.get_stage_instance(http).await
    }
    #[cfg(feature = "model")]
    async fn create_stage_instance(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateStageInstance<'_>,
    ) -> Result<StageInstance> {
        if self.kind != ChannelType::Stage {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }
        self.id.create_stage_instance(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    async fn edit_stage_instance(
        &self,
        cache_http: impl CacheHttp,
        builder: EditStageInstance<'_>,
    ) -> Result<StageInstance> {
        if self.kind != ChannelType::Stage {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }
        self.id.edit_stage_instance(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    async fn delete_stage_instance(&self, http: impl AsRef<Http>) -> Result<()> {
        if self.kind != ChannelType::Stage {
            return Err(Error::Model(ModelError::InvalidChannelType));
        }
        self.id.delete_stage_instance(http).await
    }
    #[cfg(feature = "model")]
    async fn create_thread_from_message(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
        builder: CreateThread<'_>,
    ) -> Result<GuildChannel> {
        self.id.create_thread_from_message(cache_http, message_id, builder).await
    }
    #[cfg(feature = "model")]
    async fn create_thread(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateThread<'_>,
    ) -> Result<GuildChannel> {
        self.id.create_thread(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    async fn create_forum_post(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateForumPost<'_>,
    ) -> Result<GuildChannel> {
        self.id.create_forum_post(cache_http, builder).await
    }
}
