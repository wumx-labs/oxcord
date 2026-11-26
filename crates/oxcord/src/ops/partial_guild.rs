#[cfg(feature = "model")]
use crate::builder::{
    CreateChannel,
    CreateCommand,
    CreateSoundboard,
    CreateSticker,
    EditAutoModRule,
    EditCommandPermissions,
    EditGuild,
    EditGuildWelcomeScreen,
    EditGuildWidget,
    EditMember,
    EditRole,
    EditSoundboard,
    EditSticker,
};
#[cfg(all(feature = "cache", feature = "utils", feature = "client"))]
use crate::cache::Cache;
#[cfg(feature = "collector")]
use crate::collector::{MessageCollector, ReactionCollector};
#[cfg(feature = "collector")]
use crate::gateway::ShardMessenger;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http, UserPagination};
use crate::model::prelude::*;
#[cfg(feature = "model")]
use crate::model::utils::icon_url;
#[allow(async_fn_in_trait)]
pub trait PartialGuildExt: Sized {
    #[cfg(feature = "model")]
    /// Gets all auto moderation [`Rule`]s of this guild via HTTP.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the guild is unavailable.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn automod_rules(self, http: impl AsRef<Http>) -> Result<Vec<Rule>>;
    #[cfg(feature = "model")]
    /// Gets an auto moderation [`Rule`] of this guild by its ID via HTTP.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a rule with the given ID does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn automod_rule(
        &self,
        http: impl AsRef<Http>,
        rule_id: impl Into<RuleId>,
    ) -> Result<Rule>;
    #[cfg(feature = "model")]
    /// Creates an auto moderation [`Rule`] in the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Examples
    ///
    /// See [`GuildId::create_automod_rule`] for details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn create_automod_rule(
        &self,
        cache_http: impl CacheHttp,
        builder: EditAutoModRule<'_>,
    ) -> Result<Rule>;
    #[cfg(feature = "model")]
    /// Edit an auto moderation [`Rule`], given its Id.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn edit_automod_rule(
        &self,
        cache_http: impl CacheHttp,
        rule_id: impl Into<RuleId>,
        builder: EditAutoModRule<'_>,
    ) -> Result<Rule>;
    #[cfg(feature = "model")]
    /// Deletes an auto moderation [`Rule`] from the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a rule with that Id
    /// does not exist.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn delete_automod_rule(
        &self,
        http: impl AsRef<Http>,
        rule_id: impl Into<RuleId>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Ban a [`User`] from the guild, deleting a number of days' worth of messages (`dmd`) between
    /// the range 0 and 7.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Examples
    ///
    /// Ban a member and remove all messages they've sent in the last 4 days:
    ///
    /// ```rust,ignore
    /// // assumes a `user` and `guild` have already been bound
    /// let _ = guild.ban(user, 4);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::DeleteMessageDaysAmount`] if the number of days' worth of messages
    /// to delete is over the maximum.
    ///
    /// Also may return [`Error::Http`] if the current user lacks permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    async fn ban(&self, http: impl AsRef<Http>, user: impl Into<UserId>, dmd: u8) -> Result<()>;
    #[cfg(feature = "model")]
    /// Ban a [`User`] from the guild with a reason. Refer to [`Self::ban`] to further
    /// documentation.
    ///
    /// # Errors
    ///
    /// In addition to the reasons [`Self::ban`] may return an error, can also return an error if
    /// the reason is too long.
    async fn ban_with_reason(
        &self,
        http: impl AsRef<Http>,
        user: impl Into<UserId>,
        dmd: u8,
        reason: impl AsRef<str>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Gets a list of the guild's bans, with additional options and filtering. See
    /// [`Http::get_bans`] for details.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    async fn bans(
        &self,
        http: impl AsRef<Http>,
        target: Option<UserPagination>,
        limit: Option<u8>,
    ) -> Result<Vec<Ban>>;
    #[cfg(feature = "model")]
    /// Gets a user's ban from the guild.
    /// See [`Http::get_bans`] for details.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    async fn get_ban(&self, http: impl AsRef<Http>, user_id: UserId) -> Result<Option<Ban>>;
    #[cfg(feature = "model")]
    /// Gets a list of the guild's audit log entries
    ///
    /// **Note**: Requires the [View Audit Log] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an invalid value is
    /// given.
    ///
    /// [View Audit Log]: Permissions::VIEW_AUDIT_LOG
    async fn audit_logs(
        &self,
        http: impl AsRef<Http>,
        action_type: Option<audit_log::Action>,
        user_id: Option<UserId>,
        before: Option<AuditLogEntryId>,
        limit: Option<u8>,
    ) -> Result<AuditLogs>;
    #[cfg(feature = "model")]
    /// Gets all of the guild's channels over the REST API.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not in the guild or if the guild is
    /// otherwise unavailable.
    async fn channels(&self, http: impl AsRef<Http>) -> Result<HashMap<ChannelId, GuildChannel>>;
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    #[deprecated(since = "0.0.1", note = "Use Cache::guild and Guild::channels")]
    fn channel_id_from_name(
        &self,
        cache: impl AsRef<Cache>,
        name: impl AsRef<str>,
    ) -> Option<ChannelId>;
    #[cfg(feature = "model")]
    /// Creates a [`GuildChannel`] in the guild.
    ///
    /// Refer to [`Http::create_channel`] for more information.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Create a voice channel in a guild with the name `test`:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::http::Http;
    /// # use oxcord::model::guild::PartialGuild;
    /// # use oxcord::model::id::GuildId;
    /// use oxcord::builder::CreateChannel;
    /// use oxcord::model::channel::ChannelType;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let guild = PartialGuild::get(&http, GuildId::new(7)).await?;
    /// let builder = CreateChannel::new("my-test-channel").kind(ChannelType::Text);
    ///
    /// // assuming a `guild` has already been bound
    /// let _channel = guild.create_channel(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    async fn create_channel(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateChannel<'_>,
    ) -> Result<GuildChannel>;
    #[cfg(feature = "model")]
    /// Creates an emoji in the guild with a name and base64-encoded image.
    ///
    /// Refer to the documentation for [`Guild::create_emoji`] for more information.
    ///
    /// Requires the [Create Guild Expressions] permission.
    ///
    /// # Examples
    ///
    /// See the [`EditProfile::avatar`] example for an in-depth example as to how to read an image
    /// from the filesystem and encode it as base64. Most of the example can be applied similarly
    /// for this method.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, if the emoji name is too
    /// long, or if the image is too large.
    ///
    /// [`EditProfile::avatar`]: crate::builder::EditProfile::avatar
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    async fn create_emoji(&self, http: impl AsRef<Http>, name: &str, image: &str) -> Result<Emoji>;
    #[cfg(feature = "model")]
    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn create_integration(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
        kind: &str,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Create a guild specific application [`Command`].
    ///
    /// **Note**: Unlike global commands, guild commands will update instantly.
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    ///
    /// [`CreateCommand::execute`]: ../../builder/struct.CreateCommand.html#method.execute
    async fn create_command(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateCommand,
    ) -> Result<Command>;
    #[cfg(feature = "model")]
    /// Override all guild application commands.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::create_command`].
    async fn set_commands(
        &self,
        http: impl AsRef<Http>,
        commands: Vec<CreateCommand>,
    ) -> Result<Vec<Command>>;
    #[cfg(feature = "model")]
    /// Overwrites permissions for a specific command.
    ///
    /// **Note**: It will update instantly.
    ///
    /// # Errors
    ///
    /// See [`CreateCommandPermissionsData::execute`] for a list of possible errors.
    ///
    /// [`CreateCommandPermissionsData::execute`]: ../../builder/struct.CreateCommandPermissionsData.html#method.execute
    async fn edit_command_permissions(
        &self,
        cache_http: impl CacheHttp,
        command_id: CommandId,
        builder: EditCommandPermissions,
    ) -> Result<CommandPermissions>;
    #[cfg(feature = "model")]
    /// Get all guild application commands.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    async fn get_commands(&self, http: impl AsRef<Http>) -> Result<Vec<Command>>;
    #[cfg(feature = "model")]
    /// Get all guild application commands with localizations.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    async fn get_commands_with_localizations(&self, http: impl AsRef<Http>)
    -> Result<Vec<Command>>;
    #[cfg(feature = "model")]
    /// Get a specific guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    async fn get_command(&self, http: impl AsRef<Http>, command_id: CommandId) -> Result<Command>;
    #[cfg(feature = "model")]
    /// Edit a guild application command, given its Id.
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    ///
    /// [`CreateCommand::execute`]: ../../builder/struct.CreateCommand.html#method.execute
    async fn edit_command(
        &self,
        cache_http: impl CacheHttp,
        command_id: CommandId,
        builder: CreateCommand,
    ) -> Result<Command>;
    #[cfg(feature = "model")]
    /// Delete guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    async fn delete_command(&self, http: impl AsRef<Http>, command_id: CommandId) -> Result<()>;
    #[cfg(feature = "model")]
    /// Get all guild application commands permissions only.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    async fn get_commands_permissions(
        &self,
        http: impl AsRef<Http>,
    ) -> Result<Vec<CommandPermissions>>;
    #[cfg(feature = "model")]
    /// Get permissions for specific guild application command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    async fn get_command_permissions(
        &self,
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<CommandPermissions>;
    #[cfg(feature = "model")]
    /// Creates a new role in the guild with the data set, if any.
    ///
    /// See the documentation for [`Guild::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn create_role(&self, cache_http: impl CacheHttp, builder: EditRole<'_>) -> Result<Role>;
    #[cfg(feature = "model")]
    /// Creates a new sticker in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Create Guild Expressions] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    async fn create_sticker(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateSticker<'_>,
    ) -> Result<Sticker>;
    #[cfg(feature = "model")]
    /// Deletes the current guild if the current user is the owner of the
    /// guild.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not the owner of
    /// the guild.
    async fn delete(&self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes an [`Emoji`] from the guild.
    ///
    /// **Note**: If the emoji was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an emoji with the given
    /// id does not exist in the guild.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    async fn delete_emoji(
        &self,
        http: impl AsRef<Http>,
        emoji_id: impl Into<EmojiId>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an integration with
    /// that Id does not exist in the guild.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn delete_integration(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes a [`Role`] by Id from the guild.
    ///
    /// Also see [`Role::delete`] if you have the `cache` and `model` features enabled.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a Role with that Id
    /// does not exist in the Guild.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn delete_role(&self, http: impl AsRef<Http>, role_id: impl Into<RoleId>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Deletes a [`Sticker`] by Id from the guild.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a sticker with that id
    /// does not exist.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    async fn delete_sticker(
        &self,
        http: impl AsRef<Http>,
        sticker_id: impl Into<StickerId>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits the current guild with new data where specified.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditGuild<'_>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits an [`Emoji`]'s name in the guild.
    ///
    /// Also see [`Emoji::edit`] if you have the `cache` and `methods` features enabled.
    ///
    /// **Note**: If the emoji was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if an emoji with the given
    /// id does not exist.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    async fn edit_emoji(
        &self,
        http: impl AsRef<Http>,
        emoji_id: impl Into<EmojiId>,
        name: &str,
    ) -> Result<Emoji>;
    #[cfg(feature = "model")]
    /// Edits the properties a guild member, such as muting or nicknaming them. Returns the new
    /// member.
    ///
    /// Refer to the documentation of [`EditMember`] for a full list of methods and permission
    /// restrictions.
    ///
    /// # Examples
    ///
    /// See [`GuildId::edit_member`] for details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    async fn edit_member(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
        builder: EditMember<'_>,
    ) -> Result<Member>;
    #[cfg(feature = "model")]
    /// Edits the guild's MFA level. Returns the new level on success.
    ///
    /// Requires guild ownership.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    async fn edit_mfa_level(
        &self,
        http: impl AsRef<Http>,
        mfa_level: MfaLevel,
        audit_log_reason: Option<&str>,
    ) -> Result<MfaLevel>;
    #[cfg(feature = "model")]
    /// Edits the current user's nickname for the guild.
    ///
    /// Pass [`None`] to reset the nickname.
    ///
    /// **Note**: Requires the [Change Nickname] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to change their nickname.
    ///
    /// [Change Nickname]: Permissions::CHANGE_NICKNAME
    async fn edit_nickname(&self, http: impl AsRef<Http>, new_nickname: Option<&str>)
    -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits a role, optionally setting its fields.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// See the documentation of [`GuildId::edit_role`] for details.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn edit_role(
        &self,
        cache_http: impl CacheHttp,
        role_id: impl Into<RoleId>,
        builder: EditRole<'_>,
    ) -> Result<Role>;
    #[cfg(feature = "model")]
    /// Edits the order of [`Role`]s. Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Change the order of a role:
    ///
    /// ```rust,ignore
    /// use oxcord::model::id::RoleId;
    /// partial_guild.edit_role_position(&context, RoleId::new(8), 2);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn edit_role_position(
        &self,
        http: impl AsRef<Http>,
        role_id: impl Into<RoleId>,
        position: u16,
    ) -> Result<Vec<Role>>;
    #[cfg(feature = "model")]
    /// Edits a sticker.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Examples
    ///
    /// Rename a sticker:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::http::Http;
    /// # use oxcord::model::guild::PartialGuild;
    /// # use oxcord::model::id::GuildId;
    /// use oxcord::builder::EditSticker;
    /// use oxcord::model::id::StickerId;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let guild = PartialGuild::get(&http, GuildId::new(7)).await?;
    /// let builder = EditSticker::new().name("Bun bun meow");
    /// guild.edit_sticker(&http, StickerId::new(7), builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    async fn edit_sticker(
        &self,
        cache_http: impl CacheHttp,
        sticker_id: impl Into<StickerId>,
        builder: EditSticker<'_>,
    ) -> Result<Sticker>;
    #[cfg(feature = "model")]
    /// Edits the guild's welcome screen.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn edit_welcome_screen(
        &self,
        cache_http: impl CacheHttp,
        builder: EditGuildWelcomeScreen<'_>,
    ) -> Result<GuildWelcomeScreen>;
    #[cfg(feature = "model")]
    /// Edits the guild's widget.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn edit_widget(
        &self,
        cache_http: impl CacheHttp,
        builder: EditGuildWidget<'_>,
    ) -> Result<GuildWidget>;
    #[cfg(feature = "model")]
    /// Gets a partial amount of guild data by its Id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is not
    /// in the guild.
    async fn get(cache_http: impl CacheHttp, guild_id: impl Into<GuildId>) -> Result<PartialGuild>;
    #[cfg(feature = "model")]
    /// Returns which of two [`User`]s has a higher [`Member`] hierarchy.
    ///
    /// Hierarchy is essentially who has the [`Role`] with the highest [`position`].
    ///
    /// Returns [`None`] if at least one of the given users' member instances is not present.
    /// Returns [`None`] if the users have the same hierarchy, as neither are greater than the
    /// other.
    ///
    /// If both user IDs are the same, [`None`] is returned. If one of the users is the guild
    /// owner, their ID is returned.
    ///
    /// [`position`]: Role::position
    #[cfg(feature = "cache")]
    #[deprecated(since = "0.0.1", note = "Use Cache::guild and Guild::greater_member_hierarchy")]
    fn greater_member_hierarchy(
        &self,
        cache: impl AsRef<Cache>,
        lhs_id: impl Into<UserId>,
        rhs_id: impl Into<UserId>,
    ) -> Option<UserId>;
    #[cfg(feature = "model")]
    /// Calculate a [`Member`]'s permissions in the guild.
    ///
    /// You likely want to use PartialGuild::user_permissions_in instead as this function does not
    /// consider permission overwrites.
    #[must_use]
    fn member_permissions(&self, member: &Member) -> Permissions;
    #[cfg(feature = "model")]
    /// Calculate a [`PartialMember`]'s permissions in the guild.
    ///
    /// You likely want to use PartialGuild::partial_member_permissions_in instead as this function
    /// does not consider permission overwrites.
    ///
    /// # Panics
    ///
    /// Panics if the passed [`UserId`] does not match the [`PartialMember`] id, if user is Some.
    #[must_use]
    fn partial_member_permissions(&self, member_id: UserId, member: &PartialMember) -> Permissions;
    #[cfg(feature = "model")]
    /// Calculate a [`PartialMember`]'s permissions in a given channel in a guild.
    ///
    /// # Panics
    ///
    /// Panics if the passed [`UserId`] does not match the [`PartialMember`] id, if user is Some.
    #[must_use]
    fn partial_member_permissions_in(
        &self,
        channel: &GuildChannel,
        member_id: UserId,
        member: &PartialMember,
    ) -> Permissions;
    #[cfg(feature = "model")]
    /// Re-orders the channels of the guild.
    ///
    /// Although not required, you should specify all channels' positions, regardless of whether
    /// they were updated. Otherwise, positioning can sometimes get weird.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is lacking permission.
    ///
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    async fn reorder_channels(
        &self,
        http: impl AsRef<Http>,
        channels: impl IntoIterator<Item = (ChannelId, u64)>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Returns a list of [`Member`]s in a [`Guild`] whose username or nickname starts with a
    /// provided string.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Minimum value is 1, maximum
    /// and default value is 1000.
    ///
    /// **Note**: Queries are case insensitive.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error.
    async fn search_members(
        &self,
        http: impl AsRef<Http>,
        query: &str,
        limit: Option<u64>,
    ) -> Result<Vec<Member>>;
    #[cfg(feature = "model")]
    /// Starts a prune of [`Member`]s.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires [Kick Members] and [Manage Guild] permissions.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to kick members.
    ///
    /// Otherwise will return [`Error::Http`] if the current user does not have permission.
    ///
    /// Can also return an [`Error::Json`] if there is an error deserializing the API response.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    async fn start_prune(&self, cache_http: impl CacheHttp, days: u8) -> Result<GuildPrune>;
    #[cfg(feature = "model")]
    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member cannot be kicked by the current user.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    async fn kick(&self, http: impl AsRef<Http>, user_id: impl Into<UserId>) -> Result<()>;
    #[cfg(feature = "model")]
    /// # Errors
    ///
    /// In addition to the reasons [`Self::kick`] may return an error, can also return an error if
    /// the reason is too long.
    async fn kick_with_reason(
        &self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        reason: &str,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Returns a formatted URL of the guild's icon, if the guild has an icon.
    #[must_use]
    fn icon_url(&self) -> Option<String>;
    #[cfg(feature = "model")]
    /// Returns a formatted URL of the guild's banner, if the guild has a banner.
    #[must_use]
    fn banner_url(&self) -> Option<String>;
    #[cfg(feature = "model")]
    /// Gets all [`Emoji`]s of this guild via HTTP.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the guild is unavailable.
    async fn emojis(&self, http: impl AsRef<Http>) -> Result<Vec<Emoji>>;
    #[cfg(feature = "model")]
    /// Gets an [`Emoji`] of this guild by its ID via HTTP.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if an [`Emoji`] with the given Id does not exist for the guild.
    async fn emoji(&self, http: impl AsRef<Http>, emoji_id: EmojiId) -> Result<Emoji>;
    #[cfg(feature = "model")]
    /// Gets all integration of the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn integrations(&self, http: impl AsRef<Http>) -> Result<Vec<Integration>>;
    #[cfg(feature = "model")]
    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn invites(&self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>>;
    #[cfg(feature = "model")]
    /// Returns a guild [`Member`] object for the current user.
    ///
    /// See [`Http::get_current_user_guild_member`] for more.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user is not in the guild or the access token
    /// lacks the necessary scope.
    async fn current_user_member(&self, http: impl AsRef<Http>) -> Result<Member>;
    #[cfg(feature = "model")]
    /// Leaves the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user is unable to leave the Guild, or currently is
    /// not in the guild.
    async fn leave(&self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member is not in the Guild, or if the Guild is otherwise
    /// unavailable.
    async fn member(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
    ) -> Result<Member>;
    #[cfg(feature = "model")]
    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Minimum value is 1, maximum
    /// and default value is 1000.
    ///
    /// Optionally pass in `after` to offset the results by a [`User`]'s Id.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error, may also return
    /// [`Error::NotInRange`] if the input is not within range.
    ///
    /// [`User`]: crate::model::user::User
    async fn members(
        &self,
        http: impl AsRef<Http>,
        limit: Option<u64>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<Member>>;
    #[cfg(feature = "model")]
    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the current user lacks permission, or if the member is not
    /// currently in a voice channel for this Guild.
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    async fn move_member(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
        channel_id: impl Into<ChannelId>,
    ) -> Result<Member>;
    #[cfg(feature = "model")]
    /// Calculate a [`Member`]'s permissions in a given channel in the guild.
    #[must_use]
    fn user_permissions_in(&self, channel: &GuildChannel, member: &Member) -> Permissions;
    #[cfg(feature = "model")]
    /// Calculate a [`Role`]'s permissions in a given channel in the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the [`Role`] or [`Channel`] is not from this [`Guild`].
    #[deprecated(
        since = "0.0.1",
        note = "this function ignores other roles the user may have as well as user-specific permissions; use user_permissions_in instead"
    )]
    fn role_permissions_in(&self, channel: &GuildChannel, role: &Role) -> Result<Permissions>;
    #[cfg(feature = "model")]
    /// Gets the number of [`Member`]s that would be pruned with the given number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// See [`Guild::prune_count`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    /// [`Guild::prune_count`]: crate::model::guild::Guild::prune_count
    async fn prune_count(&self, http: impl AsRef<Http>, days: u8) -> Result<GuildPrune>;
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
    #[cfg(feature = "model")]
    /// Returns the formatted URL of the guild's splash image, if one exists.
    #[must_use]
    fn splash_url(&self) -> Option<String>;
    #[cfg(feature = "model")]
    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// See [`Guild::start_integration_sync`].
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [`Guild::start_integration_sync`]: crate::model::guild::Guild::start_integration_sync
    async fn start_integration_sync(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
    ) -> Result<()>;
    #[cfg(feature = "model")]
    /// Unbans a [`User`] from the guild.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// See [`Guild::unban`].
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    /// [`Guild::unban`]: crate::model::guild::Guild::unban
    async fn unban(&self, http: impl AsRef<Http>, user_id: impl Into<UserId>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// See [`Guild::vanity_url`].
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [`Guild::vanity_url`]: crate::model::guild::Guild::vanity_url
    async fn vanity_url(&self, http: impl AsRef<Http>) -> Result<String>;
    #[cfg(feature = "model")]
    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// # Errors
    ///
    /// See [`Guild::webhooks`].
    ///
    /// [Manage Webhooks]: Permissions::MANAGE_WEBHOOKS
    /// [`Guild::webhooks`]: crate::model::guild::Guild::webhooks
    async fn webhooks(&self, http: impl AsRef<Http>) -> Result<Vec<Webhook>>;
    #[cfg(feature = "model")]
    /// Obtain a reference to a role by its name.
    ///
    /// **Note**: If two or more roles have the same name, obtained reference will be one of them.
    ///
    /// # Examples
    ///
    /// Obtain a reference to a [`Role`] by its name.
    ///
    /// ```rust,no_run
    /// # use oxcord::model::prelude::*;
    /// # use oxcord::prelude::*;
    /// # struct Handler;
    ///
    /// #[oxcord::async_trait]
    /// #[cfg(all(feature = "cache", feature = "client"))]
    /// impl EventHandler for Handler {
    ///     async fn message(&self, context: Context, msg: Message) {
    ///         if let Some(guild_id) = msg.guild_id {
    ///             if let Some(guild) = guild_id.to_guild_cached(&context) {
    ///                 if let Some(role) = guild.role_by_name("role_name") {
    ///                     println!("Obtained role's reference: {:?}", role);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[must_use]
    fn role_by_name(&self, role_name: &str) -> Option<&Role>;
    #[cfg(feature = "model")]
    /// Returns a builder which can be awaited to obtain a message or stream of messages in this
    /// guild.
    #[cfg(feature = "collector")]
    fn await_reply(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector;
    #[cfg(feature = "model")]
    /// Same as [`Self::await_reply`].
    #[cfg(feature = "collector")]
    fn await_replies(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector;
    #[cfg(feature = "model")]
    /// Returns a builder which can be awaited to obtain a message or stream of reactions sent in
    /// this guild.
    #[cfg(feature = "collector")]
    fn await_reaction(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector;
    #[cfg(feature = "model")]
    /// Same as [`Self::await_reaction`].
    #[cfg(feature = "collector")]
    fn await_reactions(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector;
    #[cfg(feature = "model")]
    /// Gets the guild active threads.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if there is an error in the deserialization, or if the bot issuing
    /// the request is not in the guild.
    async fn get_active_threads(&self, http: impl AsRef<Http>) -> Result<ThreadsData>;
    #[cfg(feature = "model")]
    /// Gets a soundboard sound from the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if there is an error in the deserialization, or if the bot issuing
    /// the request is not in the guild.
    async fn get_soundboard(self, http: impl AsRef<Http>, sound_id: SoundId) -> Result<Soundboard>;
    #[cfg(feature = "model")]
    /// Gets all soundboard sounds from the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if there is an error in the deserialization, or if the bot issuing
    /// the request is not in the guild.
    async fn get_soundboards(self, http: impl AsRef<Http>) -> Result<Vec<Soundboard>>;
    #[cfg(feature = "model")]
    /// Creates a soundboard sound for the guild.
    ///
    /// # Errors
    ///
    /// See [`CreateSoundboard::execute`] for a list of possible errors.
    ///
    /// [`CreateSoundboard::execute`]: ../../builder/struct.CreateSoundboard.html#method.execute
    async fn create_soundboard(
        self,
        cache_http: impl CacheHttp,
        builder: CreateSoundboard<'_>,
    ) -> Result<Soundboard>;
    #[cfg(feature = "model")]
    /// Edits a soundboard sound for the guild.
    ///
    /// # Errors
    ///
    /// See [`EditSoundboard::execute`] for a list of possible errors.
    ///
    /// [`EditSoundboard::execute`]: ../../builder/struct.EditSoundboard.html#method.execute
    async fn edit_soundboard(
        self,
        cache_http: impl CacheHttp,
        sound_id: SoundId,
        builder: EditSoundboard<'_>,
    ) -> Result<Soundboard>;
    #[cfg(feature = "model")]
    /// Deletes a soundboard sound for the guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a
    /// soundboard sound with that Id does not exist.
    async fn delete_soundboard(
        self,
        http: impl AsRef<Http>,
        sound_id: SoundId,
        audit_log_reason: Option<&str>,
    ) -> Result<()>;
}
impl PartialGuildExt for PartialGuild {
    #[cfg(feature = "model")]
    #[inline]
    async fn automod_rules(self, http: impl AsRef<Http>) -> Result<Vec<Rule>> {
        self.id.automod_rules(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn automod_rule(
        &self,
        http: impl AsRef<Http>,
        rule_id: impl Into<RuleId>,
    ) -> Result<Rule> {
        self.id.automod_rule(http, rule_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn create_automod_rule(
        &self,
        cache_http: impl CacheHttp,
        builder: EditAutoModRule<'_>,
    ) -> Result<Rule> {
        self.id.create_automod_rule(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn edit_automod_rule(
        &self,
        cache_http: impl CacheHttp,
        rule_id: impl Into<RuleId>,
        builder: EditAutoModRule<'_>,
    ) -> Result<Rule> {
        self.id.edit_automod_rule(cache_http, rule_id, builder).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete_automod_rule(
        &self,
        http: impl AsRef<Http>,
        rule_id: impl Into<RuleId>,
    ) -> Result<()> {
        self.id.delete_automod_rule(http, rule_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn ban(&self, http: impl AsRef<Http>, user: impl Into<UserId>, dmd: u8) -> Result<()> {
        self.ban_with_reason(http, user, dmd, "").await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn ban_with_reason(
        &self,
        http: impl AsRef<Http>,
        user: impl Into<UserId>,
        dmd: u8,
        reason: impl AsRef<str>,
    ) -> Result<()> {
        self.id.ban_with_reason(http, user, dmd, reason).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn bans(
        &self,
        http: impl AsRef<Http>,
        target: Option<UserPagination>,
        limit: Option<u8>,
    ) -> Result<Vec<Ban>> {
        self.id.bans(http, target, limit).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn get_ban(&self, http: impl AsRef<Http>, user_id: UserId) -> Result<Option<Ban>> {
        self.id.get_ban(http, user_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn audit_logs(
        &self,
        http: impl AsRef<Http>,
        action_type: Option<audit_log::Action>,
        user_id: Option<UserId>,
        before: Option<AuditLogEntryId>,
        limit: Option<u8>,
    ) -> Result<AuditLogs> {
        self.id.audit_logs(http, action_type, user_id, before, limit).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn channels(&self, http: impl AsRef<Http>) -> Result<HashMap<ChannelId, GuildChannel>> {
        self.id.channels(http).await
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn channel_id_from_name(
        &self,
        cache: impl AsRef<Cache>,
        name: impl AsRef<str>,
    ) -> Option<ChannelId> {
        let cache = cache.as_ref();
        let guild = cache.guild(self.id)?;
        #[allow(deprecated)]
        guild.channel_id_from_name(cache, name)
    }
    #[cfg(feature = "model")]
    async fn create_channel(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateChannel<'_>,
    ) -> Result<GuildChannel> {
        self.id.create_channel(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn create_emoji(&self, http: impl AsRef<Http>, name: &str, image: &str) -> Result<Emoji> {
        self.id.create_emoji(http, name, image).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn create_integration(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
        kind: &str,
    ) -> Result<()> {
        self.id.create_integration(http, integration_id, kind).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn create_command(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateCommand,
    ) -> Result<Command> {
        self.id.create_command(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    async fn set_commands(
        &self,
        http: impl AsRef<Http>,
        commands: Vec<CreateCommand>,
    ) -> Result<Vec<Command>> {
        self.id.set_commands(http, commands).await
    }
    #[cfg(feature = "model")]
    async fn edit_command_permissions(
        &self,
        cache_http: impl CacheHttp,
        command_id: CommandId,
        builder: EditCommandPermissions,
    ) -> Result<CommandPermissions> {
        self.id.edit_command_permissions(cache_http, command_id, builder).await
    }
    #[cfg(feature = "model")]
    async fn get_commands(&self, http: impl AsRef<Http>) -> Result<Vec<Command>> {
        self.id.get_commands(http).await
    }
    #[cfg(feature = "model")]
    async fn get_commands_with_localizations(
        &self,
        http: impl AsRef<Http>,
    ) -> Result<Vec<Command>> {
        self.id.get_commands_with_localizations(http).await
    }
    #[cfg(feature = "model")]
    async fn get_command(&self, http: impl AsRef<Http>, command_id: CommandId) -> Result<Command> {
        self.id.get_command(http, command_id).await
    }
    #[cfg(feature = "model")]
    async fn edit_command(
        &self,
        cache_http: impl CacheHttp,
        command_id: CommandId,
        builder: CreateCommand,
    ) -> Result<Command> {
        self.id.edit_command(cache_http, command_id, builder).await
    }
    #[cfg(feature = "model")]
    async fn delete_command(&self, http: impl AsRef<Http>, command_id: CommandId) -> Result<()> {
        self.id.delete_command(http, command_id).await
    }
    #[cfg(feature = "model")]
    async fn get_commands_permissions(
        &self,
        http: impl AsRef<Http>,
    ) -> Result<Vec<CommandPermissions>> {
        self.id.get_commands_permissions(http).await
    }
    #[cfg(feature = "model")]
    async fn get_command_permissions(
        &self,
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<CommandPermissions> {
        self.id.get_command_permissions(http, command_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn create_role(&self, cache_http: impl CacheHttp, builder: EditRole<'_>) -> Result<Role> {
        self.id.create_role(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    async fn create_sticker(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateSticker<'_>,
    ) -> Result<Sticker> {
        self.id.create_sticker(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.delete(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete_emoji(
        &self,
        http: impl AsRef<Http>,
        emoji_id: impl Into<EmojiId>,
    ) -> Result<()> {
        self.id.delete_emoji(http, emoji_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete_integration(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
    ) -> Result<()> {
        self.id.delete_integration(http, integration_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete_role(&self, http: impl AsRef<Http>, role_id: impl Into<RoleId>) -> Result<()> {
        self.id.delete_role(http, role_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn delete_sticker(
        &self,
        http: impl AsRef<Http>,
        sticker_id: impl Into<StickerId>,
    ) -> Result<()> {
        self.id.delete_sticker(http, sticker_id).await
    }
    #[cfg(feature = "model")]
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditGuild<'_>) -> Result<()> {
        let guild = self.id.edit(cache_http, builder).await?;
        self.afk_metadata = guild.afk_metadata;
        self.default_message_notifications = guild.default_message_notifications;
        self.emojis = guild.emojis;
        self.features = guild.features;
        self.icon = guild.icon;
        self.mfa_level = guild.mfa_level;
        self.name = guild.name;
        self.owner_id = guild.owner_id;
        self.roles = guild.roles;
        self.splash = guild.splash;
        self.verification_level = guild.verification_level;
        Ok(())
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn edit_emoji(
        &self,
        http: impl AsRef<Http>,
        emoji_id: impl Into<EmojiId>,
        name: &str,
    ) -> Result<Emoji> {
        self.id.edit_emoji(http, emoji_id, name).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn edit_member(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
        builder: EditMember<'_>,
    ) -> Result<Member> {
        self.id.edit_member(cache_http, user_id, builder).await
    }
    #[cfg(feature = "model")]
    async fn edit_mfa_level(
        &self,
        http: impl AsRef<Http>,
        mfa_level: MfaLevel,
        audit_log_reason: Option<&str>,
    ) -> Result<MfaLevel> {
        self.id.edit_mfa_level(http, mfa_level, audit_log_reason).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn edit_nickname(
        &self,
        http: impl AsRef<Http>,
        new_nickname: Option<&str>,
    ) -> Result<()> {
        self.id.edit_nickname(http, new_nickname).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn edit_role(
        &self,
        cache_http: impl CacheHttp,
        role_id: impl Into<RoleId>,
        builder: EditRole<'_>,
    ) -> Result<Role> {
        self.id.edit_role(cache_http, role_id, builder).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn edit_role_position(
        &self,
        http: impl AsRef<Http>,
        role_id: impl Into<RoleId>,
        position: u16,
    ) -> Result<Vec<Role>> {
        self.id.edit_role_position(http, role_id, position).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn edit_sticker(
        &self,
        cache_http: impl CacheHttp,
        sticker_id: impl Into<StickerId>,
        builder: EditSticker<'_>,
    ) -> Result<Sticker> {
        self.id.edit_sticker(cache_http, sticker_id, builder).await
    }
    #[cfg(feature = "model")]
    async fn edit_welcome_screen(
        &self,
        cache_http: impl CacheHttp,
        builder: EditGuildWelcomeScreen<'_>,
    ) -> Result<GuildWelcomeScreen> {
        self.id.edit_welcome_screen(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    async fn edit_widget(
        &self,
        cache_http: impl CacheHttp,
        builder: EditGuildWidget<'_>,
    ) -> Result<GuildWidget> {
        self.id.edit_widget(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn get(cache_http: impl CacheHttp, guild_id: impl Into<GuildId>) -> Result<PartialGuild> {
        guild_id.into().to_partial_guild(cache_http).await
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    #[inline]
    fn greater_member_hierarchy(
        &self,
        cache: impl AsRef<Cache>,
        lhs_id: impl Into<UserId>,
        rhs_id: impl Into<UserId>,
    ) -> Option<UserId> {
        let cache = cache.as_ref();
        let guild = cache.guild(self.id)?;
        guild.greater_member_hierarchy(cache, lhs_id, rhs_id)
    }
    #[cfg(feature = "model")]
    #[inline]
    fn member_permissions(&self, member: &Member) -> Permissions {
        Guild::user_permissions_in_(
            None,
            member.user.id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }
    #[cfg(feature = "model")]
    #[inline]
    fn partial_member_permissions(&self, member_id: UserId, member: &PartialMember) -> Permissions {
        if let Some(user) = &member.user {
            assert_eq!(user.id, member_id, "User::id does not match provided PartialMember");
        }
        Guild::user_permissions_in_(
            None,
            member_id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }
    #[cfg(feature = "model")]
    fn partial_member_permissions_in(
        &self,
        channel: &GuildChannel,
        member_id: UserId,
        member: &PartialMember,
    ) -> Permissions {
        if let Some(user) = &member.user {
            assert_eq!(user.id, member_id, "User::id does not match provided PartialMember");
        }
        Guild::user_permissions_in_(
            Some(channel),
            member_id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn reorder_channels(
        &self,
        http: impl AsRef<Http>,
        channels: impl IntoIterator<Item = (ChannelId, u64)>,
    ) -> Result<()> {
        self.id.reorder_channels(http, channels).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn search_members(
        &self,
        http: impl AsRef<Http>,
        query: &str,
        limit: Option<u64>,
    ) -> Result<Vec<Member>> {
        self.id.search_members(http, query, limit).await
    }
    #[cfg(feature = "model")]
    async fn start_prune(&self, cache_http: impl CacheHttp, days: u8) -> Result<GuildPrune> {
        self.id.start_prune(cache_http.http(), days).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn kick(&self, http: impl AsRef<Http>, user_id: impl Into<UserId>) -> Result<()> {
        self.id.kick(http, user_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn kick_with_reason(
        &self,
        http: impl AsRef<Http>,
        user_id: impl Into<UserId>,
        reason: &str,
    ) -> Result<()> {
        self.id.kick_with_reason(http, user_id, reason).await
    }
    #[cfg(feature = "model")]
    fn icon_url(&self) -> Option<String> {
        icon_url(self.id, self.icon.as_ref())
    }
    #[cfg(feature = "model")]
    fn banner_url(&self) -> Option<String> {
        self.banner.as_ref().map(|banner| cdn!("/banners/{}/{}.webp", self.id, banner))
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn emojis(&self, http: impl AsRef<Http>) -> Result<Vec<Emoji>> {
        self.id.emojis(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn emoji(&self, http: impl AsRef<Http>, emoji_id: EmojiId) -> Result<Emoji> {
        self.id.emoji(http, emoji_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn integrations(&self, http: impl AsRef<Http>) -> Result<Vec<Integration>> {
        self.id.integrations(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn invites(&self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {
        self.id.invites(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn current_user_member(&self, http: impl AsRef<Http>) -> Result<Member> {
        self.id.current_user_member(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn leave(&self, http: impl AsRef<Http>) -> Result<()> {
        self.id.leave(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn member(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
    ) -> Result<Member> {
        self.id.member(cache_http, user_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn members(
        &self,
        http: impl AsRef<Http>,
        limit: Option<u64>,
        after: impl Into<Option<UserId>>,
    ) -> Result<Vec<Member>> {
        self.id.members(http, limit, after).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn move_member(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<UserId>,
        channel_id: impl Into<ChannelId>,
    ) -> Result<Member> {
        self.id.move_member(cache_http, user_id, channel_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    fn user_permissions_in(&self, channel: &GuildChannel, member: &Member) -> Permissions {
        Guild::user_permissions_in_(
            Some(channel),
            member.user.id,
            &member.roles,
            self.id,
            &self.roles,
            self.owner_id,
        )
    }
    #[cfg(feature = "model")]
    #[inline]
    fn role_permissions_in(&self, channel: &GuildChannel, role: &Role) -> Result<Permissions> {
        Guild::role_permissions_in_(channel, role, self.id).map_err(From::from)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn prune_count(&self, http: impl AsRef<Http>, days: u8) -> Result<GuildPrune> {
        self.id.prune_count(http, days).await
    }
    #[cfg(feature = "model")]
    #[cfg(all(feature = "cache", feature = "utils"))]
    #[inline]
    fn shard_id(&self, cache: impl AsRef<Cache>) -> u32 {
        self.id.shard_id(cache)
    }
    #[cfg(feature = "model")]
    #[cfg(all(feature = "utils", not(feature = "cache")))]
    #[inline]
    fn shard_id(&self, shard_count: u32) -> u32 {
        self.id.shard_id(shard_count)
    }
    #[cfg(feature = "model")]
    #[inline]
    fn splash_url(&self) -> Option<String> {
        self.splash.as_ref().map(|splash| cdn!("/splashes/{}/{}.webp?size=4096", self.id, splash))
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn start_integration_sync(
        &self,
        http: impl AsRef<Http>,
        integration_id: impl Into<IntegrationId>,
    ) -> Result<()> {
        self.id.start_integration_sync(http, integration_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn unban(&self, http: impl AsRef<Http>, user_id: impl Into<UserId>) -> Result<()> {
        self.id.unban(http, user_id).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn vanity_url(&self, http: impl AsRef<Http>) -> Result<String> {
        self.id.vanity_url(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn webhooks(&self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> {
        self.id.webhooks(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    fn role_by_name(&self, role_name: &str) -> Option<&Role> {
        self.roles.values().find(|role| role_name == role.name)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reply(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector {
        MessageCollector::new(shard_messenger).guild_id(self.id)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_replies(&self, shard_messenger: impl AsRef<ShardMessenger>) -> MessageCollector {
        self.await_reply(shard_messenger)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reaction(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector {
        ReactionCollector::new(shard_messenger).guild_id(self.id)
    }
    #[cfg(feature = "model")]
    #[cfg(feature = "collector")]
    fn await_reactions(&self, shard_messenger: impl AsRef<ShardMessenger>) -> ReactionCollector {
        self.await_reaction(shard_messenger)
    }
    #[cfg(feature = "model")]
    async fn get_active_threads(&self, http: impl AsRef<Http>) -> Result<ThreadsData> {
        self.id.get_active_threads(http).await
    }
    #[cfg(feature = "model")]
    async fn get_soundboard(self, http: impl AsRef<Http>, sound_id: SoundId) -> Result<Soundboard> {
        self.id.get_soundboard(http, sound_id).await
    }
    #[cfg(feature = "model")]
    async fn get_soundboards(self, http: impl AsRef<Http>) -> Result<Vec<Soundboard>> {
        self.id.get_soundboards(http).await
    }
    #[cfg(feature = "model")]
    async fn create_soundboard(
        self,
        cache_http: impl CacheHttp,
        builder: CreateSoundboard<'_>,
    ) -> Result<Soundboard> {
        self.id.create_soundboard(cache_http, builder).await
    }
    #[cfg(feature = "model")]
    async fn edit_soundboard(
        self,
        cache_http: impl CacheHttp,
        sound_id: SoundId,
        builder: EditSoundboard<'_>,
    ) -> Result<Soundboard> {
        self.id.edit_soundboard(cache_http, sound_id, builder).await
    }
    #[cfg(feature = "model")]
    async fn delete_soundboard(
        self,
        http: impl AsRef<Http>,
        sound_id: SoundId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.id.delete_soundboard(http, sound_id, audit_log_reason).await
    }
}
