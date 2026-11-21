use std::fmt;

use crate::model::prelude::*;

/// Represents a guild's text, news, or voice channel.
///
/// Some methods are available only for voice channels and some are only available for text
/// channels. News channels are a subset of text channels and lack slow mode hence
/// [`Self::rate_limit_per_user`] will be [`None`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GuildChannel {
    /// The unique Id of the channel.
    pub id: ChannelId,
    /// The bitrate of the channel.
    ///
    /// **Note**: This is only available for voice and stage channels.
    pub bitrate: Option<u32>,
    /// The Id of the parent category for a channel, or of the parent text channel for a thread.
    ///
    /// **Note**: This is only available for channels in a category and thread channels.
    pub parent_id: Option<ChannelId>,
    /// The Id of the guild the channel is located in.
    ///
    /// The original voice channel has an Id equal to the guild's Id, incremented by one.
    ///
    /// [`id`]: GuildChannel::id
    #[serde(default)]
    pub guild_id: GuildId,
    /// The type of the channel.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The Id of the user who created this channel
    ///
    /// **Note**: This is only available for threads and forum posts
    pub owner_id: Option<UserId>,
    /// The Id of the last message sent in the channel.
    ///
    /// **Note**: This is only available for text channels.
    pub last_message_id: Option<MessageId>,
    /// The timestamp of the time a pin was most recently made.
    ///
    /// **Note**: This is only available for text channels.
    pub last_pin_timestamp: Option<Timestamp>,
    /// The name of the channel.
    pub name: String,
    /// Permission overwrites for [`Member`]s and for [`Role`]s.
    #[serde(default)]
    pub permission_overwrites: Vec<PermissionOverwrite>,
    /// The position of the channel.
    ///
    /// The default text channel will _almost always_ have a position of `0`.
    #[serde(default)]
    pub position: u16,
    /// The topic of the channel.
    ///
    /// **Note**: This is only available for text, forum and stage channels.
    pub topic: Option<String>,
    /// The maximum number of members allowed in the channel.
    ///
    /// **Note**: This is only available for voice channels.
    pub user_limit: Option<u32>,
    /// Used to tell if the channel is not safe for work.
    // This field can or can not be present sometimes, but if it isn't default to `false`.
    #[serde(default)]
    pub nsfw: bool,
    /// A rate limit that applies per user and excludes bots.
    ///
    /// **Note**: This is only available for text channels excluding news channels.
    #[doc(alias = "slowmode")]
    #[serde(default)]
    pub rate_limit_per_user: Option<u16>,
    /// The region override.
    ///
    /// **Note**: This is only available for voice and stage channels. [`None`] for voice and stage
    /// channels means automatic region selection.
    pub rtc_region: Option<String>,
    /// The video quality mode for a voice channel.
    pub video_quality_mode: Option<VideoQualityMode>,
    /// An approximate count of messages in the thread.
    ///
    /// This is currently saturated at 255 to prevent breaking.
    ///
    /// **Note**: This is only available on thread channels.
    pub message_count: Option<u32>,
    /// An approximate count of users in a thread, stops counting at 50.
    ///
    /// **Note**: This is only available on thread channels.
    pub member_count: Option<u8>,
    /// The thread metadata.
    ///
    /// **Note**: This is only available on thread channels.
    pub thread_metadata: Option<ThreadMetadata>,
    /// Thread member object for the current user, if they have joined the thread, only included on
    /// certain API endpoints.
    pub member: Option<PartialThreadMember>,
    /// Default duration for newly created threads, in minutes, to automatically archive the thread
    /// after recent activity.
    pub default_auto_archive_duration: Option<AutoArchiveDuration>,
    /// Computed permissions for the invoking user in the channel, including overwrites.
    ///
    /// Only included inside [`CommandDataResolved`].
    pub permissions: Option<Permissions>,
    /// Extra information about the channel
    ///
    /// **Note**: This is only available in forum channels.
    #[serde(default)]
    pub flags: ChannelFlags,
    /// The number of messages ever sent in a thread, it's similar to `message_count` on message
    /// creation, but will not decrement the number when a message is deleted.
    pub total_message_sent: Option<u64>,
    /// The set of available tags.
    ///
    /// **Note**: This is only available in forum channels.
    #[serde(default)]
    pub available_tags: Vec<ForumTag>,
    /// The set of applied tags.
    ///
    /// **Note**: This is only available in a thread in a forum.
    #[serde(default)]
    pub applied_tags: Vec<ForumTagId>,
    /// The emoji to show in the add reaction button
    ///
    /// **Note**: This is only available in a forum.
    pub default_reaction_emoji: Option<ForumEmoji>,
    /// The initial `rate_limit_per_user` to set on newly created threads in a channel. This field
    /// is copied to the thread at creation time and does not live update.
    ///
    /// **Note**: This is only available in a forum or text channel.
    pub default_thread_rate_limit_per_user: Option<u16>,
    /// The status of a voice channel.
    ///
    /// **Note**: This is only available in voice channels.
    pub status: Option<String>,
    /// The default sort order type used to order posts
    ///
    /// **Note**: This is only available in a forum.
    pub default_sort_order: Option<SortOrder>,
    /// The default forum layout view used to display posts in a forum. Defaults to 0, which
    /// indicates a layout view has not been set by a channel admin.
    ///
    /// **Note**: This is only available in a forum.
    pub default_forum_layout: Option<ForumLayoutType>,
}

enum_number! {
    /// See [`GuildChannel::default_forum_layout`].
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object-forum-layout-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum ForumLayoutType {
        /// No default has been set for forum channel.
        #[default]
        NotSet = 0,
        /// Display posts as a list.
        ListView = 1,
        /// Display posts as a collection of tiles.
        GalleryView = 2,
        _ => Unknown(u8),
    }
}

impl fmt::Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.id.mention(), f)
    }
}

/// A partial guild channel.
///
/// [Discord docs](https://docs.discord.com/developers/resources/channel#channel-object),
/// [subset description](https://docs.discord.com/developers/events/gateway-events#thread-delete)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialGuildChannel {
    /// The channel Id.
    pub id: ChannelId,
    /// The channel guild Id.
    pub guild_id: GuildId,
    /// The channel category Id,  or the parent text channel Id for a thread.
    pub parent_id: ChannelId,
    /// The channel type.
    #[serde(rename = "type")]
    pub kind: ChannelType,
}
