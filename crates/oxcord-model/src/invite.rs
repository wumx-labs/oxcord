//! Models for server and channel invites.

use super::prelude::*;

/// Information about an invite code.
///
/// Information can not be accessed for guilds the current user is banned from.
///
/// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Invite {
    /// The approximate number of [`Member`]s in the related [`Guild`].
    pub approximate_member_count: Option<u64>,
    /// The approximate number of [`Member`]s with an active session in the related [`Guild`].
    ///
    /// An active session is defined as an open, heartbeating WebSocket connection.
    /// These include [invisible][`OnlineStatus::Invisible`] members.
    pub approximate_presence_count: Option<u64>,
    /// The unique code for the invite.
    pub code: String,
    /// A representation of the minimal amount of information needed about the [`GuildChannel`]
    /// being invited to.
    pub channel: InviteChannel,
    /// A representation of the minimal amount of information needed about the [`Guild`] being
    /// invited to.
    pub guild: Option<InviteGuild>,
    /// A representation of the minimal amount of information needed about the [`User`] that
    /// created the invite.
    ///
    /// This can be [`None`] for invites created by Discord such as invite-widgets or vanity invite
    /// links.
    pub inviter: Option<User>,
    /// The type of target for this voice channel invite.
    pub target_type: Option<InviteTargetType>,
    /// The user whose stream to display for this voice channel stream invite.
    ///
    /// Only shows up if `target_type` is `Stream`.
    pub target_user: Option<UserId>,
    /// The embedded application to open for this voice channel embedded application invite.
    ///
    /// Only shows up if `target_type` is `EmmbeddedApplication`.
    pub target_application: Option<ApplicationId>,
    /// The expiration date of this invite, returned from `Http::get_invite` when `with_expiration`
    /// is true.
    pub expires_at: Option<Timestamp>,
    /// The Stage instance data if there is a public Stage instance in the Stage channel this
    /// invite is for.
    pub stage_instance: Option<InviteStageInstance>,
    /// Guild scheduled event data, only included if guild_scheduled_event_id contains a valid
    /// guild scheduled event id (according to Discord docs, whatever that means).
    #[serde(rename = "guild_scheduled_event")]
    pub scheduled_event: Option<ScheduledEvent>,
}

/// A minimal amount of information about the channel an invite points to.
///
/// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object-example-invite-object).
#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InviteChannel {
    pub id: ChannelId,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ChannelType,
}

/// Subset of [`Guild`] used in [`Invite`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object-example-invite-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteGuild {
    pub id: GuildId,
    pub name: String,
    pub splash: Option<ImageHash>,
    pub banner: Option<ImageHash>,
    pub description: Option<String>,
    pub icon: Option<ImageHash>,
    pub features: Vec<String>,
    pub verification_level: VerificationLevel,
    pub vanity_url_code: Option<String>,
    pub nsfw_level: NsfwLevel,
    pub premium_subscription_count: Option<u64>,
}

/// Detailed information about an invite.
///
/// This information can only be retrieved by anyone with the [Manage Guild] permission. Otherwise,
/// a minimal amount of information can be retrieved via the [`Invite`] struct.
///
/// [Manage Guild]: Permissions::MANAGE_GUILD
///
/// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-metadata-object) (extends [`Invite`] fields).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RichInvite {
    /// A representation of the minimal amount of information needed about the channel being
    /// invited to.
    pub channel: InviteChannel,
    /// The unique code for the invite.
    pub code: String,
    /// When the invite was created.
    pub created_at: Timestamp,
    /// A representation of the minimal amount of information needed about the [`Guild`] being
    /// invited to.
    pub guild: Option<InviteGuild>,
    /// The user that created the invite.
    pub inviter: Option<User>,
    /// The maximum age of the invite in seconds, from when it was created.
    pub max_age: u32,
    /// The maximum number of times that an invite may be used before it expires.
    ///
    /// Note that this does not supersede the [`Self::max_age`] value, if the value of
    /// [`Self::temporary`] is `true`. If the value of `temporary` is `false`, then the invite
    /// _will_ self-expire after the given number of max uses.
    ///
    /// If the value is `0`, then the invite is permanent.
    pub max_uses: u8,
    /// Indicator of whether the invite self-expires after a certain amount of time or uses.
    pub temporary: bool,
    /// The amount of times that an invite has been used.
    pub uses: u64,
}

/// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-stage-instance-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct InviteStageInstance {
    /// The members speaking in the Stage
    pub members: Vec<PartialMember>,
    /// The number of users in the Stage
    pub participant_count: u64,
    /// The number of users speaking in the Stage
    pub speaker_count: u64,
    /// The topic of the Stage instance (1-120 characters)
    pub topic: String,
}

enum_number! {
    /// Type of target for a voice channel invite.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/invite#invite-object-invite-target-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum InviteTargetType {
        Stream = 1,
        EmbeddedApplication = 2,
        _ => Unknown(u8),
    }
}
