//! User information-related models.

use std::fmt;
#[cfg(feature = "model")]
use std::fmt::Write;
use std::num::NonZeroU16;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use super::prelude::*;

/// Used with `#[serde(with|deserialize_with|serialize_with)]`
///
/// # Examples
///
/// ```rust,ignore
/// use std::num::NonZeroU16;
///
/// #[derive(Deserialize, Serialize)]
/// struct A {
///     #[serde(with = "discriminator")]
///     id: Option<NonZeroU16>,
/// }
///
/// #[derive(Deserialize)]
/// struct B {
///     #[serde(deserialize_with = "discriminator::deserialize")]
///     id: Option<NonZeroU16>,
/// }
///
/// #[derive(Serialize)]
/// struct C {
///     #[serde(serialize_with = "discriminator::serialize")]
///     id: Option<NonZeroU16>,
/// }
/// ```
pub(crate) mod discriminator {
    use std::fmt;

    use serde::de::{Error, Visitor};

    struct DiscriminatorVisitor;

    impl Visitor<'_> for DiscriminatorVisitor {
        type Value = u16;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("string or integer discriminator")
        }

        fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
            u16::try_from(value).map_err(Error::custom)
        }

        fn visit_str<E: Error>(self, s: &str) -> Result<Self::Value, E> {
            s.parse().map_err(Error::custom)
        }
    }

    use std::num::NonZeroU16;

    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<NonZeroU16>, D::Error> {
        deserializer.deserialize_option(OptionalDiscriminatorVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref, clippy::ref_option)]
    pub fn serialize<S: Serializer>(
        value: &Option<NonZeroU16>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => serializer.serialize_some(&format_args!("{value:04}")),
            None => serializer.serialize_none(),
        }
    }

    struct OptionalDiscriminatorVisitor;

    impl<'de> Visitor<'de> for OptionalDiscriminatorVisitor {
        type Value = Option<NonZeroU16>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("optional string or integer discriminator")
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            deserializer.deserialize_any(DiscriminatorVisitor).map(NonZeroU16::new)
        }
    }
}

/// Information about the current user.
///
/// [Discord docs](https://docs.discord.com/developers/resources/user#user-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct CurrentUser(User);

impl Deref for CurrentUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CurrentUser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<CurrentUser> for User {
    fn from(user: CurrentUser) -> Self {
        user.0
    }
}

/// The representation of a user's status.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#update-presence-status-types).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
)]
#[non_exhaustive]
pub enum OnlineStatus {
    #[serde(rename = "dnd")]
    DoNotDisturb,
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "invisible")]
    Invisible,
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "online")]
    #[default]
    Online,
    #[serde(other)]
    Unknown,
}

impl OnlineStatus {
    #[must_use]
    pub fn name(&self) -> &str {
        match *self {
            OnlineStatus::DoNotDisturb => "dnd",
            OnlineStatus::Idle => "idle",
            OnlineStatus::Invisible => "invisible",
            OnlineStatus::Offline => "offline",
            OnlineStatus::Online => "online",
            OnlineStatus::Unknown => "unknown",
        }
    }
}

/// Information about a user.
///
/// [Discord docs](https://docs.discord.com/developers/resources/user#user-object), existence of
/// additional partial member field documented [here](https://docs.discord.com/developers/events/gateway-events#message-create).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct User {
    /// The unique Id of the user. Can be used to calculate the account's creation date.
    pub id: UserId,
    /// The account's username. Changing username will trigger a discriminator
    /// change if the username+discriminator pair becomes non-unique. Unless the account has
    /// migrated to a next generation username, which does not have a discriminant.
    #[serde(rename = "username")]
    pub name: String,
    /// The account's discriminator to differentiate the user from others with
    /// the same [`Self::name`]. The name+discriminator pair is always unique.
    /// If the discriminator is not present, then this is a next generation username
    /// which is implicitly unique.
    #[serde(default, skip_serializing_if = "Option::is_none", with = "discriminator")]
    pub discriminator: Option<NonZeroU16>,
    /// The account's display name, if it is set.
    pub global_name: Option<String>,
    /// Optional avatar hash.
    pub avatar: Option<ImageHash>,
    /// Indicator of whether the user is a bot.
    #[serde(default)]
    pub bot: bool,
    /// Whether the user is an Official Discord System user (part of the urgent message system).
    #[serde(default)]
    pub system: bool,
    /// Whether the user has two factor enabled on their account
    #[serde(default)]
    pub mfa_enabled: bool,
    /// Optional banner hash.
    ///
    /// **Note**: This will only be present if the user is fetched via Rest API, e.g. with
    /// [`crate::http::Http::get_user`].
    pub banner: Option<ImageHash>,
    /// The user's banner colour encoded as an integer representation of hexadecimal colour code
    ///
    /// **Note**: This will only be present if the user is fetched via Rest API, e.g. with
    /// [`crate::http::Http::get_user`].
    #[serde(rename = "accent_color")]
    pub accent_colour: Option<Colour>,
    /// The user's chosen language option
    pub locale: Option<String>,
    /// Whether the email on this account has been verified
    ///
    /// Requires [`Scope::Email`]
    pub verified: Option<bool>,
    /// The user's email
    ///
    /// Requires [`Scope::Email`]
    pub email: Option<String>,
    /// The flags on a user's account
    #[serde(default)]
    pub flags: UserPublicFlags,
    /// The type of Nitro subscription on a user's account
    #[serde(default)]
    pub premium_type: PremiumType,
    /// The public flags on a user's account
    pub public_flags: Option<UserPublicFlags>,
    /// Only included in [`Message::mentions`] for messages from the gateway.
    ///
    /// [Discord docs](https://docs.discord.com/developers/events/gateway-events#message-create-message-create-extra-fields).
    // Box required to avoid infinitely recursive types
    pub member: Option<Box<PartialMember>>,
    /// The primary guild and tag the user has active.
    ///
    /// Note: just because this guild is populated does not mean the tag is visible.
    pub primary_guild: Option<PrimaryGuild>,
    /// Information about this user's avatar decoration.
    pub avatar_decoration_data: Option<AvatarDecorationData>,
    /// The collectibles the user currently has active, excluding avatar decorations and profile
    /// effects.
    pub collectibles: Option<Collectibles>,
}

enum_number! {
    /// Premium types denote the level of premium a user has. Visit the [Nitro](https://discord.com/nitro)
    /// page to learn more about the premium plans Discord currently offers.
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/user#user-object-premium-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum PremiumType {
        #[default]
        None = 0,
        NitroClassic = 1,
        Nitro = 2,
        NitroBasic = 3,
        _ => Unknown(u8),
    }
}

bitflags! {
    /// User's public flags
    ///
    /// [Discord docs](https://docs.discord.com/developers/resources/user#user-object-user-flags).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct UserPublicFlags: u32 {
        /// User's flag as discord employee
        const DISCORD_EMPLOYEE = 1 << 0;
        /// User's flag as partnered server owner
        const PARTNERED_SERVER_OWNER = 1 << 1;
        /// User's flag as hypesquad events
        const HYPESQUAD_EVENTS = 1 << 2;
        /// User's flag as bug hunter level 1
        const BUG_HUNTER_LEVEL_1 = 1 << 3;
        /// User's flag as house bravery
        const HOUSE_BRAVERY = 1 << 6;
        /// User's flag as house brilliance
        const HOUSE_BRILLIANCE = 1 << 7;
        /// User's flag as house balance
        const HOUSE_BALANCE = 1 << 8;
        /// User's flag as early supporter
        const EARLY_SUPPORTER = 1 << 9;
        /// User's flag as team user
        const TEAM_USER = 1 << 10;
        /// User's flag as system
        const SYSTEM = 1 << 12;
        /// User's flag as bug hunter level 2
        const BUG_HUNTER_LEVEL_2 = 1 << 14;
        /// User's flag as verified bot
        const VERIFIED_BOT = 1 << 16;
        /// User's flag as early verified bot developer
        const EARLY_VERIFIED_BOT_DEVELOPER = 1 << 17;
        /// User's flag as discord certified moderator
        const DISCORD_CERTIFIED_MODERATOR = 1 << 18;
        /// Bot's running with HTTP interactions
        const BOT_HTTP_INTERACTIONS = 1 << 19;
        /// User's flag for suspected spam activity.
        #[cfg(feature = "unstable_discord_api")]
        const SPAMMER = 1 << 20;
        /// User's flag as active developer
        const ACTIVE_DEVELOPER = 1 << 22;
    }
}

/// User's Primary Guild object
///
/// [Discord docs](https://docs.discord.com/developers/resources/user#user-object-user-primary-guild)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PrimaryGuild {
    /// The id of the user's primary guild.
    pub identity_guild_id: Option<GuildId>,
    /// Whether the user is displaying the primary guild's server tag. This can be null if the
    /// system clears the identity, e.g. because the server no longer supports tags.
    pub identity_enabled: Option<bool>,
    /// The text of the [`User`]'s server tag.
    pub tag: Option<String>,
    /// The hash of the server badge.
    pub badge: Option<ImageHash>,
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[non_exhaustive]
/// The data for a [`User`]'s avatar decoration.
///
/// [Discord docs](https://docs.discord.com/developers/resources/user#avatar-decoration-data-object).
pub struct AvatarDecorationData {
    /// The avatar decoration hash
    pub asset: ImageHash,
    /// id of the avatar decoration's SKU
    pub sku_id: SkuId,
}

/// The collectibles the user has, excluding Avatar Decorations and Profile Effects.
///
/// [Discord docs](https://docs.discord.com/developers/resources/user#collectibles).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Collectibles {
    /// The [`User`]'s nameplate, if they have one.
    pub nameplate: Option<Nameplate>,
}

/// A nameplate, shown on the member list on official clients.
///
/// [Discord docs](https://docs.discord.com/developers/resources/user#nameplate-nameplate-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Nameplate {
    /// Id of the nameplate SKU
    pub sku_id: SkuId,
    /// Path to the nameplate asset.
    pub asset: String,
    /// The label of this nameplate.
    pub label: String,
    /// Background color of the nameplate, one of: `crimson`, `berry`, `sky`, `teal`, `forest`,
    /// `bubble_gum`, `violet`, `cobalt`, `clover`, `lemon`, `white`
    pub palette: String,
}

use std::hash::{Hash, Hasher};

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for User {}

impl Hash for User {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl fmt::Display for User {
    /// Formats a string which will mention the user.
    // This is in the format of: `<@USER_ID>`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.id.mention(), f)
    }
}

impl From<Member> for UserId {
    /// Gets the Id of a [`Member`].
    fn from(member: Member) -> UserId {
        member.user.id
    }
}

impl From<&Member> for UserId {
    /// Gets the Id of a [`Member`].
    fn from(member: &Member) -> UserId {
        member.user.id
    }
}

impl From<User> for UserId {
    /// Gets the Id of a [`User`].
    fn from(user: User) -> UserId {
        user.id
    }
}

impl From<&User> for UserId {
    /// Gets the Id of a [`User`].
    fn from(user: &User) -> UserId {
        user.id
    }
}

#[cfg(feature = "model")]
pub fn default_avatar_url(user: &User) -> String {
    let avatar_id = if let Some(discriminator) = user.discriminator {
        discriminator.get() % 5 // Legacy username system
    } else {
        ((user.id.get() >> 22) % 6) as u16 // New username system
    };

    cdn!("/embed/avatars/{}.png", avatar_id)
}

#[cfg(feature = "model")]
pub fn static_avatar_url(user_id: UserId, hash: Option<&ImageHash>) -> Option<String> {
    hash.map(|hash| cdn!("/avatars/{}/{}.webp?size=1024", user_id, hash))
}

#[cfg(feature = "model")]
pub fn tag(name: &str, discriminator: Option<NonZeroU16>) -> String {
    // 32: max length of username
    // 1: `#`
    // 4: max length of discriminator
    let mut tag = String::with_capacity(37);
    tag.push_str(name);
    if let Some(discriminator) = discriminator {
        tag.push('#');
        write!(tag, "{discriminator:04}").expect("writing to a string should never fail");
    }
    tag
}

#[cfg(feature = "model")]
pub fn primary_guild_badge_url(
    guild_id: Option<GuildId>,
    hash: Option<&ImageHash>,
) -> Option<String> {
    if let Some(guild_id) = guild_id {
        return hash.map(|hash| cdn!("/guild-tag-badges/{}/{}.png?size=1024", guild_id, hash));
    }

    None
}

#[cfg(feature = "model")]
pub fn avatar_decoration_url(hash: &ImageHash) -> String {
    cdn!("/avatar-decoration-presets/{}.png?size=1024", hash)
}

#[cfg(all(feature = "unstable_discord_api", feature = "model"))]
pub fn nameplate_url(path: &str) -> String {
    cdn!("https://cdn.discordapp.com/assets/collectibles/{}/asset.webm", path)
}

#[cfg(all(feature = "unstable_discord_api", feature = "model"))]
#[cfg(feature = "model")]
pub fn static_nameplate_url(path: &str) -> String {
    cdn!("https://cdn.discordapp.com/assets/collectibles/{}/static.png", path)
}

#[cfg(test)]
mod test {

    use std::num::NonZeroU16;

    #[test]
    fn test_discriminator_serde() {
        use serde::{Deserialize, Serialize};

        use super::discriminator;
        use crate::json::{assert_json, json};

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct User {
            #[serde(default, skip_serializing_if = "Option::is_none", with = "discriminator")]
            discriminator: Option<NonZeroU16>,
        }

        let user = User {
            discriminator: NonZeroU16::new(123),
        };
        assert_json(&user, json!({"discriminator": "0123"}));

        let user_no_discriminator = User {
            discriminator: None,
        };
        assert_json(&user_no_discriminator, json!({}));
    }
}
