//! Parsing helpers for invites, mentions, emojis, webhook URLs and similar text forms.

use std::num::NonZeroU16;

use url::Url;

use crate::model::prelude::*;

/// Retrieves the "code" part of an invite out of a URL.
///
/// # Examples
///
/// Two formats of [invite][`RichInvite`] codes are supported, both regardless of protocol prefix.
/// Some examples:
///
/// 1. Retrieving the code from the URL `"https://discord.gg/0cDvIgU2voY8RSYL"`:
///
/// ```rust
/// use oxcord_model::parse::parse_invite;
///
/// let url = "https://discord.gg/0cDvIgU2voY8RSYL";
///
/// assert_eq!(parse_invite(url), "0cDvIgU2voY8RSYL");
/// ```
///
/// 2. Retrieving the code from the URL `"http://discord.com/invite/0cDvIgU2voY8RSYL"`:
///
/// ```rust
/// use oxcord_model::parse::parse_invite;
///
/// let url = "http://discord.com/invite/0cDvIgU2voY8RSYL";
///
/// assert_eq!(parse_invite(url), "0cDvIgU2voY8RSYL");
/// ```
///
/// [`RichInvite`]: crate::model::invite::RichInvite
#[must_use]
pub fn parse_invite(code: &str) -> &str {
    let code = code.trim_start_matches("http://").trim_start_matches("https://");
    let lower = code.to_lowercase();
    if lower.starts_with("discord.gg/") {
        &code[11..]
    } else if lower.starts_with("discord.com/invite/") {
        &code[19..]
    } else {
        code
    }
}

/// Retrieves the username and discriminator out of a user tag (`name#discrim`).
/// In order to accomodate next gen Discord usernames, this will also accept `name` style tags.
///
/// If the user tag is invalid, None is returned.
///
/// # Examples
/// ```rust
/// use std::num::NonZeroU16;
///
/// use oxcord_model::parse::parse_user_tag;
///
/// assert_eq!(parse_user_tag("kangalioo#9108"), Some(("kangalioo", NonZeroU16::new(9108))));
/// assert_eq!(parse_user_tag("kangalioo#10108"), None);
/// assert_eq!(parse_user_tag("kangalioo"), Some(("kangalioo", None)));
/// ```
#[must_use]
pub fn parse_user_tag(s: &str) -> Option<(&str, Option<NonZeroU16>)> {
    if let Some((name, discrim)) = s.split_once('#') {
        let discrim: u16 = discrim.parse().ok()?;
        if discrim > 9999 {
            return None;
        }
        Some((name, NonZeroU16::new(discrim)))
    } else {
        Some((s, None))
    }
}

/// Retrieves an Id from a user mention.
///
/// If the mention is invalid, then [`None`] is returned.
///
/// # Examples
///
/// Retrieving an Id from a valid [`User`] mention:
///
/// ```rust
/// use oxcord_model::id::UserId;
/// use oxcord_model::parse::parse_user_mention;
///
/// // regular username mention
/// assert_eq!(parse_user_mention("<@114941315417899012>"), Some(UserId::new(114941315417899012)));
///
/// // nickname mention
/// assert_eq!(parse_user_mention("<@!114941315417899012>"), Some(UserId::new(114941315417899012)));
/// ```
///
/// Asserting that an invalid username or nickname mention returns [`None`]:
///
/// ```rust
/// use oxcord_model::parse::parse_user_mention;
///
/// assert!(parse_user_mention("<@1149413154aa17899012").is_none());
/// assert!(parse_user_mention("<@!11494131541789a90b1c2").is_none());
/// ```
///
/// [`User`]: crate::model::user::User
#[must_use]
pub fn parse_user_mention(mention: &str) -> Option<UserId> {
    if mention.len() < 4 {
        return None;
    }

    let len = mention.len() - 1;
    if mention.starts_with("<@!") {
        mention[3..len].parse().ok()
    } else if mention.starts_with("<@") {
        mention[2..len].parse().ok()
    } else {
        None
    }
}

/// Retrieves an Id from a role mention.
///
/// If the mention is invalid, then [`None`] is returned.
///
/// # Examples
///
/// Retrieving an Id from a valid [`Role`] mention:
///
/// ```rust
/// use oxcord_model::id::RoleId;
/// use oxcord_model::parse::parse_role_mention;
///
/// assert_eq!(parse_role_mention("<@&136107769680887808>"), Some(RoleId::new(136107769680887808)));
/// ```
///
/// Asserting that an invalid role mention returns [`None`]:
///
/// ```rust
/// use oxcord_model::parse::parse_role_mention;
///
/// assert!(parse_role_mention("<@&136107769680887808").is_none());
/// ```
///
/// [`Role`]: crate::model::guild::Role
#[must_use]
pub fn parse_role_mention(mention: &str) -> Option<RoleId> {
    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@&") && mention.ends_with('>') {
        let len = mention.len() - 1;
        mention[3..len].parse().ok()
    } else {
        None
    }
}

/// Retrieves an Id from a channel mention.
///
/// If the channel mention is invalid, then [`None`] is returned.
///
/// # Examples
///
/// Retrieving an Id from a valid [`Channel`] mention:
///
/// ```rust
/// use oxcord_model::id::ChannelId;
/// use oxcord_model::parse::parse_channel_mention;
///
/// assert_eq!(
///     parse_channel_mention("<#81384788765712384>"),
///     Some(ChannelId::new(81384788765712384))
/// );
/// ```
///
/// Asserting that an invalid channel mention returns [`None`]:
///
/// ```rust
/// use oxcord_model::parse::parse_channel_mention;
///
/// assert!(parse_channel_mention("<#!81384788765712384>").is_none());
/// assert!(parse_channel_mention("<#81384788765712384").is_none());
/// ```
///
/// [`Channel`]: crate::model::channel::Channel
#[must_use]
pub fn parse_channel_mention(mention: &str) -> Option<ChannelId> {
    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<#") && mention.ends_with('>') {
        let len = mention.len() - 1;
        mention[2..len].parse().ok()
    } else {
        None
    }
}

/// Retrieves the animated state, name and Id from an emoji mention, in the form of an
/// [`EmojiIdentifier`].
///
/// If the emoji usage is invalid, then [`None`] is returned.
///
/// # Examples
///
/// Ensure that a valid [`Emoji`] usage is correctly parsed:
///
/// ```rust
/// use oxcord_model::id::{EmojiId, GuildId};
/// use oxcord_model::misc::EmojiIdentifier;
/// use oxcord_model::parse::parse_emoji;
///
/// let emoji = parse_emoji("<:smugAnimeFace:302516740095606785>").unwrap();
/// assert_eq!(emoji.animated, false);
/// assert_eq!(emoji.id, EmojiId::new(302516740095606785));
/// assert_eq!(emoji.name, "smugAnimeFace".to_string());
/// ```
///
/// Asserting that an invalid emoji usage returns [`None`]:
///
/// ```rust
/// use oxcord_model::parse::parse_emoji;
///
/// assert!(parse_emoji("<:smugAnimeFace:302516740095606785").is_none());
/// ```
///
/// [`Emoji`]: crate::model::guild::Emoji
pub fn parse_emoji(mention: impl AsRef<str>) -> Option<EmojiIdentifier> {
    let mention = mention.as_ref();

    let len = mention.len();

    if !(6..=56).contains(&len) {
        return None;
    }

    if (mention.starts_with("<:") || mention.starts_with("<a:")) && mention.ends_with('>') {
        let mut name = String::default();
        let mut id = String::default();
        let animated = &mention[1..3] == "a:";

        let start = if animated { 3 } else { 2 };

        for (i, x) in mention[start..].chars().enumerate() {
            if x == ':' {
                let from = i + start + 1;

                for y in mention[from..].chars() {
                    if y == '>' {
                        break;
                    }
                    id.push(y);
                }

                break;
            }
            name.push(x);
        }

        id.parse().ok().map(|id| EmojiIdentifier {
            animated,
            id,
            name,
        })
    } else {
        None
    }
}

/// Turns a string into a vector of string arguments, splitting by spaces, but parsing content
/// within quotes as one individual argument.
///
/// # Examples
///
/// Parsing two quoted commands:
///
/// ```rust
/// use oxcord_model::parse::parse_quotes;
///
/// let command = r#""this is the first" "this is the second""#;
/// let expected = vec!["this is the first".to_string(), "this is the second".to_string()];
///
/// assert_eq!(parse_quotes(command), expected);
/// ```
///
/// ```rust
/// use oxcord_model::parse::parse_quotes;
///
/// let command = r#""this is a quoted command that doesn't have an ending quotation"#;
/// let expected =
///     vec!["this is a quoted command that doesn't have an ending quotation".to_string()];
///
/// assert_eq!(parse_quotes(command), expected);
/// ```
pub fn parse_quotes(s: impl AsRef<str>) -> Vec<String> {
    let s = s.as_ref();
    let mut args = vec![];
    let mut in_string = false;
    let mut escaping = false;
    let mut current_str = String::default();

    for x in s.chars() {
        if in_string {
            if x == '\\' && !escaping {
                escaping = true;
            } else if x == '"' && !escaping {
                if !current_str.is_empty() {
                    args.push(current_str);
                }

                current_str = String::default();
                in_string = false;
            } else {
                current_str.push(x);
                escaping = false;
            }
        } else if x == ' ' {
            if !current_str.is_empty() {
                args.push(current_str.clone());
            }

            current_str = String::default();
        } else if x == '"' {
            if !current_str.is_empty() {
                args.push(current_str.clone());
            }

            in_string = true;
            current_str = String::default();
        } else {
            current_str.push(x);
        }
    }

    if !current_str.is_empty() {
        args.push(current_str);
    }

    args
}

/// Discord's official domains. This is used in [`parse_webhook`] and in its corresponding test.
pub const DOMAINS: &[&str] = &[
    "discord.com",
    "canary.discord.com",
    "ptb.discord.com",
    "discordapp.com",
    "canary.discordapp.com",
    "ptb.discordapp.com",
];

/// Parses the id and token from a webhook url. Expects a [`url::Url`] rather than a [`&str`].
///
/// # Examples
///
/// ```rust
/// use oxcord_model::parse::parse_webhook;
///
/// let url_str = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
/// let url = url_str.parse().unwrap();
/// let (id, token) = parse_webhook(&url).unwrap();
///
/// assert_eq!(id, 245037420704169985);
/// assert_eq!(token, "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV");
/// ```
#[must_use]
pub fn parse_webhook(url: &Url) -> Option<(WebhookId, &str)> {
    let (webhook_id, token) = url.path().strip_prefix("/api/webhooks/")?.split_once('/')?;
    if !["http", "https"].contains(&url.scheme())
        || !DOMAINS.contains(&url.domain()?)
        || !(17..=20).contains(&webhook_id.len())
        || !(60..=68).contains(&token.len())
    {
        return None;
    }
    Some((webhook_id.parse().ok()?, token))
}
