use oxcord_model::prelude::*;
use oxcord_rest::CacheHttp;

use super::Builder;
use crate::internal::prelude::*;

/// A builder to edit the status of a voice channel.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#modify-voice-channel-status).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditVoiceChannelStatus<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<Option<String>>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditVoiceChannelStatus<'a> {
    /// Creates a new default builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets or clears the status of the voice channel.
    pub fn status(mut self, status: impl Into<Option<String>>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}

#[async_trait::async_trait]
impl Builder for EditVoiceChannelStatus<'_> {
    type Context<'ctx> = ChannelId;
    type Built = ();

    /// Edits the voice channel's status.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or the request fails.
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        cache_http.http().edit_voice_status(ctx, &self, self.audit_log_reason).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_voice_channel_status_serialization() {
        let builder = EditVoiceChannelStatus::new().status(Some("Playing games".to_string()));
        let json = serde_json::to_string(&builder).unwrap();
        assert_eq!(json, r#"{"status":"Playing games"}"#);

        let clear = EditVoiceChannelStatus::new().status(None);
        let json_clear = serde_json::to_string(&clear).unwrap();
        assert_eq!(json_clear, r#"{"status":null}"#);
    }
}
