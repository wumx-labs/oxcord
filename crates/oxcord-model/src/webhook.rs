//! Webhook model and implementations.

use secrecy::SecretString;

use super::utils::secret;
use crate::model::prelude::*;

enum_number! {
    /// A representation of a type of webhook.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/webhook#webhook-object-webhook-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum WebhookType {
        /// An indicator that the webhook can post messages to channels with a token.
        Incoming = 1,
        /// An indicator that the webhook is managed by Discord for posting new messages to
        /// channels without a token.
        ChannelFollower = 2,
        /// Application webhooks are webhooks used with Interactions.
        Application = 3,
        _ => Unknown(u8),
    }
}

impl WebhookType {
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Incoming => "incoming",
            Self::ChannelFollower => "channel follower",
            Self::Application => "application",
            Self::Unknown(_) => "unknown",
        }
    }
}

/// A representation of a webhook, which is a low-effort way to post messages to channels. They do
/// not necessarily require a bot user or authentication to use.
///
/// [Discord docs](https://docs.discord.com/developers/resources/webhook#webhook-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Webhook {
    /// The unique Id.
    ///
    /// Can be used to calculate the creation date of the webhook.
    pub id: WebhookId,
    /// The type of the webhook.
    #[serde(rename = "type")]
    pub kind: WebhookType,
    /// The Id of the guild that owns the webhook.
    pub guild_id: Option<GuildId>,
    /// The Id of the channel that owns the webhook.
    pub channel_id: Option<ChannelId>,
    /// The user that created the webhook.
    ///
    /// **Note**: This is not received when getting a webhook by its token.
    pub user: Option<User>,
    /// The default name of the webhook.
    ///
    /// This can be temporarily overridden via [`ExecuteWebhook::username`].
    pub name: Option<String>,
    /// The default avatar.
    ///
    /// This can be temporarily overridden via [`ExecuteWebhook::avatar_url`].
    pub avatar: Option<ImageHash>,
    /// The webhook's secure token.
    #[serde(with = "secret", default)]
    pub token: Option<SecretString>,
    /// The bot/OAuth2 application that created this webhook.
    pub application_id: Option<ApplicationId>,
    /// The guild of the channel that this webhook is following (returned for
    /// [`WebhookType::ChannelFollower`])
    pub source_guild: Option<WebhookGuild>,
    /// The channel that this webhook is following (returned for
    /// [`WebhookType::ChannelFollower`]).
    pub source_channel: Option<WebhookChannel>,
    /// The url used for executing the webhook (returned by the webhooks OAuth2 flow).
    #[serde(with = "secret", default)]
    pub url: Option<SecretString>,
}

/// The guild object returned by a [`Webhook`], of type [`WebhookType::ChannelFollower`].
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookGuild {
    /// The unique Id identifying the guild.
    pub id: GuildId,
    /// The name of the guild.
    pub name: String,
    /// The hash of the icon used by the guild.
    ///
    /// In the client, this appears on the guild list on the left-hand side.
    pub icon: Option<ImageHash>,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookChannel {
    /// The unique Id of the channel.
    pub id: ChannelId,
    /// The name of the channel.
    pub name: String,
}
