#[cfg(feature = "model")]
use secrecy::ExposeSecret;

#[cfg(feature = "model")]
use crate::builder::{Builder, EditWebhook, EditWebhookMessage, ExecuteWebhook};
#[cfg(feature = "cache")]
use crate::cache::{Cache, GuildChannelRef, GuildRef};
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait WebhookChannelExt: Sized {
    #[cfg(feature = "model")]
    /// Attempts to find a [`GuildChannel`] by its Id in the cache.
    #[cfg(feature = "cache")]
    #[deprecated(since = "0.0.1", note = "Use Cache::guild and Guild::channels")]
    fn to_channel_cached(self, cache: &Cache) -> Option<GuildChannelRef<'_>>;
    #[cfg(feature = "model")]
    /// First attempts to retrieve the channel from the `temp_cache` if enabled, otherwise performs
    /// a HTTP request.
    ///
    /// It is recommended to first check if the channel is accessible via `Cache::guild` and
    /// `Guild::members`, although this requires a `GuildId`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the channel retrieval request failed.
    async fn to_channel(self, cache_http: impl CacheHttp) -> Result<GuildChannel>;
}
impl WebhookChannelExt for WebhookChannel {
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    #[inline]
    fn to_channel_cached(self, cache: &Cache) -> Option<GuildChannelRef<'_>> {
        #[allow(deprecated)]
        cache.channel(self.id)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn to_channel(self, cache_http: impl CacheHttp) -> Result<GuildChannel> {
        let channel = self.id.to_channel(cache_http).await?;
        channel.guild().ok_or(Error::Model(ModelError::InvalidChannelType))
    }
}
#[allow(async_fn_in_trait)]
pub trait WebhookExt: Sized {
    #[cfg(feature = "model")]
    /// Retrieves a webhook given its Id.
    ///
    /// This method requires authentication, whereas [`Webhook::from_id_with_token`] and
    /// [`Webhook::from_url`] do not.
    ///
    /// # Examples
    ///
    /// Retrieve a webhook by Id:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::http::Http;
    /// # use oxcord::model::{webhook::Webhook, id::WebhookId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let id = WebhookId::new(245037420704169985);
    /// let webhook = Webhook::from_id(&http, id).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not authenticated, or if the webhook does
    /// not exist.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserialising Discord's
    /// response.
    async fn from_id(http: impl AsRef<Http>, webhook_id: impl Into<WebhookId>) -> Result<Self>;
    #[cfg(feature = "model")]
    /// Retrieves a webhook given its Id and unique token.
    ///
    /// This method does _not_ require authentication.
    ///
    /// # Examples
    ///
    /// Retrieve a webhook by Id and its unique token:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::http::Http;
    /// # use oxcord::model::{webhook::Webhook, id::WebhookId};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let id = WebhookId::new(245037420704169985);
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let webhook = Webhook::from_id_with_token(&http, id, token).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the webhook does not exist, or if the token is invalid.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserialising Discord's
    /// response.
    async fn from_id_with_token(
        http: impl AsRef<Http>,
        webhook_id: impl Into<WebhookId>,
        token: &str,
    ) -> Result<Self>;
    #[cfg(feature = "model")]
    /// Retrieves a webhook given its url.
    ///
    /// This method does _not_ require authentication.
    ///
    /// # Examples
    ///
    /// Retrieve a webhook by url:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::http::Http;
    /// # use oxcord::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let webhook = Webhook::from_url(&http, url).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the url is malformed, or otherwise if the webhook does not
    /// exist, or if the token is invalid.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserialising Discord's
    /// response.
    async fn from_url(http: impl AsRef<Http>, url: &str) -> Result<Self>;
    #[cfg(feature = "model")]
    /// Deletes the webhook.
    ///
    /// If [`Self::token`] is set, then authentication is _not_ required. Otherwise, if it is
    /// [`None`], then authentication _is_ required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the webhook does not exist, the token is invalid, or if the
    /// webhook could not otherwise be deleted.
    async fn delete(&self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits the webhook.
    ///
    /// If [`Self::token`] is set, then authentication is _not_ required. Otherwise, if it is
    /// [`None`], then authentication _is_ required.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::http::Http;
    /// # use oxcord::builder::EditWebhook;
    /// # use oxcord::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let mut webhook = Webhook::from_url(&http, url).await?;
    ///
    /// let builder = EditWebhook::new().name("new name");
    /// webhook.edit(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the content is malformed, or if the token is invalid.
    ///
    /// Or may return an [`Error::Json`] if there is an error in deserialising Discord's response.
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditWebhook<'_>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Executes a webhook with the fields set via the given builder.
    ///
    /// # Examples
    ///
    /// Execute a webhook with message content of `test`:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::builder::ExecuteWebhook;
    /// # use oxcord::http::Http;
    /// # use oxcord::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let mut webhook = Webhook::from_url(&http, url).await?;
    ///
    /// let builder = ExecuteWebhook::new().content("test");
    /// webhook.execute(&http, false, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Execute a webhook with message content of `test`, overriding the username to `oxcord`,
    /// and sending an embed:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::http::Http;
    /// # use oxcord::model::webhook::Webhook;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// use oxcord::builder::{CreateEmbed, ExecuteWebhook};
    ///
    /// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let mut webhook = Webhook::from_url(&http, url).await?;
    ///
    /// let embed = CreateEmbed::new()
    ///     .title("Rust's website")
    ///     .description(
    ///         "Rust is a systems programming language that runs blazingly fast, prevents \
    ///         segfaults, and guarantees thread safety.",
    ///     )
    ///     .url("https://rust-lang.org");
    ///
    /// let builder = ExecuteWebhook::new().content("test").username("oxcord").embed(embed);
    /// webhook.execute(&http, false, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the content is malformed, or if the webhook's token
    /// is invalid.
    ///
    /// Or may return an [`Error::Json`] if there is an error deserialising Discord's response.
    async fn execute(
        &self,
        cache_http: impl CacheHttp,
        wait: bool,
        builder: ExecuteWebhook,
    ) -> Result<Option<Message>>;
    #[cfg(feature = "model")]
    /// Gets a previously sent message from the webhook.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return [`Error::Http`] if the webhook's token is invalid, or the given message Id
    /// does not belong to the current webhook.
    ///
    /// Or may return an [`Error::Json`] if there is an error deserialising Discord's response.
    async fn get_message(
        &self,
        http: impl AsRef<Http>,
        thread_id: Option<ChannelId>,
        message_id: MessageId,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Edits a webhook message with the fields set via the given builder.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if [`Self::token`] is [`None`], or if the message content is
    /// too long.
    ///
    /// May also return an [`Error::Http`] if the content is malformed, the webhook's token is
    /// invalid, or the given message Id does not belong to the current webhook.
    ///
    /// Or may return an [`Error::Json`] if there is an error deserialising Discord's response.
    async fn edit_message(
        &self,
        cache_http: impl CacheHttp,
        message_id: MessageId,
        builder: EditWebhookMessage,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Deletes a webhook message.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the webhook's token is invalid or the given message
    /// Id does not belong to the current webhook.
    async fn delete_message(
        &self,
        http: impl AsRef<Http>,
        thread_id: Option<ChannelId>,
        message_id: MessageId,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Retrieves the latest information about the webhook, editing the webhook in-place.
    ///
    /// As this calls the [`Http::get_webhook_with_token`] function, authentication is not
    /// required.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    ///
    /// May also return an [`Error::Http`] if the http client errors or if Discord returns an
    /// error. Such as if the [`Webhook`] was deleted.
    ///
    /// Or may return an [`Error::Json`] if there is an error deserialising Discord's response.
    async fn refresh(&mut self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Returns the url of the webhook.
    ///
    /// ```rust,ignore
    /// assert_eq!(hook.url(), "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV")
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the [`Self::token`] is [`None`].
    fn url(&self) -> Result<String>;
}
impl WebhookExt for Webhook {
    #[cfg(feature = "model")]
    async fn from_id(http: impl AsRef<Http>, webhook_id: impl Into<WebhookId>) -> Result<Self> {
        http.as_ref().get_webhook(webhook_id.into()).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn from_id_with_token(
        http: impl AsRef<Http>,
        webhook_id: impl Into<WebhookId>,
        token: &str,
    ) -> Result<Self> {
        http.as_ref().get_webhook_with_token(webhook_id.into(), token).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn from_url(http: impl AsRef<Http>, url: &str) -> Result<Self> {
        http.as_ref().get_webhook_from_url(url).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete(&self, http: impl AsRef<Http>) -> Result<()> {
        let http = http.as_ref();
        match &self.token {
            Some(token) => http
                .delete_webhook_with_token(self.id, token.expose_secret(), None)
                .await
                .map_err(From::from),
            None => http.delete_webhook(self.id, None).await.map_err(From::from),
        }
    }
    #[cfg(feature = "model")]
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditWebhook<'_>) -> Result<()> {
        let token = self.token.as_ref().map(ExposeSecret::expose_secret).map(String::as_str);
        *self = builder.execute(cache_http, (self.id, token)).await?;
        Ok(())
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn execute(
        &self,
        cache_http: impl CacheHttp,
        wait: bool,
        builder: ExecuteWebhook,
    ) -> Result<Option<Message>> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?.expose_secret();
        builder.execute(cache_http, (self.id, token, wait)).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn get_message(
        &self,
        http: impl AsRef<Http>,
        thread_id: Option<ChannelId>,
        message_id: MessageId,
    ) -> Result<Message> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?.expose_secret();
        http.as_ref()
            .get_webhook_message(self.id, thread_id, token, message_id)
            .await
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn edit_message(
        &self,
        cache_http: impl CacheHttp,
        message_id: MessageId,
        builder: EditWebhookMessage,
    ) -> Result<Message> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?.expose_secret();
        builder.execute(cache_http, (self.id, token, message_id)).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn delete_message(
        &self,
        http: impl AsRef<Http>,
        thread_id: Option<ChannelId>,
        message_id: MessageId,
    ) -> Result<()> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?.expose_secret();
        http.as_ref()
            .delete_webhook_message(self.id, thread_id, token, message_id)
            .await
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn refresh(&mut self, http: impl AsRef<Http>) -> Result<()> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?.expose_secret();
        http.as_ref()
            .get_webhook_with_token(self.id, token)
            .await
            .map(|replacement| {
                *self = replacement;
            })
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    fn url(&self) -> Result<String> {
        let token = self.token.as_ref().ok_or(ModelError::NoTokenSet)?.expose_secret();
        Ok(format!("https://discord.com/api/webhooks/{}/{token}", self.id))
    }
}
#[allow(async_fn_in_trait)]
pub trait WebhookGuildExt: Sized {
    #[cfg(feature = "model")]
    /// Tries to find the [`Guild`] by its Id in the cache.
    #[cfg(feature = "cache")]
    fn to_guild_cached(self, cache: &impl AsRef<Cache>) -> Option<GuildRef<'_>>;
    #[cfg(feature = "model")]
    /// Requests [`PartialGuild`] over REST API.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send
    /// all data with a guild retrieval.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild.
    async fn to_partial_guild(self, cache_http: impl CacheHttp) -> Result<PartialGuild>;
    #[cfg(feature = "model")]
    /// Requests [`PartialGuild`] over REST API with counts.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send all data with a guild
    /// retrieval.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild.
    async fn to_partial_guild_with_counts(self, http: impl AsRef<Http>) -> Result<PartialGuild>;
}
impl WebhookGuildExt for WebhookGuild {
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    #[inline]
    fn to_guild_cached(self, cache: &impl AsRef<Cache>) -> Option<GuildRef<'_>> {
        cache.as_ref().guild(self.id)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn to_partial_guild(self, cache_http: impl CacheHttp) -> Result<PartialGuild> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = cache.guild(self.id) {
                    return Ok(guild.clone().into());
                }
            }
        }
        cache_http.http().get_guild(self.id).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn to_partial_guild_with_counts(self, http: impl AsRef<Http>) -> Result<PartialGuild> {
        http.as_ref().get_guild_with_counts(self.id).await.map_err(From::from)
    }
}
#[allow(async_fn_in_trait)]
pub trait WebhookIdExt: Sized {
    #[cfg(feature = "model")]
    /// Requests [`Webhook`] over REST API.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the http client errors or if Discord returns an error. Such
    /// as if the [`WebhookId`] does not exist.
    ///
    /// May also return an [`Error::Json`] if there is an error in deserialising the response.
    ///
    /// [Manage Webhooks]: super::permissions::Permissions::MANAGE_WEBHOOKS
    async fn to_webhook(self, http: impl AsRef<Http>) -> Result<Webhook>;
}
impl WebhookIdExt for WebhookId {
    #[cfg(feature = "model")]
    #[inline]
    async fn to_webhook(self, http: impl AsRef<Http>) -> Result<Webhook> {
        http.as_ref().get_webhook(self).await.map_err(From::from)
    }
}
