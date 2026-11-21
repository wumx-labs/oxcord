#[cfg(doc)]
use std::fmt::Display as _;

#[cfg(feature = "model")]
use tracing::warn;

#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait ReactionExt: Sized {
    #[cfg(feature = "model")]
    /// Retrieves the associated the reaction was made in.
    ///
    /// If the cache is enabled, this will search for the already-cached channel. If not - or the
    /// channel was not found - this will perform a request over the REST API for the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the channel no longer
    /// exists.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    async fn channel(&self, cache_http: impl CacheHttp) -> Result<Channel>;
    #[cfg(feature = "model")]
    /// Deletes the reaction, but only if the current user is the user who made the reaction or has
    /// permission to.
    ///
    /// Requires the [Manage Messages] permission, _if_ the current user did not perform the
    /// reaction.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a [`ModelError::InvalidPermissions`] if the current
    /// user does not have the required [permissions].
    ///
    /// Otherwise returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    /// [permissions]: crate::model::permissions
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes all reactions from the message with this emoji.
    ///
    /// Requires the [Manage Messages] permission
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a [`ModelError::InvalidPermissions`] if the current
    /// user does not have the required [permissions].
    ///
    /// Otherwise returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    /// [permissions]: crate::model::permissions
    async fn delete_all(&self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// Retrieves the [`Message`] associated with this reaction.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to read message history, or if
    /// the message was deleted.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    async fn message(&self, cache_http: impl CacheHttp) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Retrieves the user that made the reaction.
    ///
    /// If the cache is enabled, this will search for the already-cached user. If not - or the user
    /// was not found - this will perform a request over the REST API for the user.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the user that made the reaction is unable to be retrieved from
    /// the API.
    async fn user(&self, cache_http: impl CacheHttp) -> Result<User>;
    #[cfg(feature = "model")]
    /// Retrieves the list of [`User`]s who have reacted to a [`Message`] with a certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different maximum number of
    /// users. The maximum that may be retrieve at a time is `100`, if a greater number is provided
    /// then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain user. This is
    /// useful for pagination.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// **Note**: This will send a request to the REST API.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::InvalidPermissions`] if the current user does not have the required
    /// [permissions].
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    /// [permissions]: crate::model::permissions
    async fn users<R, U>(
        &self,
        http: impl AsRef<Http>,
        reaction_type: R,
        limit: Option<u8>,
        after: Option<U>,
    ) -> Result<Vec<User>>
    where
        R: Into<ReactionType>,
        U: Into<UserId>;
}
impl ReactionExt for Reaction {
    #[cfg(feature = "model")]
    #[inline]
    async fn channel(&self, cache_http: impl CacheHttp) -> Result<Channel> {
        self.channel_id.to_channel(cache_http).await
    }
    #[cfg(feature = "model")]
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg_attr(not(feature = "cache"), allow(unused_mut))]
        let mut user_id = self.user_id;
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.user_id == Some(cache.current_user().id) {
                    user_id = None;
                }
                if user_id.is_some() {
                    crate::utils::user_has_perms_cache(
                        cache,
                        self.channel_id,
                        Permissions::MANAGE_MESSAGES,
                    )?;
                }
            }
        }
        self.channel_id
            .delete_reaction(cache_http.http(), self.message_id, user_id, self.emoji.clone())
            .await
    }
    #[cfg(feature = "model")]
    async fn delete_all(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(
                    cache,
                    self.channel_id,
                    Permissions::MANAGE_MESSAGES,
                )?;
            }
        }
        cache_http
            .http()
            .as_ref()
            .delete_message_reaction_emoji(self.channel_id, self.message_id, &self.emoji)
            .await
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn message(&self, cache_http: impl CacheHttp) -> Result<Message> {
        self.channel_id.message(cache_http, self.message_id).await
    }
    #[cfg(feature = "model")]
    async fn user(&self, cache_http: impl CacheHttp) -> Result<User> {
        if let Some(id) = self.user_id {
            id.to_user(cache_http).await
        } else {
            #[cfg(feature = "cache")]
            {
                if let Some(cache) = cache_http.cache() {
                    return Ok(cache.current_user().clone().into());
                }
            }
            Ok(cache_http.http().get_current_user().await?.into())
        }
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn users<R, U>(
        &self,
        http: impl AsRef<Http>,
        reaction_type: R,
        limit: Option<u8>,
        after: Option<U>,
    ) -> Result<Vec<User>>
    where
        R: Into<ReactionType>,
        U: Into<UserId>,
    {
        self.users_(http, &reaction_type.into(), limit, after.map(Into::into)).await
    }
}

#[cfg(feature = "model")]
#[allow(async_fn_in_trait)]
pub(crate) trait ReactionHelpers {
    async fn users_(
        &self,
        http: impl AsRef<Http>,
        reaction_type: &ReactionType,
        limit: Option<u8>,
        after: Option<UserId>,
    ) -> Result<Vec<User>>;
}

#[cfg(feature = "model")]
impl ReactionHelpers for Reaction {
    async fn users_(
        &self,
        http: impl AsRef<Http>,
        reaction_type: &ReactionType,
        limit: Option<u8>,
        after: Option<UserId>,
    ) -> Result<Vec<User>> {
        let mut limit = limit.unwrap_or(50);

        if limit > 100 {
            limit = 100;
            warn!("Reaction users limit clamped to 100! (API Restriction)");
        }

        http.as_ref()
            .get_reaction_users(
                self.channel_id,
                self.message_id,
                reaction_type,
                limit,
                after.map(UserId::get),
            )
            .await
            .map_err(From::from)
    }
}
