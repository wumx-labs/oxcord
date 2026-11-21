#[cfg(feature = "model")]
use crate::builder::{
    Builder,
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
    EditInteractionResponse,
};
#[cfg(feature = "collector")]
use crate::client::Context;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;
#[cfg(all(feature = "collector", feature = "utils"))]
use crate::utils::{CreateQuickModal, QuickModalResponse};
#[allow(async_fn_in_trait)]
pub trait ComponentInteractionExt: Sized {
    #[cfg(feature = "model")]
    /// Gets the interaction response.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if there is no interaction response.
    async fn get_response(&self, http: impl AsRef<Http>) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Creates a response to the interaction received.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long. May also return an
    /// [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is an error in
    /// deserializing the API response.
    async fn create_response(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInteractionResponse,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits the initial interaction response.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long. May also return an
    /// [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is an error in
    /// deserializing the API response.
    async fn edit_response(
        &self,
        cache_http: impl CacheHttp,
        builder: EditInteractionResponse,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Deletes the initial interaction response.
    ///
    /// Does not work on ephemeral messages.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error. Such as if the response was already
    /// deleted.
    async fn delete_response(&self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Creates a followup response to the response sent.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the content is too long. May also return [`Error::Http`] if the
    /// API returns an error, or [`Error::Json`] if there is an error in deserializing the
    /// response.
    async fn create_followup(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInteractionResponseFollowup,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Edits a followup response to the response sent.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the content is too long. May also return [`Error::Http`] if the
    /// API returns an error, or [`Error::Json`] if there is an error in deserializing the
    /// response.
    async fn edit_followup(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
        builder: CreateInteractionResponseFollowup,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Deletes a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error. Such as if the response was already
    /// deleted.
    async fn delete_followup<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Gets a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error. Such as if the response was
    /// deleted.
    async fn get_followup<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<Message>;
    #[cfg(feature = "model")]
    /// Helper function to defer an interaction.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is
    /// an error in deserializing the API response.
    async fn defer(&self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// Helper function to defer an interaction ephemerally
    ///
    /// # Errors
    ///
    /// May also return an [`Error::Http`] if the API returns an error, or an [`Error::Json`] if
    /// there is an error in deserializing the API response.
    async fn defer_ephemeral(&self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// See [`CreateQuickModal`].
    ///
    /// # Errors
    ///
    /// See [`CreateQuickModal::execute()`].
    #[cfg(all(feature = "collector", feature = "utils"))]
    async fn quick_modal(
        &self,
        ctx: &Context,
        builder: CreateQuickModal,
    ) -> Result<Option<QuickModalResponse>>;
}
impl ComponentInteractionExt for ComponentInteraction {
    #[cfg(feature = "model")]
    async fn get_response(&self, http: impl AsRef<Http>) -> Result<Message> {
        http.as_ref().get_original_interaction_response(&self.token).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn create_response(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInteractionResponse,
    ) -> Result<()> {
        builder.execute(cache_http, (self.id, &self.token)).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn edit_response(
        &self,
        cache_http: impl CacheHttp,
        builder: EditInteractionResponse,
    ) -> Result<Message> {
        builder.execute(cache_http, &self.token).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn delete_response(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_original_interaction_response(&self.token).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn create_followup(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInteractionResponseFollowup,
    ) -> Result<Message> {
        builder.execute(cache_http, (None, &self.token)).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn edit_followup(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
        builder: CreateInteractionResponseFollowup,
    ) -> Result<Message> {
        builder
            .execute(cache_http, (Some(message_id.into()), &self.token))
            .await
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn delete_followup<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<()> {
        http.as_ref()
            .delete_followup_message(&self.token, message_id.into())
            .await
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn get_followup<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<Message> {
        http.as_ref().get_followup_message(&self.token, message_id.into()).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn defer(&self, cache_http: impl CacheHttp) -> Result<()> {
        self.create_response(cache_http, CreateInteractionResponse::Acknowledge).await
    }
    #[cfg(feature = "model")]
    async fn defer_ephemeral(&self, cache_http: impl CacheHttp) -> Result<()> {
        let builder = CreateInteractionResponse::Defer(
            CreateInteractionResponseMessage::new().ephemeral(true),
        );
        self.create_response(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    #[cfg(all(feature = "collector", feature = "utils"))]
    async fn quick_modal(
        &self,
        ctx: &Context,
        builder: CreateQuickModal,
    ) -> Result<Option<QuickModalResponse>> {
        builder.execute(ctx, self.id, &self.token).await
    }
}
