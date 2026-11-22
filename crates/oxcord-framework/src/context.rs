//! The unified command context, covering both slash and prefix invocations.

use std::sync::Arc;

use oxcord_model::id::{ChannelId, GuildId, UserId};
use oxcord_model::user::User;

use crate::error::FrameworkError;

/// Raw invocation data for the current command.
#[derive(Debug, Clone)]
pub enum InvocationData {
    /// Slash (application) command interaction token and data.
    Slash {
        /// The interaction token, used to reply.
        token: String,
        /// Guild the interaction was triggered in, if any.
        guild_id: Option<GuildId>,
    },
    /// Prefix (text) command message content.
    Prefix {
        /// The raw message content after the command name.
        args: String,
        /// Guild the message was sent in, if any.
        guild_id: Option<GuildId>,
    },
}

/// A lightweight HTTP/interaction client handle passed into command handlers.
///
/// In a full Oxcord integration this would wrap `Arc<Http>` and an optional
/// `Arc<Cache>`. We keep it generic here so the framework crate does not
/// depend on the entire `oxcord` facade.
#[derive(Clone)]
pub struct FrameworkHttp {
    /// Interaction token for slash commands, or `None` for prefix.
    pub(crate) interaction_token: Option<String>,
    /// Application ID, needed to follow up interactions.
    pub(crate) application_id: u64,
    /// Inner HTTP handle (type-erased to avoid a full `oxcord` dep).
    pub(crate) channel_id: ChannelId,
}

/// The unified context passed to every command handler.
///
/// Regardless of whether the command was invoked as a slash command or a prefix
/// message, all handlers receive a `Context<'_>`. Methods like [`Self::say`]
/// automatically pick the right Discord API path.
#[derive(Clone)]
pub struct Context<'a> {
    /// The user who invoked the command.
    pub author: &'a User,
    /// Channel the command was invoked in.
    pub channel_id: ChannelId,
    /// Guild the command was invoked in, if any.
    pub guild_id: Option<GuildId>,
    /// Raw invocation data (slash token or prefix args).
    pub invocation: &'a InvocationData,
    /// A lightweight handle used by helper methods.
    pub(crate) http: Arc<FrameworkHttp>,
}

impl<'a> Context<'a> {
    /// Reply to the user with a plain text message.
    ///
    /// For slash commands this sends an interaction response; for prefix
    /// commands it sends a regular channel message.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Http`] if the Discord API call fails.
    pub async fn say(&self, content: impl Into<String>) -> Result<(), FrameworkError> {
        let content = content.into();
        match self.invocation {
            InvocationData::Slash {
                token, ..
            } => {
                tracing::debug!(token = %token, "slash reply: {}", content);
                // In a real integration: http.create_interaction_response(token, ...).await
                Ok(())
            },
            InvocationData::Prefix {
                ..
            } => {
                tracing::debug!(channel_id = %self.channel_id, "prefix reply: {}", content);
                // In a real integration: http.send_message(channel_id, content).await
                Ok(())
            },
        }
    }

    /// Return the invoking user's ID.
    #[must_use]
    pub fn author_id(&self) -> UserId {
        self.author.id
    }
}

impl std::fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("author", &self.author.id)
            .field("channel_id", &self.channel_id)
            .field("guild_id", &self.guild_id)
            .finish_non_exhaustive()
    }
}
