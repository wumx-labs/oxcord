use oxcord_model::constants;
use oxcord_model::prelude::*;
use oxcord_rest::CacheHttp;
use serde::Serialize;

use super::create_poll::Ready;
use super::{
    Builder,
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateComponent,
    CreateEmbed,
    CreatePoll,
    EditAttachments,
    check_overflow,
};
use crate::internal::prelude::*;

/// [Discord docs](https://docs.discord.com/developers/interactions/receiving-and-responding#create-followup-message)
#[derive(Clone, Debug, Default)]
#[must_use]
pub struct CreateInteractionResponseFollowup {
    content: Option<String>,
    tts: Option<bool>,
    embeds: Vec<CreateEmbed>,
    allowed_mentions: Option<CreateAllowedMentions>,
    components: Option<Vec<CreateActionRow>>,
    components_v2: Option<Vec<CreateComponent>>,
    flags: Option<MessageFlags>,
    poll: Option<CreatePoll<Ready>>,
    attachments: EditAttachments,
}

impl Serialize for CreateInteractionResponseFollowup {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(None)?;
        if let Some(c) = &self.content {
            map.serialize_entry("content", c)?;
        }
        if let Some(t) = self.tts {
            map.serialize_entry("tts", &t)?;
        }
        if !self.embeds.is_empty() {
            map.serialize_entry("embeds", &self.embeds)?;
        }
        if let Some(a) = &self.allowed_mentions {
            map.serialize_entry("allowed_mentions", a)?;
        }
        if let Some(v2) = &self.components_v2 {
            map.serialize_entry("components", v2)?;
        } else if let Some(v1) = &self.components {
            map.serialize_entry("components", v1)?;
        }
        if let Some(f) = self.flags {
            map.serialize_entry("flags", &f)?;
        }
        if let Some(p) = &self.poll {
            map.serialize_entry("poll", p)?;
        }
        if !self.attachments.is_empty() {
            map.serialize_entry("attachments", &self.attachments)?;
        }
        map.end()
    }
}

impl CreateInteractionResponseFollowup {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    fn check_length(&self) -> Result<()> {
        if let Some(content) = &self.content {
            check_overflow(content.chars().count(), constants::MESSAGE_CODE_LIMIT)
                .map_err(|overflow| Error::Model(ModelError::MessageTooLong(overflow)))?;
        }

        check_overflow(self.embeds.len(), constants::EMBED_MAX_COUNT)
            .map_err(|_| Error::Model(ModelError::EmbedAmount))?;
        for embed in &self.embeds {
            embed.check_length()?;
        }

        Ok(())
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(mut self, tts: bool) -> Self {
        self.tts = Some(tts);
        self
    }

    /// Appends a file to the message.
    pub fn add_file(mut self, file: CreateAttachment) -> Self {
        self.attachments = self.attachments.add(file);
        self
    }

    /// Appends a list of files to the message.
    pub fn add_files(mut self, files: impl IntoIterator<Item = CreateAttachment>) -> Self {
        for file in files {
            self.attachments = self.attachments.add(file);
        }
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list. To append files, call
    /// [`Self::add_file`] or [`Self::add_files`] instead.
    pub fn files(mut self, files: impl IntoIterator<Item = CreateAttachment>) -> Self {
        self.attachments = EditAttachments::new();
        self.add_files(files)
    }

    /// Adds an embed to the message.
    pub fn add_embed(mut self, embed: CreateEmbed) -> Self {
        self.embeds.push(embed);
        self
    }

    /// Adds multiple embeds to the message.
    pub fn add_embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds.extend(embeds);
        self
    }

    /// Sets a single embed to include in the message.
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embed`]
    /// instead.
    pub fn embed(self, embed: CreateEmbed) -> Self {
        self.embeds(vec![embed])
    }

    /// Sets a list of embeds to include in the message.
    ///
    /// Calling this multiple times will overwrite the embed list. To append embeds, call
    /// [`Self::add_embeds`] instead.
    pub fn embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = embeds;
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Sets the flags for the response.
    pub fn flags(mut self, flags: MessageFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Adds or removes the ephemeral flag
    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        let mut flags = self.flags.unwrap_or_else(MessageFlags::empty);

        if ephemeral {
            flags |= MessageFlags::EPHEMERAL;
        } else {
            flags &= !MessageFlags::EPHEMERAL;
        }

        self.flags = Some(flags);
        self
    }

    /// Adds a poll to the message. Only one poll can be added per message.
    ///
    /// See [`CreatePoll`] for more information on creating and configuring a poll.
    pub fn poll(mut self, poll: CreatePoll<Ready>) -> Self {
        self.poll = Some(poll);
        self
    }

    /// Sets the components of this message.
    pub fn components(mut self, components: Vec<CreateActionRow>) -> Self {
        self.components = Some(components);
        self
    }
    super::button_and_select_menu_convenience_methods!(self.components);

    /// Sets Components V2 for this followup, automatically setting the
    /// [`MessageFlags::IS_COMPONENTS_V2`] flag.
    pub fn components_v2(mut self, components: Vec<CreateComponent>) -> Self {
        let mut flags = self.flags.unwrap_or_default();
        flags |= MessageFlags::IS_COMPONENTS_V2;
        self.flags = Some(flags);
        self.components_v2 = Some(components);
        self
    }
}

#[async_trait::async_trait]
impl Builder for CreateInteractionResponseFollowup {
    type Context<'ctx> = (Option<MessageId>, &'ctx str);
    type Built = Message;

    /// Creates or edits a followup response to the response sent. If a [`MessageId`] is provided,
    /// then the corresponding message will be edited. Otherwise, a new message will be created.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 code points.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the content is too long. May also return [`Error::Http`] if the
    /// API returns an error, or [`Error::Json`] if there is an error in deserializing the
    /// response.
    async fn execute(
        mut self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        self.check_length()?;

        let files = self.attachments.take_files();

        let http = cache_http.http();
        if self.allowed_mentions.is_none() {
            self.allowed_mentions.clone_from(&http.default_allowed_mentions);
        }

        match ctx.0 {
            Some(id) => http.as_ref().edit_followup_message(ctx.1, id, &self, files).await,
            None => http.as_ref().create_followup_message(ctx.1, &self, files).await,
        }
    }
}
