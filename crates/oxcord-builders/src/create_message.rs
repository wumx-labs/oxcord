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

/// A builder to specify the contents of an send message request, primarily meant for use
/// through [`ChannelId::send_message`].
///
/// There are three situations where different field requirements are present:
///
/// 1. When sending a message without embeds or stickers, [`Self::content`] is the only required
///    field that is required to be set.
/// 2. When sending an [`Self::embed`], no other field is required.
/// 3. When sending stickers with [`Self::sticker_id`] or other sticker methods, no other field is
///    required.
///
/// Note that if you only need to send the content of a message, without specifying other fields,
/// then [`ChannelId::say`] may be a more preferable option.
///
/// # Examples
///
/// Sending a message with a content of `"test"` and applying text-to-speech:
///
/// ```rust,ignore
/// # use oxcord::prelude::*;
/// use oxcord::builder::{CreateEmbed, CreateMessage};
/// use oxcord::model::id::ChannelId;
/// # use oxcord::http::Http;
/// # use std::sync::Arc;
/// #
/// # async fn run() {
/// # let http: Arc<Http> = unimplemented!();
/// # let channel_id = ChannelId::new(7);
/// let embed = CreateEmbed::new().title("This is an embed").description("With a description");
/// let builder = CreateMessage::new().content("test").tts(true).embed(embed);
/// let _ = channel_id.send_message(&http, builder).await;
/// # }
/// ```
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#create-message)
#[derive(Clone, Debug, Default)]
#[must_use]
pub struct CreateMessage {
    content: Option<String>,
    nonce: Option<Nonce>,
    tts: bool,
    embeds: Vec<CreateEmbed>,
    allowed_mentions: Option<CreateAllowedMentions>,
    message_reference: Option<MessageReference>,
    components: Option<Vec<CreateActionRow>>,
    components_v2: Option<Vec<CreateComponent>>,
    sticker_ids: Vec<StickerId>,
    flags: Option<MessageFlags>,
    pub(crate) attachments: EditAttachments,
    enforce_nonce: bool,
    poll: Option<CreatePoll<super::create_poll::Ready>>,

    // The following fields are handled separately.
    reactions: Vec<ReactionType>,
}

impl Serialize for CreateMessage {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(None)?;
        if let Some(c) = &self.content {
            map.serialize_entry("content", c)?;
        }
        if let Some(n) = &self.nonce {
            map.serialize_entry("nonce", n)?;
        }
        if self.tts {
            map.serialize_entry("tts", &true)?;
        }
        if !self.embeds.is_empty() {
            map.serialize_entry("embeds", &self.embeds)?;
        }
        if let Some(a) = &self.allowed_mentions {
            map.serialize_entry("allowed_mentions", a)?;
        }
        if let Some(m) = &self.message_reference {
            map.serialize_entry("message_reference", m)?;
        }
        if let Some(v2) = &self.components_v2 {
            map.serialize_entry("components", v2)?;
        } else if let Some(v1) = &self.components {
            map.serialize_entry("components", v1)?;
        }
        if !self.sticker_ids.is_empty() {
            map.serialize_entry("sticker_ids", &self.sticker_ids)?;
        }
        if let Some(flags) = self.flags {
            map.serialize_entry("flags", &flags)?;
        }
        if self.enforce_nonce {
            map.serialize_entry("enforce_nonce", &true)?;
        }
        if let Some(poll) = &self.poll {
            map.serialize_entry("poll", poll)?;
        }
        map.end()
    }
}

impl CreateMessage {
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

        check_overflow(self.sticker_ids.len(), constants::STICKER_MAX_COUNT)
            .map_err(|_| Error::Model(ModelError::StickerAmount))?;

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

    /// Add an embed for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::embed()`] to replace existing
    /// embeds.
    pub fn add_embed(mut self, embed: CreateEmbed) -> Self {
        self.embeds.push(embed);
        self
    }

    /// Add multiple embeds for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::embeds()`] to replace existing
    /// embeds.
    pub fn add_embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds.extend(embeds);
        self
    }

    /// Set an embed for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embed()`] to keep existing
    /// embeds.
    pub fn embed(self, embed: CreateEmbed) -> Self {
        self.embeds(vec![embed])
    }

    /// Set multiple embeds for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embeds()`] to keep existing
    /// embeds.
    pub fn embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = embeds;
        self
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(mut self, tts: bool) -> Self {
        self.tts = tts;
        self
    }

    /// Adds a list of reactions to create after the message's sent.
    #[inline]
    pub fn reactions<R: Into<ReactionType>>(
        mut self,
        reactions: impl IntoIterator<Item = R>,
    ) -> Self {
        self.reactions = reactions.into_iter().map(Into::into).collect();
        self
    }

    /// Appends a file to the message.
    ///
    /// **Note**: Requires the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    pub fn add_file(mut self, file: CreateAttachment) -> Self {
        self.attachments = self.attachments.add(file);
        self
    }

    /// Appends a list of files to the message.
    ///
    /// **Note**: Requires the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
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
    ///
    /// **Note**: Requires the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    pub fn files(mut self, files: impl IntoIterator<Item = CreateAttachment>) -> Self {
        self.attachments = EditAttachments::new();
        self.add_files(files)
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Set the message this reply or forward is referring to.
    pub fn reference_message(mut self, reference: impl Into<MessageReference>) -> Self {
        self.message_reference = Some(reference.into());
        self
    }

    /// Sets the components of this message.
    pub fn components(mut self, components: Vec<CreateActionRow>) -> Self {
        self.components = Some(components);
        self
    }
    super::button_and_select_menu_convenience_methods!(self.components);

    /// Sets Components V2 for this message, automatically setting the
    /// [`MessageFlags::IS_COMPONENTS_V2`] flag.
    pub fn components_v2(mut self, components: Vec<CreateComponent>) -> Self {
        let mut flags = self.flags.unwrap_or_default();
        flags |= MessageFlags::IS_COMPONENTS_V2;
        self.flags = Some(flags);
        self.components_v2 = Some(components);
        self
    }

    /// Sets the flags for the message.
    pub fn flags(mut self, flags: MessageFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Sets a single sticker ID to include in the message.
    ///
    /// **Note**: This will replace all existing stickers. Use [`Self::add_sticker_id()`] to keep
    /// existing stickers.
    pub fn sticker_id(self, sticker_id: impl Into<StickerId>) -> Self {
        self.sticker_ids(vec![sticker_id.into()])
    }

    /// Sets a list of sticker IDs to include in the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will replace all existing stickers. Use [`Self::add_sticker_id()`] or
    /// [`Self::add_sticker_ids()`] to keep existing stickers.
    pub fn sticker_ids<T: Into<StickerId>>(
        mut self,
        sticker_ids: impl IntoIterator<Item = T>,
    ) -> Self {
        self.sticker_ids = sticker_ids.into_iter().map(Into::into).collect();
        self
    }

    /// Add a sticker ID for the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will keep all existing stickers. Use [`Self::sticker_id()`] to replace
    /// existing sticker.
    pub fn add_sticker_id(mut self, sticker_id: impl Into<StickerId>) -> Self {
        self.sticker_ids.push(sticker_id.into());
        self
    }

    /// Add multiple sticker IDs for the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will keep all existing stickers. Use [`Self::sticker_ids()`] to replace
    /// existing stickers.
    pub fn add_sticker_ids<T: Into<StickerId>>(
        mut self,
        sticker_ids: impl IntoIterator<Item = T>,
    ) -> Self {
        for sticker_id in sticker_ids {
            self = self.add_sticker_id(sticker_id);
        }
        self
    }

    /// Can be used to verify a message was sent (up to 25 characters). Value will appear in
    /// [`Message::nonce`]
    ///
    /// See [`Self::enforce_nonce`] if you would like discord to perform de-duplication.
    pub fn nonce(mut self, nonce: Nonce) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// If true and [`Self::nonce`] is provided, it will be checked for uniqueness in the past few
    /// minutes. If another message was created by the same author with the same nonce, that
    /// message will be returned and no new message will be created.
    pub fn enforce_nonce(mut self, enforce_nonce: bool) -> Self {
        self.enforce_nonce = enforce_nonce;
        self
    }

    /// Sets the [`Poll`] for this message.
    pub fn poll(mut self, poll: CreatePoll<Ready>) -> Self {
        self.poll = Some(poll);
        self
    }
}

#[async_trait::async_trait]
impl Builder for CreateMessage {
    type Context<'ctx> = (ChannelId, Option<GuildId>);
    type Built = Message;

    /// Send a message to the channel.
    ///
    /// **Note**: Requires the [Send Messages] permission. Additionally, attaching files requires
    /// the [Attach Files] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the message contents are over the above limits.
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    /// [Attach Files]: Permissions::ATTACH_FILES
    async fn execute(
        mut self,
        cache_http: impl CacheHttp,
        (channel_id, guild_id): Self::Context<'_>,
    ) -> Result<Self::Built> {
        #[cfg(feature = "cache")]
        {
            let mut req = Permissions::SEND_MESSAGES;
            if !self.attachments.is_empty() {
                req |= Permissions::ATTACH_FILES;
            }
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(cache, channel_id, req)?;
            }
        }

        self.check_length()?;

        let http = cache_http.http();

        let files = self.attachments.take_files();
        if self.allowed_mentions.is_none() {
            self.allowed_mentions.clone_from(&http.default_allowed_mentions);
        }

        #[cfg_attr(not(feature = "cache"), allow(unused_mut))]
        let mut message = http.send_message(channel_id, files, &self).await?;

        for reaction in self.reactions {
            http.create_reaction(channel_id, message.id, &reaction).await?;
        }

        // HTTP sent Messages don't have guild_id set, so we fill it in ourselves by best effort
        if message.guild_id.is_none() {
            // If we were called from GuildChannel, we can fill in the GuildId ourselves.
            message.guild_id = guild_id;
        }

        Ok(message)
    }
}
