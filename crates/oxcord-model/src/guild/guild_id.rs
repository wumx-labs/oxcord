use std::fmt;

#[cfg(feature = "model")]
#[allow(unused_imports)]
use crate::internal::prelude::*;
use crate::model::prelude::*;

#[cfg(feature = "model")]
impl GuildId {
    /// Gets the default permission role (@everyone) from the guild.
    #[inline]
    #[must_use]
    pub const fn everyone_role(&self) -> RoleId {
        RoleId::new(self.get())
    }
}

impl From<PartialGuild> for GuildId {
    /// Gets the Id of a partial guild.
    fn from(guild: PartialGuild) -> GuildId {
        guild.id
    }
}

impl From<&PartialGuild> for GuildId {
    /// Gets the Id of a partial guild.
    fn from(guild: &PartialGuild) -> GuildId {
        guild.id
    }
}

impl From<GuildInfo> for GuildId {
    /// Gets the Id of Guild information struct.
    fn from(guild_info: GuildInfo) -> GuildId {
        guild_info.id
    }
}

impl From<&GuildInfo> for GuildId {
    /// Gets the Id of Guild information struct.
    fn from(guild_info: &GuildInfo) -> GuildId {
        guild_info.id
    }
}

impl From<InviteGuild> for GuildId {
    /// Gets the Id of Invite Guild struct.
    fn from(invite_guild: InviteGuild) -> GuildId {
        invite_guild.id
    }
}

impl From<&InviteGuild> for GuildId {
    /// Gets the Id of Invite Guild struct.
    fn from(invite_guild: &InviteGuild) -> GuildId {
        invite_guild.id
    }
}

impl From<Guild> for GuildId {
    /// Gets the Id of Guild.
    fn from(live_guild: Guild) -> GuildId {
        live_guild.id
    }
}

impl From<&Guild> for GuildId {
    /// Gets the Id of Guild.
    fn from(live_guild: &Guild) -> GuildId {
        live_guild.id
    }
}

impl From<WebhookGuild> for GuildId {
    /// Gets the Id of Webhook Guild struct.
    fn from(webhook_guild: WebhookGuild) -> GuildId {
        webhook_guild.id
    }
}

impl From<&WebhookGuild> for GuildId {
    /// Gets the Id of Webhook Guild struct.
    fn from(webhook_guild: &WebhookGuild) -> GuildId {
        webhook_guild.id
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum GuildWidgetStyle {
    Shield,
    Banner1,
    Banner2,
    Banner3,
    Banner4,
    #[serde(other)]
    Unknown,
}

impl fmt::Display for GuildWidgetStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shield => f.write_str("shield"),
            Self::Banner1 => f.write_str("banner1"),
            Self::Banner2 => f.write_str("banner2"),
            Self::Banner3 => f.write_str("banner3"),
            Self::Banner4 => f.write_str("banner4"),
            Self::Unknown => f.write_str("unknown"),
        }
    }
}
