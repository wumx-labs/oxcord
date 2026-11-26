//! Models relating to Discord channels.

use crate::model::prelude::*;
use crate::model::utils::{StrOrInt, deserialize_components, discord_colours};

/// A representation of a message over a guild's text channel, a group, or a private channel.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#message-object) with some
/// [extra fields](https://docs.discord.com/developers/events/gateway-events#message-create-message-create-extra-fields).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Message {
    /// The unique Id of the message. Can be used to calculate the creation date of the message.
    pub id: MessageId,
    /// The Id of the [`Channel`] that the message was sent to.
    pub channel_id: ChannelId,
    /// The user that sent the message.
    pub author: User,
    /// The content of the message.
    pub content: String,
    /// Initial message creation timestamp, calculated from its Id.
    pub timestamp: Timestamp,
    /// The timestamp of the last time the message was updated, if it was.
    pub edited_timestamp: Option<Timestamp>,
    /// Indicator of whether the command is to be played back via text-to-speech.
    ///
    /// In the client, this is done via the `/tts` slash command.
    pub tts: bool,
    /// Indicator of whether the message mentions everyone.
    pub mention_everyone: bool,
    /// Array of users mentioned in the message.
    pub mentions: Vec<User>,
    /// Array of [`Role`]s' Ids mentioned in the message.
    pub mention_roles: Vec<RoleId>,
    /// Channels specifically mentioned in this message.
    ///
    /// **Note**: Not all channel mentions in a message will appear in [`Self::mention_channels`].
    /// Only textual channels that are visible to everyone in a lurkable guild will ever be
    /// included.
    ///
    /// A lurkable guild is one that allows users to read public channels in a server without
    /// actually joining the server. It also allows users to look at these channels without being
    /// logged in to Discord.
    ///
    /// Only crossposted messages (via Channel Following) currently include
    /// [`Self::mention_channels`] at all. If no mentions in the message meet these requirements,
    /// this field will not be sent.
    ///
    /// [Refer to Discord's documentation for more information][discord-docs].
    ///
    /// [discord-docs]: https://docs.discord.com/developers/resources/message#message-object
    #[serde(default = "Vec::new")]
    pub mention_channels: Vec<ChannelMention>,
    /// An vector of the files attached to a message.
    pub attachments: Vec<Attachment>,
    /// Array of embeds sent with the message.
    pub embeds: Vec<Embed>,
    /// Array of reactions performed on the message.
    #[serde(default)]
    pub reactions: Vec<MessageReaction>,
    /// Non-repeating number used for ensuring message order.
    #[serde(default)]
    pub nonce: Option<Nonce>,
    /// Indicator of whether the message is pinned.
    pub pinned: bool,
    /// The Id of the webhook that sent this message, if one did.
    pub webhook_id: Option<WebhookId>,
    /// Indicator of the type of message this is, i.e. whether it is a regular message or a system
    /// message.
    #[serde(rename = "type")]
    pub kind: MessageType,
    /// Sent with Rich Presence-related chat embeds.
    pub activity: Option<MessageActivity>,
    /// Sent with Rich Presence-related chat embeds.
    pub application: Option<MessageApplication>,
    /// If the message is an Interaction or application-owned webhook, this is the id of the
    /// application.
    pub application_id: Option<ApplicationId>,
    /// Reference data sent with crossposted messages.
    pub message_reference: Option<MessageReference>,
    /// Bit flags describing extra features of the message.
    pub flags: Option<MessageFlags>,
    /// The message that was replied to using this message.
    pub referenced_message: Option<Box<Message>>, // Boxed to avoid recursion
    /// An array of message snapshots, known as forwarded messages.
    #[serde(default, deserialize_with = "deserialize_snapshots")]
    pub message_snapshots: Vec<MessageSnapshot>,
    /// Sent if the message is a response to an [`Interaction`].
    ///
    /// [`Interaction`]: crate::model::application::Interaction
    pub interaction_metadata: Option<Box<MessageInteractionMetadata>>,
    /// The thread that was started from this message, includes thread member object.
    pub thread: Option<GuildChannel>,
    /// The components of this message
    #[serde(default, deserialize_with = "deserialize_components")]
    pub components: Vec<ActionRow>,
    /// Array of message sticker item objects.
    #[serde(default)]
    pub sticker_items: Vec<StickerItem>,
    /// A generally increasing integer (there may be gaps or duplicates) that represents the
    /// approximate position of the message in a thread, it can be used to estimate the relative
    /// position of the message in a thread in company with total_message_sent on parent thread.
    pub position: Option<u64>,
    /// Data of the role subscription purchase or renewal that prompted this
    /// [`MessageType::RoleSubscriptionPurchase`] message.
    pub role_subscription_data: Option<RoleSubscriptionData>,
    // Field omitted: stickers (it's deprecated by Discord)
    /// The Id of the [`Guild`] that the message was sent in. This value will only be present if
    /// this message was received over the gateway, therefore **do not use this to check if message
    /// is in DMs**, it is not a reliable method.
    // TODO: maybe introduce an `enum MessageLocation { Dm, Guild(GuildId) }` and store
    // `Option<MessageLocation` here. Instead of None being ambiguous (is it in DMs? Or do we just
    // not know because HTTP retrieved Messages don't have guild ID?), we'd set
    // Some(MessageLocation::Dm) in gateway and None in HTTP.
    pub guild_id: Option<GuildId>,
    /// A partial amount of data about the user's member data
    ///
    /// Only present in [`MessageCreateEvent`].
    pub member: Option<Box<PartialMember>>,
    /// A poll that may be attached to a message.
    ///
    /// This is often omitted, so is boxed to improve memory usage.
    ///
    /// Only present in [`MessageCreateEvent`].
    pub poll: Option<Box<Poll>>,
}

impl AsRef<MessageId> for Message {
    fn as_ref(&self) -> &MessageId {
        &self.id
    }
}

impl From<Message> for MessageId {
    /// Gets the Id of a [`Message`].
    fn from(message: Message) -> MessageId {
        message.id
    }
}

impl From<&Message> for MessageId {
    /// Gets the Id of a [`Message`].
    fn from(message: &Message) -> MessageId {
        message.id
    }
}

/// A representation of a reaction to a message.
///
/// Multiple of the same [reaction type] are sent into one [`MessageReaction`], with an associated
/// [`Self::count`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#reaction-object).
///
/// [reaction type]: ReactionType
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageReaction {
    /// The amount of the type of reaction that have been sent for the associated message
    /// including super reactions.
    pub count: u64,
    /// A breakdown of what reactions were from regular reactions and super reactions.
    pub count_details: CountDetails,
    /// Indicator of whether the current user has sent this type of reaction.
    pub me: bool,
    /// Indicator of whether the current user has sent the type of super-reaction.
    pub me_burst: bool,
    /// The type of reaction.
    #[serde(rename = "emoji")]
    pub reaction_type: ReactionType,
    // The colours used for super reactions.
    #[serde(rename = "burst_colors", deserialize_with = "discord_colours")]
    pub burst_colours: Vec<Colour>,
}

/// A representation of reaction count details.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#reaction-count-details-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CountDetails {
    pub burst: u64,
    pub normal: u64,
}

enum_number! {
    /// Differentiates between regular and different types of system messages.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/message#message-object-message-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum MessageType {
        /// A regular message.
        #[default]
        Regular = 0,
        /// An indicator that a recipient was added by the author.
        GroupRecipientAddition = 1,
        /// An indicator that a recipient was removed by the author.
        GroupRecipientRemoval = 2,
        /// An indicator that a call was started by the author.
        GroupCallCreation = 3,
        /// An indicator that the group name was modified by the author.
        GroupNameUpdate = 4,
        /// An indicator that the group icon was modified by the author.
        GroupIconUpdate = 5,
        /// An indicator that a message was pinned by the author.
        PinsAdd = 6,
        /// An indicator that a member joined the guild.
        MemberJoin = 7,
        /// An indicator that someone has boosted the guild.
        NitroBoost = 8,
        /// An indicator that the guild has reached nitro tier 1
        NitroTier1 = 9,
        /// An indicator that the guild has reached nitro tier 2
        NitroTier2 = 10,
        /// An indicator that the guild has reached nitro tier 3
        NitroTier3 = 11,
        /// An indicator that the channel is now following a news channel
        ChannelFollowAdd = 12,
        /// An indicator that the guild is disqualified for Discovery Feature
        GuildDiscoveryDisqualified = 14,
        /// An indicator that the guild is requalified for Discovery Feature
        GuildDiscoveryRequalified = 15,
        /// The first warning before guild discovery removal.
        GuildDiscoveryGracePeriodInitialWarning = 16,
        /// The last warning before guild discovery removal.
        GuildDiscoveryGracePeriodFinalWarning = 17,
        /// Message sent to inform users that a thread was created.
        ThreadCreated = 18,
        /// A message reply.
        InlineReply = 19,
        /// A slash command.
        ChatInputCommand = 20,
        /// A thread start message.
        ThreadStarterMessage = 21,
        /// Server setup tips.
        GuildInviteReminder = 22,
        /// A context menu command.
        ContextMenuCommand = 23,
        /// A message from an auto moderation action.
        AutoModAction = 24,
        RoleSubscriptionPurchase = 25,
        InteractionPremiumUpsell = 26,
        StageStart = 27,
        StageEnd = 28,
        StageSpeaker = 29,
        StageTopic = 31,
        GuildApplicationPremiumSubscription = 32,
        GuildIncidentAlertModeEnabled = 36,
        GuildIncidentAlertModeDisabled = 37,
        GuildIncidentReportRaid = 38,
        GuildIncidentReportFalseAlarm = 39,
        PurchaseNotification = 44,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// [Discord docs](https://docs.discord.com/developers/resources/message#message-object-message-activity-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum MessageActivityKind {
        Join = 1,
        Spectate = 2,
        Listen = 3,
        JoinRequest = 5,
        _ => Unknown(u8),
    }
}

/// Rich Presence application information.
///
/// [Discord docs](https://docs.discord.com/developers/resources/application#application-object),
/// [subset undocumented](https://docs.discord.com/developers/resources/message#message-object-message-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageApplication {
    /// ID of the application.
    pub id: ApplicationId,
    /// ID of the embed's image asset.
    pub cover_image: Option<ImageHash>,
    /// Application's description.
    pub description: String,
    /// ID of the application's icon.
    pub icon: Option<ImageHash>,
    /// Name of the application.
    pub name: String,
}

/// Rich Presence activity information.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#message-object-message-activity-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageActivity {
    /// Kind of message activity.
    #[serde(rename = "type")]
    pub kind: MessageActivityKind,
    /// `party_id` from a Rich Presence event.
    pub party_id: Option<String>,
}

enum_number! {
    /// Message Reference Type information
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/message#message-reference-types)
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum MessageReferenceKind {
        #[default]
        Default = 0,
        Forward = 1,
        _ => Unknown(u8),
    }
}

/// Reference data sent with crossposted messages.
///
/// [Discord docs](https://docs.discord.com/developers/resources/message#message-reference-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageReference {
    /// The Type of Message Reference
    #[serde(rename = "type", default = "MessageReferenceKind::default")]
    pub kind: MessageReferenceKind,
    /// ID of the originating message.
    pub message_id: Option<MessageId>,
    /// ID of the originating message's channel.
    pub channel_id: ChannelId,
    /// ID of the originating message's guild.
    pub guild_id: Option<GuildId>,
    /// When sending, whether to error if the referenced message doesn't exist instead of sending
    /// as a normal (non-reply) message, default true.
    pub fail_if_not_exists: Option<bool>,
}

impl MessageReference {
    #[must_use]
    pub fn new(kind: MessageReferenceKind, channel_id: ChannelId) -> Self {
        Self {
            kind,
            channel_id,
            message_id: None,
            guild_id: None,
            fail_if_not_exists: None,
        }
    }

    #[must_use]
    pub fn message_id(mut self, message_id: MessageId) -> Self {
        self.message_id = Some(message_id);
        self
    }

    #[must_use]
    pub fn guild_id(mut self, guild_id: GuildId) -> Self {
        self.guild_id = Some(guild_id);
        self
    }

    #[must_use]
    pub fn fail_if_not_exists(mut self, fail_if_not_exists: bool) -> Self {
        self.fail_if_not_exists = Some(fail_if_not_exists);
        self
    }
}

impl From<&Message> for MessageReference {
    fn from(m: &Message) -> Self {
        Self {
            kind: MessageReferenceKind::default(),
            message_id: Some(m.id),
            channel_id: m.channel_id,
            guild_id: m.guild_id,
            fail_if_not_exists: None,
        }
    }
}

impl From<(ChannelId, MessageId)> for MessageReference {
    // TODO(next): Remove this
    fn from(pair: (ChannelId, MessageId)) -> Self {
        Self {
            kind: MessageReferenceKind::default(),
            message_id: Some(pair.1),
            channel_id: pair.0,
            guild_id: None,
            fail_if_not_exists: None,
        }
    }
}

/// [Discord docs](https://docs.discord.com/developers/resources/message#channel-mention-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChannelMention {
    /// ID of the channel.
    pub id: ChannelId,
    /// ID of the guild containing the channel.
    pub guild_id: GuildId,
    /// The kind of channel
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The name of the channel
    pub name: String,
}

/// [Discord docs](https://docs.discord.com/developers/resources/message#message-snapshot-structure)
///
/// For field documentation, see [`Message`].
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MessageSnapshot {
    pub content: String,
    pub timestamp: Timestamp,
    pub edited_timestamp: Option<Timestamp>,
    pub mentions: Vec<User>,
    #[serde(default)]
    pub mention_roles: Vec<RoleId>,
    pub attachments: Vec<Attachment>,
    pub embeds: Vec<Embed>,
    #[serde(rename = "type")]
    pub kind: MessageType,
    pub flags: Option<MessageFlags>,
    #[serde(default, deserialize_with = "deserialize_components")]
    pub components: Vec<ActionRow>,
    #[serde(default)]
    pub sticker_items: Vec<StickerItem>,
}

/// Custom deserialization function to handle the nested "message" field
pub(crate) fn deserialize_snapshots<'de, D>(
    deserializer: D,
) -> Result<Vec<MessageSnapshot>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct MessageSnapshotWrapper {
        pub message: MessageSnapshot,
    }

    let snapshots: Vec<MessageSnapshotWrapper> = Deserialize::deserialize(deserializer)?;

    let result = snapshots.into_iter().map(|wrapper| wrapper.message).collect();

    Ok(result)
}

bitflags! {
    /// Describes extra features of the message.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/message#message-object-message-flags).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct MessageFlags: u64 {
        /// This message has been published to subscribed channels (via Channel Following).
        const CROSSPOSTED = 1 << 0;
        /// This message originated from a message in another channel (via Channel Following).
        const IS_CROSSPOST = 1 << 1;
        /// Do not include any embeds when serializing this message.
        const SUPPRESS_EMBEDS = 1 << 2;
        /// The source message for this crosspost has been deleted (via Channel Following).
        const SOURCE_MESSAGE_DELETED = 1 << 3;
        /// This message came from the urgent message system.
        const URGENT = 1 << 4;
        /// This message has an associated thread, with the same id as the message.
        const HAS_THREAD = 1 << 5;
        /// This message is only visible to the user who invoked the Interaction.
        const EPHEMERAL = 1 << 6;
        /// This message is an Interaction Response and the bot is "thinking".
        const LOADING = 1 << 7;
        /// This message failed to mention some roles and add their members to the thread.
        const FAILED_TO_MENTION_SOME_ROLES_IN_THREAD = 1 << 8;
        /// This message will not trigger push and desktop notifications.
        const SUPPRESS_NOTIFICATIONS = 1 << 12;
        /// This message is a voice message.
        ///
        /// Voice messages have the following properties:
        /// - They cannot be edited.
        /// - Only a single audio attachment is allowed. No content, stickers, etc...
        /// - The [`Attachment`] has additional fields: `duration_secs` and `waveform`.
        ///
        /// As of 2023-04-14, clients upload a 1 channel, 48000 Hz, 32kbps Opus stream in an OGG container.
        /// The encoding is a Discord implementation detail and may change without warning or documentation.
        ///
        /// As of 2023-04-20, bots are currently not able to send voice messages
        /// ([source](https://github.com/discord/discord-api-docs/pull/6082)).
        const IS_VOICE_MESSAGE = 1 << 13;
        /// This message contains Components V2 (e.g. Container, Section, TextDisplay, etc.).
        const IS_COMPONENTS_V2 = 1 << 15;
    }
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Nonce {
    String(String),
    Number(u64),
}

impl<'de> serde::Deserialize<'de> for Nonce {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(StrOrInt::deserialize(deserializer)?.into_enum(Self::String, Self::Number))
    }
}

/// [Discord docs](https://docs.discord.com/developers/resources/message#role-subscription-data-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoleSubscriptionData {
    /// The id of the sku and listing that the user is subscribed to.
    pub role_subscription_listing_id: SkuId,
    /// The name of the tier that the user is subscribed to.
    pub tier_name: String,
    /// The cumulative number of months that the user has been subscribed for.
    pub total_months_subscribed: u16,
    /// Whether this notification is for a renewal rather than a new purchase.
    pub is_renewal: bool,
}

/// A poll that has been attached to a [`Message`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/poll#poll-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Poll {
    pub question: PollMedia,
    pub answers: Vec<PollAnswer>,
    pub expiry: Option<Timestamp>,
    pub allow_multiselect: bool,
    pub layout_type: PollLayoutType,
    /// The results of the Poll.
    ///
    /// None does **not** mean that there are no results, simply that Discord has not provide them.
    /// See the discord docs for a more detailed explaination.
    pub results: Option<PollResults>,
}

/// A piece of data used in mutliple parts of the [`Poll`] structure.
///
/// Currently holds text and an optional emoji, but this is expected to change in future
///
/// [Discord docs](https://docs.discord.com/developers/resources/poll#poll-media-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollMedia {
    pub text: Option<String>,
    pub emoji: Option<PollMediaEmoji>,
}

/// The "Partial Emoji" attached to a [`PollMedia`] model.
///
/// [Discord docs](https://docs.discord.com/developers/resources/poll#poll-media-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PollMediaEmoji {
    Name(String),
    Id(EmojiId),
}

impl<'de> serde::Deserialize<'de> for PollMediaEmoji {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct RawPollMediaEmoji {
            name: Option<String>,
            id: Option<EmojiId>,
        }

        let raw = RawPollMediaEmoji::deserialize(deserializer)?;
        if let Some(name) = raw.name {
            Ok(PollMediaEmoji::Name(name))
        } else if let Some(id) = raw.id {
            Ok(PollMediaEmoji::Id(id))
        } else {
            Err(serde::de::Error::duplicate_field("emoji"))
        }
    }
}

impl From<String> for PollMediaEmoji {
    fn from(value: String) -> Self {
        Self::Name(value)
    }
}

impl From<EmojiId> for PollMediaEmoji {
    fn from(value: EmojiId) -> Self {
        Self::Id(value)
    }
}

/// A possible answer for a [`Poll`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/poll#poll-answer-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollAnswer {
    pub answer_id: AnswerId,
    pub poll_media: PollMedia,
}

enum_number! {
    /// Represents the different layouts that a [`Poll`] may have.
    ///
    /// Currently, there is only the one option.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/poll#layout-type)
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum PollLayoutType {
        #[default]
        Default = 1,
        _ => Unknown(u8),
    }
}

/// The model for the results of a [`Poll`].
///
/// If `is_finalized` is `false`, `answer_counts` will be inaccurate due to Discord's scale.
///
/// [Discord docs](https://docs.discord.com/developers/resources/poll#poll-results-object-poll-results-object-structure)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollResults {
    pub is_finalized: bool,
    pub answer_counts: Vec<PollAnswerCount>,
}

/// The count of a single [`PollAnswer`]'s results.
///
/// [Discord docs](https://docs.discord.com/developers/resources/poll#poll-answer-object-poll-answer-object-structure)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollAnswerCount {
    pub id: AnswerId,
    pub count: u64,
    pub me_voted: bool,
}
