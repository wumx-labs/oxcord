#[cfg(feature = "model")]
use crate::builder::CreateInvite;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait InviteExt: Sized {
    #[cfg(feature = "model")]
    /// Creates an invite for the given channel.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    async fn create(
        cache_http: impl CacheHttp,
        channel_id: impl Into<ChannelId>,
        builder: CreateInvite<'_>,
    ) -> Result<RichInvite>;
    #[cfg(feature = "model")]
    /// Deletes the invite.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have the required [permission].
    ///
    /// Otherwise may return [`Error::Http`] if permissions are lacking, or if the invite is
    /// invalid.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [permission]: super::permissions
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<Invite>;
    #[cfg(feature = "model")]
    /// Gets information about an invite.
    ///
    /// # Arguments
    /// * `code` - The invite code.
    /// * `member_counts` - Whether to include information about the current number of members in
    ///   the server that the invite belongs to.
    /// * `expiration` - Whether to include information about when the invite expires.
    /// * `event_id` - An optional server event ID to include with the invite.
    ///
    /// More information about these arguments can be found on Discord's
    /// [API documentation](https://docs.discord.com/developers/resources/invite#get-invite).
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the invite is invalid. Can also return an [`Error::Json`]
    /// if there is an error deserializing the API response.
    async fn get(
        http: impl AsRef<Http>,
        code: &str,
        member_counts: bool,
        expiration: bool,
        event_id: Option<ScheduledEventId>,
    ) -> Result<Invite>;
    #[cfg(feature = "model")]
    /// Returns a URL to use for the invite.
    ///
    /// # Examples
    ///
    /// Retrieve the URL for an invite with the code `WxZumR`:
    ///
    /// ```rust
    /// # use oxcord::json::{json, from_value};
    /// # use oxcord::model::prelude::*;
    /// #
    /// # fn main() {
    /// # let invite = from_value::<Invite>(json!({
    /// #     "approximate_member_count": Some(1812),
    /// #     "approximate_presence_count": Some(717),
    /// #     "code": "WxZumR",
    /// #     "channel": {
    /// #         "id": ChannelId::new(1),
    /// #         "name": "foo",
    /// #         "type": ChannelType::Text,
    /// #     },
    /// #     "guild": {
    /// #         "id": GuildId::new(2),
    /// #         "icon": None::<String>,
    /// #         "name": "bar",
    /// #         "splash_hash": None::<String>,
    /// #         "text_channel_count": 7,
    /// #         "voice_channel_count": 3,
    /// #         "features": ["NEWS", "DISCOVERABLE"],
    /// #         "verification_level": 2,
    /// #         "nsfw_level": 0,
    /// #     },
    /// #     "inviter": {
    /// #         "id": UserId::new(3),
    /// #         "username": "foo",
    /// #         "discriminator": "1234",
    /// #         "avatar": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    /// #     },
    /// # })).unwrap();
    /// #
    /// assert_eq!(invite.url(), "https://discord.gg/WxZumR");
    /// # }
    /// ```
    #[must_use]
    fn url(&self) -> String;
}
impl InviteExt for Invite {
    #[cfg(feature = "model")]
    #[inline]
    async fn create(
        cache_http: impl CacheHttp,
        channel_id: impl Into<ChannelId>,
        builder: CreateInvite<'_>,
    ) -> Result<RichInvite> {
        channel_id.into().create_invite(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<Invite> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(
                    cache,
                    self.channel.id,
                    Permissions::MANAGE_GUILD,
                )?;
            }
        }
        cache_http.http().as_ref().delete_invite(&self.code, None).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn get(
        http: impl AsRef<Http>,
        code: &str,
        member_counts: bool,
        expiration: bool,
        event_id: Option<ScheduledEventId>,
    ) -> Result<Invite> {
        let mut invite = code;
        #[cfg(feature = "utils")]
        {
            invite = crate::utils::parse_invite(invite);
        }
        http.as_ref()
            .get_invite(invite, member_counts, expiration, event_id)
            .await
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    fn url(&self) -> String {
        format!("https://discord.gg/{}", self.code)
    }
}
#[allow(async_fn_in_trait)]
pub trait InviteGuildInviteExt: Sized {
    #[cfg(feature = "model")]
    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total number of shards.
    ///
    /// **Note**: When the cache is enabled, this function unlocks the cache to retrieve the total
    /// number of shards in use. If you already have the total, consider using [`utils::shard_id`].
    ///
    /// [`utils::shard_id`]: crate::utils::shard_id
    #[cfg(all(feature = "cache", feature = "utils"))]
    #[must_use]
    fn shard_id(&self, cache: impl AsRef<Cache>) -> u32;
    #[cfg(feature = "model")]
    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total number of shards.
    ///
    /// When the cache is not enabled, the total number of shards being used will need to be
    /// passed.
    ///
    /// # Examples
    ///
    /// Retrieve the Id of the shard for a guild with Id `81384788765712384`, using 17 shards:
    ///
    /// ```rust,ignore
    /// use oxcord::utils;
    ///
    /// // assumes a `guild` has already been bound
    ///
    /// assert_eq!(guild.shard_id(17), 7);
    /// ```
    #[cfg(all(feature = "utils", not(feature = "cache")))]
    #[must_use]
    fn shard_id(&self, shard_count: u32) -> u32;
}
impl InviteGuildInviteExt for InviteGuild {
    #[cfg(feature = "model")]
    #[cfg(all(feature = "cache", feature = "utils"))]
    #[inline]
    fn shard_id(&self, cache: impl AsRef<Cache>) -> u32 {
        self.id.shard_id(&cache)
    }
    #[cfg(feature = "model")]
    #[cfg(all(feature = "utils", not(feature = "cache")))]
    #[inline]
    fn shard_id(&self, shard_count: u32) -> u32 {
        self.id.shard_id(shard_count)
    }
}
#[allow(async_fn_in_trait)]
pub trait RichInviteExt: Sized {
    #[cfg(feature = "model")]
    /// Deletes the invite.
    ///
    /// Refer to [`Http::delete_invite`] for more information.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then this returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have the required [permission].
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [permission]: super::permissions
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<Invite>;
    #[cfg(feature = "model")]
    /// Returns a URL to use for the invite.
    ///
    /// # Examples
    ///
    /// Retrieve the URL for an invite with the code `WxZumR`:
    ///
    /// ```rust
    /// # use oxcord::json::{json, from_value};
    /// # use oxcord::model::prelude::*;
    /// #
    /// # fn main() {
    /// # let invite = from_value::<RichInvite>(json!({
    /// #     "code": "WxZumR",
    /// #     "channel": {
    /// #         "id": ChannelId::new(1),
    /// #         "name": "foo",
    /// #         "type": ChannelType::Text,
    /// #     },
    /// #     "created_at": "2017-01-29T15:35:17.136000+00:00",
    /// #     "guild": {
    /// #         "id": GuildId::new(2),
    /// #         "icon": None::<String>,
    /// #         "name": "baz",
    /// #         "splash_hash": None::<String>,
    /// #         "text_channel_count": None::<u64>,
    /// #         "voice_channel_count": None::<u64>,
    /// #         "features": ["NEWS", "DISCOVERABLE"],
    /// #         "verification_level": 2,
    /// #         "nsfw_level": 0,
    /// #     },
    /// #     "inviter": {
    /// #         "avatar": None::<String>,
    /// #         "bot": false,
    /// #         "discriminator": "1234",
    /// #         "id": UserId::new(4),
    /// #         "username": "qux",
    /// #         "public_flags": None::<UserPublicFlags>,
    /// #     },
    /// #     "max_age": 5,
    /// #     "max_uses": 6,
    /// #     "temporary": true,
    /// #     "uses": 7,
    /// # })).unwrap();
    /// #
    /// assert_eq!(invite.url(), "https://discord.gg/WxZumR");
    /// # }
    /// ```
    #[must_use]
    fn url(&self) -> String;
}
impl RichInviteExt for RichInvite {
    #[cfg(feature = "model")]
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<Invite> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(
                    cache,
                    self.channel.id,
                    Permissions::MANAGE_GUILD,
                )?;
            }
        }
        cache_http.http().as_ref().delete_invite(&self.code, None).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    fn url(&self) -> String {
        format!("https://discord.gg/{}", self.code)
    }
}
