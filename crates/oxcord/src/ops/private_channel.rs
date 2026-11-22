#[cfg(feature = "model")]
use std::sync::Arc;

#[cfg(feature = "model")]
use crate::builder::{CreateAttachment, CreateMessage, EditMessage, GetMessages};
#[cfg(feature = "model")]
use crate::http::CacheHttp;
#[cfg(feature = "model")]
use crate::http::{Http, Typing};
use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait PrivateChannelExt: Sized {
    #[cfg(feature = "model")]
    /// Broadcasts that the current user is typing to the recipient.
    ///
    /// See [ChannelId::broadcast_typing] for more details.
    #[allow(clippy::missing_errors_doc)]
    async fn broadcast_typing(&self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most cases.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the reaction cannot be added, or if a message with that Id does
    /// not exist.
    async fn create_reaction(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes the channel. This does not delete the contents of the channel, and is equivalent to
    /// closing a private channel on the client, which can be re-opened.
    #[allow(clippy::missing_errors_doc)]
    async fn delete(&self, http: impl AsRef<Http>) -> Result<PrivateChannel>;
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
    /// [Manage Channel]: Permissions::MANAGE_CHANNELS
    #[allow(clippy::missing_errors_doc)]
    async fn delete_permission(
        &self,
        http: impl AsRef<Http>,
        permission_type: PermissionOverwriteType,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: In private channels, the current user may only delete it's own reactions.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the reaction is not from the current user.
    async fn delete_reaction(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()>;
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
    ///
    /// [`EditMessage::execute`]: ../../builder/struct.EditMessage.html#method.execute
    async fn edit_message(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
        builder: EditMessage,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Determines if the channel is NSFW.
    #[must_use]
    #[allow(clippy::unused_self)]
    #[deprecated(since = "0.0.1", note = "This always returns false")]
    fn is_nsfw(&self) -> bool;
    #[cfg(feature = "model")]
    /// Gets a message from the channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a message with that Id does not exist in this channel.
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
    /// Returns "DM with $username#discriminator".
    #[must_use]
    fn name(&self) -> String;
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
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a message with the given Id does not exist in the channel.
    async fn reaction_users(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<User>>;
    #[cfg(feature = "model")]
    /// Pins a [`Message`] to the channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the number of pinned messages would exceed the 50 message limit.
    async fn pin(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Retrieves the list of messages that have been pinned in the private channel.
    #[allow(clippy::missing_errors_doc)]
    async fn pins(&self, http: impl AsRef<Http>) -> Result<Vec<Message>>;
    #[cfg(feature = "model")]
    /// Sends a message with just the given message content in the channel.
    ///
    /// **Note**: Message content must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content length is over the above limit. See
    /// [`CreateMessage::execute`] for more details.
    ///
    /// [`CreateMessage::execute`]: ../../builder/struct.CreateMessage.html#method.execute
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
    ///
    /// [`CreateMessage::execute`]: ../../builder/struct.CreateMessage.html#method.execute
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
    ///
    /// [`CreateMessage::execute`]: ../../builder/struct.CreateMessage.html#method.execute
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
    /// # use oxcord::{cache::Cache, http::Http, model::channel::PrivateChannel, Result};
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # let http: Arc<Http> = unimplemented!();
    /// # let cache = Cache::default();
    /// # let channel: PrivateChannel = unimplemented!();
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
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the current user cannot send a direct message to this user.
    fn start_typing(self, http: &Arc<Http>) -> Typing;
    #[cfg(feature = "model")]
    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, if the message was deleted,
    /// or if the channel already has the limit of 50 pinned messages.
    async fn unpin(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()>;
}
impl PrivateChannelExt for PrivateChannel {
    #[cfg(feature = "model")]
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    async fn broadcast_typing(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.broadcast_typing(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn create_reaction(
        &self,
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        self.id.create_reaction(http, message_id, reaction_type).await
    }
    #[cfg(feature = "model")]
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    async fn delete(&self, http: impl AsRef<Http>) -> Result<PrivateChannel> {
        self.id.delete(http).await?.private().ok_or(Error::Model(ModelError::InvalidChannelType))
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
    #[allow(clippy::missing_errors_doc)]
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
    async fn edit_message(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
        builder: EditMessage,
    ) -> Result<Message> {
        self.id.edit_message(cache_http, message_id, builder).await
    }
    #[cfg(feature = "model")]
    #[inline]
    #[allow(clippy::unused_self)]
    fn is_nsfw(&self) -> bool {
        false
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
    fn name(&self) -> String {
        format!("DM with {}", self.recipient.tag())
    }
    #[cfg(feature = "model")]
    #[inline]
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
    async fn pin(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()> {
        self.id.pin(http, message_id).await
    }
    #[cfg(feature = "model")]
    #[allow(clippy::missing_errors_doc)]
    #[inline]
    async fn pins(&self, http: impl AsRef<Http>) -> Result<Vec<Message>> {
        self.id.pins(http).await
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
        self.id.send_files(cache_http, files, builder).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn send_message(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateMessage,
    ) -> Result<Message> {
        self.id.send_message(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    fn start_typing(self, http: &Arc<Http>) -> Typing {
        http.start_typing(self.id)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn unpin(&self, http: impl AsRef<Http>, message_id: impl Into<MessageId>) -> Result<()> {
        self.id.unpin(http, message_id).await
    }
}
