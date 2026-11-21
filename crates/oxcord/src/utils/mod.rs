//! A set of utilities to help with common use cases that are not required to fully use the
//! library.

#[cfg(feature = "client")]
mod argument_convert;
#[cfg(feature = "cache")]
mod content_safe;
mod custom_message;
mod formatted_timestamp;
mod message_builder;
#[cfg(feature = "collector")]
mod quick_modal;

pub mod token;

#[cfg(feature = "client")]
pub use argument_convert::*;
#[cfg(feature = "cache")]
pub use content_safe::*;
pub use formatted_timestamp::*;
#[cfg(feature = "collector")]
pub use quick_modal::*;

pub use self::custom_message::CustomMessage;
pub use self::message_builder::{Content, ContentModifier, EmbedMessageBuilding, MessageBuilder};
#[doc(inline)]
pub use self::token::validate as validate_token;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
pub use crate::model::parse::*;
use crate::model::prelude::*;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::ops::guild_mod::GuildExt;

/// Tries to find a user's permissions using the cache. Unlike [`user_has_perms`], this function
/// will return `true` even when the permissions are not in the cache.
#[cfg(all(feature = "cache", feature = "model"))]
#[inline]
pub(crate) fn user_has_perms_cache(
    cache: impl AsRef<Cache>,
    channel_id: ChannelId,
    required_permissions: Permissions,
) -> Result<()> {
    match user_perms(cache, channel_id) {
        Ok(perms) => {
            if perms.contains(required_permissions) {
                Ok(())
            } else {
                Err(Error::Model(ModelError::InvalidPermissions {
                    required: required_permissions,
                    present: perms,
                }))
            }
        },
        Err(Error::Model(err)) if err.is_cache_err() => Ok(()),
        Err(other) => Err(other),
    }
}

#[cfg(all(feature = "cache", feature = "model"))]
pub(crate) fn user_perms(cache: impl AsRef<Cache>, channel_id: ChannelId) -> Result<Permissions> {
    let cache = cache.as_ref();

    let Some(guild_id) = cache.channels.get(&channel_id).map(|c| *c) else {
        return Err(Error::Model(ModelError::ChannelNotFound));
    };

    let Some(guild) = cache.guild(guild_id) else {
        return Err(Error::Model(ModelError::GuildNotFound));
    };

    let Some(channel) = guild.channels.get(&channel_id) else {
        return Err(Error::Model(ModelError::ChannelNotFound));
    };

    let Some(member) = guild.members.get(&cache.current_user().id) else {
        return Err(Error::Model(ModelError::MemberNotFound));
    };

    Ok(guild.user_permissions_in(channel, member))
}

/// Calculates the Id of the shard responsible for a guild, given its Id and total number of shards
/// used.
///
/// # Examples
///
/// Retrieve the Id of the shard for a guild with Id `81384788765712384`, using 17 shards:
///
/// ```rust
/// use oxcord::model::id::GuildId;
/// use oxcord::utils;
///
/// assert_eq!(utils::shard_id(GuildId::new(81384788765712384), 17), 7);
/// ```
#[inline]
#[must_use]
pub fn shard_id(guild_id: GuildId, shard_count: u32) -> u32 {
    ((guild_id.get() >> 22) % (shard_count as u64)) as u32
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::parse::DOMAINS;

    #[test]
    fn test_invite_parser() {
        assert_eq!(parse_invite("https://discord.gg/abc"), "abc");
        assert_eq!(parse_invite("http://discord.gg/abc"), "abc");
        assert_eq!(parse_invite("discord.gg/abc"), "abc");
        assert_eq!(parse_invite("DISCORD.GG/ABC"), "ABC");
        assert_eq!(parse_invite("https://discord.com/invite/abc"), "abc");
        assert_eq!(parse_invite("http://discord.com/invite/abc"), "abc");
        assert_eq!(parse_invite("discord.com/invite/abc"), "abc");
    }

    #[test]
    fn test_username_parser() {
        assert_eq!(parse_user_mention("<@12345>").unwrap(), 12_345);
        assert_eq!(parse_user_mention("<@!12345>").unwrap(), 12_345);
    }

    #[test]
    fn role_parser() {
        assert_eq!(parse_role_mention("<@&12345>").unwrap(), 12_345);
    }

    #[test]
    fn test_channel_parser() {
        assert_eq!(parse_channel_mention("<#12345>").unwrap(), 12_345);
    }

    #[test]
    fn test_emoji_parser() {
        let emoji = parse_emoji("<:name:12345>").unwrap();
        assert_eq!(emoji.name, "name");
        assert_eq!(emoji.id, 12_345);
    }

    #[test]
    fn test_quote_parser() {
        let parsed = parse_quotes("a \"b c\" d\"e f\"  g");
        assert_eq!(parsed, ["a", "b c", "d", "e f", "g"]);
    }

    #[test]
    fn test_webhook_parser() {
        for domain in DOMAINS {
            let url = format!("https://{domain}/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV").parse().unwrap();
            let (id, token) = parse_webhook(&url).unwrap();
            assert_eq!(id, 245037420704169985);
            assert_eq!(
                token,
                "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV"
            );
        }
    }
}
