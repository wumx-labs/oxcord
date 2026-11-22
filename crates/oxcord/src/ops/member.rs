#[cfg(feature = "cache")]
use std::cmp::Reverse;

#[cfg(feature = "model")]
use crate::builder::EditMember;
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;
#[cfg(feature = "model")]
use crate::model::utils::{avatar_url, user_banner_url};
#[allow(async_fn_in_trait)]
pub trait MemberExt: Sized {
    #[cfg(feature = "model")]
    /// Adds a [`Role`] to the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a role with the given
    /// Id does not exist.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn add_role(&self, http: impl AsRef<Http>, role_id: impl Into<RoleId>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Adds one or multiple [`Role`]s to the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a role with a given Id
    /// does not exist.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn add_roles(&self, http: impl AsRef<Http>, role_ids: &[RoleId]) -> Result<()>;
    #[cfg(feature = "model")]
    /// Ban a [`User`] from the guild, deleting a number of days' worth of messages (`dmd`) between
    /// the range 0 and 7.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::DeleteMessageDaysAmount`] if the `dmd` is greater than 7. Can also
    /// return [`Error::Http`] if the current user lacks permission to ban this member.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    async fn ban(&self, http: impl AsRef<Http>, dmd: u8) -> Result<()>;
    #[cfg(feature = "model")]
    /// Ban the member from the guild with a reason. Refer to [`Self::ban`] to further
    /// documentation.
    ///
    /// # Errors
    ///
    /// In addition to the errors [`Self::ban`] may return, can also return
    /// [`Error::ExceededLimit`] if the length of the reason is greater than 512.
    async fn ban_with_reason(
        &self,
        http: impl AsRef<Http>,
        dmd: u8,
        reason: impl AsRef<str>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Determines the member's colour.
    #[cfg(feature = "cache")]
    fn colour(&self, cache: impl AsRef<Cache>) -> Option<Colour>;
    #[cfg(feature = "model")]
    /// Returns the "default channel" of the guild for the member. (This returns the first channel
    /// that can be read by the member, if there isn't one returns [`None`])
    #[cfg(feature = "cache")]
    fn default_channel(&self, cache: impl AsRef<Cache>) -> Option<GuildChannel>;
    #[cfg(feature = "model")]
    /// Times the user out until `time`.
    ///
    /// Requires the [Moderate Members] permission.
    ///
    /// **Note**: [Moderate Members]: crate::model::permission::Permissions::MODERATE_MEMBERS
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if `time` is greater than
    /// 28 days from the current time.
    ///
    /// [Moderate Members]: Permissions::MODERATE_MEMBERS
    #[doc(alias = "timeout")]
    async fn disable_communication_until_datetime(
        &mut self,
        cache_http: impl CacheHttp,
        time: Timestamp,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Calculates the member's display name.
    ///
    /// The nickname takes priority over the member's username if it exists.
    #[must_use]
    fn display_name(&self) -> &str;
    #[cfg(feature = "model")]
    /// Returns the DiscordTag of a Member, taking possible nickname into account.
    #[must_use]
    #[deprecated(
        since = "0.0.1",
        note = "Use User::tag to get the correct Discord username format or Self::display_name for the name that users will see."
    )]
    fn distinct(&self) -> String;
    #[cfg(feature = "model")]
    /// Edits the member in place with the given data.
    ///
    /// See [`EditMember`] for the permission(s) required for separate builder methods, as well as
    /// usage of this.
    ///
    /// # Examples
    ///
    /// See [`GuildId::edit_member`] for details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks necessary permissions.
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditMember<'_>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Allow a user to communicate, removing their timeout, if there is one.
    ///
    /// **Note**: Requires the [Moderate Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Moderate Members]: Permissions::MODERATE_MEMBERS
    #[doc(alias = "timeout")]
    async fn enable_communication(&mut self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// Retrieves the ID and position of the member's highest role in the hierarchy, if they have
    /// one.
    ///
    /// This _may_ return [`None`] if the user has roles, but they are not present in the cache for
    /// cache inconsistency reasons.
    ///
    /// The "highest role in hierarchy" is defined as the role with the highest position. If two or
    /// more roles have the same highest position, then the role with the lowest ID is the highest.
    #[cfg(feature = "cache")]
    #[deprecated(since = "0.0.1", note = "Use Guild::member_highest_role")]
    fn highest_role_info(&self, cache: impl AsRef<Cache>) -> Option<(RoleId, u16)>;
    #[cfg(feature = "model")]
    /// Kick the member from the guild.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Examples
    ///
    /// Kick a member from its guild:
    ///
    /// ```rust,ignore
    /// // assuming a `member` has already been bound
    /// match member.kick().await {
    ///     Ok(()) => println!("Successfully kicked member"),
    ///     Err(Error::Model(ModelError::GuildNotFound)) => {
    ///         println!("Couldn't determine guild of member");
    ///     },
    ///     Err(Error::Model(ModelError::InvalidPermissions(missing_perms))) => {
    ///         println!("Didn't have permissions; missing: {:?}", missing_perms);
    ///     },
    ///     _ => {},
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the Id of the member's guild could not be
    /// determined.
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to perform the kick.
    ///
    /// Otherwise will return [`Error::Http`] if the current user lacks permission.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    async fn kick(&self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// Kicks the member from the guild, with a reason.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Examples
    ///
    /// Kicks a member from it's guild, with an optional reason:
    ///
    /// ```rust,ignore
    /// match member.kick(&ctx.http, "A Reason").await {
    ///     Ok(()) => println!("Successfully kicked member"),
    ///     Err(Error::Model(ModelError::GuildNotFound)) => {
    ///         println!("Couldn't determine guild of member");
    ///     },
    ///     Err(Error::Model(ModelError::InvalidPermissions(missing_perms))) => {
    ///         println!("Didn't have permissions; missing: {:?}", missing_perms);
    ///     },
    ///     _ => {},
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// In addition to the reasons [`Self::kick`] may return an error, can also return an error if
    /// the given reason is too long.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    async fn kick_with_reason(&self, cache_http: impl CacheHttp, reason: &str) -> Result<()>;
    #[cfg(feature = "model")]
    /// Moves the member to a voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member is not currently in a voice channel, or if the
    /// current user lacks permission.
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    async fn move_to_voice_channel(
        &self,
        cache_http: impl CacheHttp,
        channel: impl Into<ChannelId>,
    ) -> Result<Member>;
    #[cfg(feature = "model")]
    /// Disconnects the member from their voice channel if any.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member is not currently in a voice channel, or if the
    /// current user lacks permission.
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    async fn disconnect_from_voice(&self, cache_http: impl CacheHttp) -> Result<Member>;
    #[cfg(feature = "model")]
    /// Returns the guild-level permissions for the member.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // assuming there's a `member` variable gotten from anything.
    /// println!("The permission bits for the member are: {}",
    /// member.permissions(&cache).expect("permissions").bits());
    /// ```
    ///
    /// # Note
    ///
    /// You likely want to use Guild::user_permissions_in instead as this function does not consider
    /// permission overwrites.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the guild the member's in could not be
    /// found in the cache.
    ///
    /// And/or returns [`ModelError::ItemMissing`] if the "default channel" of the guild is not
    /// found.
    #[cfg(feature = "cache")]
    #[deprecated(
        since = "0.0.1",
        note = "Use Guild::user_permissions_in, as this doesn't consider permission overwrites"
    )]
    fn permissions(&self, cache: impl AsRef<Cache>) -> Result<Permissions>;
    #[cfg(feature = "model")]
    /// Removes a [`Role`] from the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a role with the given Id does not exist, or if the current user
    /// lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn remove_role(&self, http: impl AsRef<Http>, role_id: impl Into<RoleId>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Removes one or multiple [`Role`]s from the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a role with a given Id does not exist, or if the current user
    /// lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn remove_roles(&self, http: impl AsRef<Http>, role_ids: &[RoleId]) -> Result<()>;
    #[cfg(feature = "model")]
    /// Retrieves the full role data for the user's roles.
    ///
    /// This is shorthand for manually searching through the Cache.
    ///
    /// If role data can not be found for the member, then [`None`] is returned.
    #[cfg(feature = "cache")]
    fn roles(&self, cache: impl AsRef<Cache>) -> Option<Vec<Role>>;
    #[cfg(feature = "model")]
    /// Unbans the [`User`] from the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to perform bans.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    async fn unban(&self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Returns the formatted URL of the member's per guild avatar, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the member has a GIF avatar.
    #[must_use]
    fn avatar_url(&self) -> Option<String>;
    #[cfg(feature = "model")]
    /// Returns the formatted URL of the member's per guild banner, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the member has a GIF avatar.
    #[must_use]
    fn banner_url(&self) -> Option<String>;
    #[cfg(feature = "model")]
    /// Retrieves the URL to the current member's avatar, falling back to the user's avatar, then
    /// default avatar if needed.
    ///
    /// This will call [`Self::avatar_url`] first, and if that returns [`None`], it then falls back
    /// to [`User::face()`].
    #[must_use]
    fn face(&self) -> String;
}
impl MemberExt for Member {
    #[cfg(feature = "model")]
    #[inline]
    async fn add_role(&self, http: impl AsRef<Http>, role_id: impl Into<RoleId>) -> Result<()> {
        http.as_ref()
            .add_member_role(self.guild_id, self.user.id, role_id.into(), None)
            .await
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn add_roles(&self, http: impl AsRef<Http>, role_ids: &[RoleId]) -> Result<()> {
        for role_id in role_ids {
            self.add_role(http.as_ref(), role_id).await?;
        }
        Ok(())
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn ban(&self, http: impl AsRef<Http>, dmd: u8) -> Result<()> {
        self.ban_with_reason(http, dmd, "").await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn ban_with_reason(
        &self,
        http: impl AsRef<Http>,
        dmd: u8,
        reason: impl AsRef<str>,
    ) -> Result<()> {
        self.guild_id.ban_with_reason(http, self.user.id, dmd, reason).await
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn colour(&self, cache: impl AsRef<Cache>) -> Option<Colour> {
        let guild = cache.as_ref().guild(self.guild_id)?;
        let mut roles = self
            .roles
            .iter()
            .filter_map(|role_id| guild.roles.get(role_id))
            .collect::<Vec<&Role>>();
        roles.sort_by_key(|&b| Reverse(b));
        let default = Colour::default();
        roles.iter().find(|r| r.colour.0 != default.0).map(|r| r.colour)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn default_channel(&self, cache: impl AsRef<Cache>) -> Option<GuildChannel> {
        let guild = self.guild_id.to_guild_cached(&cache)?;
        let member = guild.members.get(&self.user.id)?;
        for channel in guild.channels.values() {
            if channel.kind != ChannelType::Category
                && guild.user_permissions_in(channel, member).view_channel()
            {
                return Some(channel.clone());
            }
        }
        None
    }
    #[cfg(feature = "model")]
    async fn disable_communication_until_datetime(
        &mut self,
        cache_http: impl CacheHttp,
        time: Timestamp,
    ) -> Result<()> {
        let builder = EditMember::new().disable_communication_until_datetime(time);
        match self.guild_id.edit_member(cache_http, self.user.id, builder).await {
            Ok(_) => {
                self.communication_disabled_until = Some(time);
                Ok(())
            },
            Err(why) => Err(why),
        }
    }
    #[cfg(feature = "model")]
    #[inline]
    fn display_name(&self) -> &str {
        self.nick.as_ref().or(self.user.global_name.as_ref()).unwrap_or(&self.user.name)
    }
    #[cfg(feature = "model")]
    #[inline]
    fn distinct(&self) -> String {
        if let Some(discriminator) = self.user.discriminator {
            format!("{}#{:04}", self.display_name(), discriminator.get())
        } else {
            self.display_name().to_string()
        }
    }
    #[cfg(feature = "model")]
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditMember<'_>) -> Result<()> {
        *self = self.guild_id.edit_member(cache_http, self.user.id, builder).await?;
        Ok(())
    }
    #[cfg(feature = "model")]
    async fn enable_communication(&mut self, cache_http: impl CacheHttp) -> Result<()> {
        let builder = EditMember::new().enable_communication();
        *self = self.guild_id.edit_member(cache_http, self.user.id, builder).await?;
        Ok(())
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn highest_role_info(&self, cache: impl AsRef<Cache>) -> Option<(RoleId, u16)> {
        cache
            .as_ref()
            .guild(self.guild_id)
            .as_ref()
            .and_then(|g| g.member_highest_role(self))
            .map(|r| (r.id, r.position))
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn kick(&self, cache_http: impl CacheHttp) -> Result<()> {
        self.kick_with_reason(cache_http, "").await
    }
    #[cfg(feature = "model")]
    async fn kick_with_reason(&self, cache_http: impl CacheHttp, reason: &str) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let lookup = cache.guild(self.guild_id).as_deref().cloned();
                if let Some(guild) = lookup {
                    guild.require_perms(cache, Permissions::KICK_MEMBERS)?;
                    guild.check_hierarchy(cache, self.user.id)?;
                }
            }
        }
        self.guild_id.kick_with_reason(cache_http.http(), self.user.id, reason).await
    }
    #[cfg(feature = "model")]
    async fn move_to_voice_channel(
        &self,
        cache_http: impl CacheHttp,
        channel: impl Into<ChannelId>,
    ) -> Result<Member> {
        self.guild_id.move_member(cache_http, self.user.id, channel).await
    }
    #[cfg(feature = "model")]
    async fn disconnect_from_voice(&self, cache_http: impl CacheHttp) -> Result<Member> {
        self.guild_id.disconnect_member(cache_http, self.user.id).await
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn permissions(&self, cache: impl AsRef<Cache>) -> Result<Permissions> {
        let guild = cache.as_ref().guild(self.guild_id).ok_or(ModelError::GuildNotFound)?;
        #[allow(deprecated)]
        Ok(guild.member_permissions(self))
    }
    #[cfg(feature = "model")]
    async fn remove_role(&self, http: impl AsRef<Http>, role_id: impl Into<RoleId>) -> Result<()> {
        http.as_ref()
            .remove_member_role(self.guild_id, self.user.id, role_id.into(), None)
            .await
            .map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn remove_roles(&self, http: impl AsRef<Http>, role_ids: &[RoleId]) -> Result<()> {
        for role_id in role_ids {
            self.remove_role(http.as_ref(), role_id).await?;
        }
        Ok(())
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn roles(&self, cache: impl AsRef<Cache>) -> Option<Vec<Role>> {
        Some(
            cache
                .as_ref()
                .guild(self.guild_id)?
                .roles
                .iter()
                .filter(|(id, _)| self.roles.contains(id))
                .map(|(_, role)| role.clone())
                .collect(),
        )
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn unban(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().remove_ban(self.guild_id, self.user.id, None).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    #[inline]
    fn avatar_url(&self) -> Option<String> {
        avatar_url(Some(self.guild_id), self.user.id, self.avatar.as_ref())
    }
    #[cfg(feature = "model")]
    fn banner_url(&self) -> Option<String> {
        user_banner_url(Some(self.guild_id), self.user.id, self.banner.as_ref())
    }
    #[cfg(feature = "model")]
    #[inline]
    fn face(&self) -> String {
        self.avatar_url().unwrap_or_else(|| self.user.face())
    }
}
