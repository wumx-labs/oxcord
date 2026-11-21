use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait PermissionsExt: Sized {
    #[cfg(feature = "model")]
    #[must_use]
    fn dm_permissions() -> Self;
}
impl PermissionsExt for Permissions {
    #[cfg(feature = "model")]
    fn dm_permissions() -> Self {
        Self::ADD_REACTIONS
            | Self::STREAM
            | Self::VIEW_CHANNEL
            | Self::SEND_MESSAGES
            | Self::SEND_TTS_MESSAGES
            | Self::EMBED_LINKS
            | Self::ATTACH_FILES
            | Self::READ_MESSAGE_HISTORY
            | Self::MENTION_EVERYONE
            | Self::USE_EXTERNAL_EMOJIS
            | Self::CONNECT
            | Self::SPEAK
            | Self::USE_VAD
            | Self::USE_APPLICATION_COMMANDS
            | Self::USE_EXTERNAL_STICKERS
            | Self::SEND_VOICE_MESSAGES
            | Self::SEND_POLLS
            | Self::USE_EXTERNAL_APPS
    }
}
