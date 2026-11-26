//! Top-level [`Framework`] builder and options.

use std::any::Any;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use oxcord_builders::{CreateAllowedMentions, CreateCommand, CreateCommandOption};
use oxcord_model::Permissions;
use oxcord_model::application::{
    CommandDataOptionValue,
    CommandInteraction,
    CommandType,
    ComponentInteraction,
    MembershipState,
    ModalInteraction,
    ResolvedTarget,
    TeamMemberRole,
};
use oxcord_model::channel::{ChannelType, Message, MessageType};
use oxcord_model::id::{GuildId, MessageId, UserId};
use oxcord_model::user::User;
use oxcord_rest::Http;

/// Whether a channel type counts as a thread, for the
/// `SEND_MESSAGES`/`SEND_MESSAGES_IN_THREADS` permission swap (mirrors
/// poise's identical check).
fn is_thread_channel(kind: ChannelType) -> bool {
    matches!(kind, ChannelType::NewsThread | ChannelType::PrivateThread | ChannelType::PublicThread)
}

/// Resolved permissions Discord sends directly on a slash-command
/// interaction, with the thread `SEND_MESSAGES` swap applied.
fn adjust_for_thread(perms: Option<Permissions>, is_thread: bool) -> Option<Permissions> {
    perms.map(|mut p| {
        if is_thread {
            p.set(Permissions::SEND_MESSAGES, p.send_messages_in_threads());
        }
        p
    })
}

/// Find the option Discord marked as focused for autocomplete, descending
/// into `SubCommand`/`SubCommandGroup` option nesting (the focused option is
/// always on the innermost, actually-invoked subcommand, not the top level).
fn find_focused_option(
    options: &[oxcord_model::application::CommandDataOption],
) -> Option<&oxcord_model::application::CommandDataOption> {
    for opt in options {
        match &opt.value {
            CommandDataOptionValue::Autocomplete {
                ..
            } => return Some(opt),
            CommandDataOptionValue::SubCommand(inner)
            | CommandDataOptionValue::SubCommandGroup(inner) => {
                if let Some(found) = find_focused_option(inner) {
                    return Some(found);
                }
            },
            _ => {},
        }
    }
    None
}

/// A leaf command's own parameters as slash-command options (no subcommands
/// of its own).
fn command_options_as_slash(cmd: &Command) -> Vec<CreateCommandOption> {
    cmd.options
        .iter()
        .map(|opt| {
            let mut builder = CreateCommandOption::new(opt.kind, &opt.name, &opt.description)
                .required(opt.required)
                .set_autocomplete(cmd.autocomplete.contains_key(&opt.name));
            for (name, value) in &opt.choices {
                builder = builder.add_string_choice(name.clone(), value.clone());
            }
            builder
        })
        .collect()
}

/// Serialize one subcommand (or subcommand group, if it has subcommands of
/// its own) as a `CreateCommandOption`, recursively. Mirrors poise's
/// `Command::create_as_subcommand`. Discord allows at most two levels of
/// nesting (group -> subcommand); deeper trees than that will be rejected by
/// Discord at registration time, same as poise.
fn command_as_subcommand_option(cmd: &Command) -> Option<CreateCommandOption> {
    if cmd.handler.is_none() && cmd.subcommands.is_empty() {
        // Nothing to invoke and nothing nested under it — skip rather than
        // register a dead-end option.
        return None;
    }

    let kind = if cmd.subcommands.is_empty() {
        oxcord_model::application::CommandOptionType::SubCommand
    } else {
        oxcord_model::application::CommandOptionType::SubCommandGroup
    };
    let mut builder = CreateCommandOption::new(kind, &cmd.name, &cmd.description);
    for (locale, name) in &cmd.name_localizations {
        builder = builder.name_localized(locale, name);
    }
    for (locale, desc) in &cmd.description_localizations {
        builder = builder.description_localized(locale, desc);
    }

    if cmd.subcommands.is_empty() {
        for option in command_options_as_slash(cmd) {
            builder = builder.add_sub_option(option);
        }
    } else {
        for sub in &cmd.subcommands {
            if let Some(sub_option) = command_as_subcommand_option(sub) {
                builder = builder.add_sub_option(sub_option);
            }
        }
    }

    Some(builder)
}

use crate::command::{CheckFn, Command, CommandKind, ContextMenuAction};
use crate::context::{Context, ErrorContext, FrameworkHttp, InvocationData, ReplyCallback};
use crate::dispatch::{Dispatcher, Invocation};
use crate::error::FrameworkError;
use crate::prefix::PrefixOptions;

/// Bentuk callback async buat [`FrameworkOptions::on_error`].
pub type ErrorHook = Arc<
    dyn Fn(FrameworkError, ErrorContext) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
>;

/// Bentuk callback async buat [`FrameworkOptions::pre_command`] dan [`FrameworkOptions::post_command`].
pub type CommandHook =
    Arc<dyn for<'a> Fn(Context<'a>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send + Sync>;

/// Global framework configuration.
#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct FrameworkOptions {
    /// All registered commands.
    pub commands: Vec<Command>,
    /// Prefix used for text commands (e.g. `"!"`, `"?"`, `"."`).
    /// `None` disables prefix command support entirely.
    pub prefix: Option<String>,
    /// Opsi lanjutan untuk prefix command (`additional_prefixes`, `dynamic_prefix`, `edit_tracker`, dll).
    pub prefix_options: PrefixOptions,
    /// User IDs considered bot owners (bypass `owner_only` checks).
    pub owners: HashSet<UserId>,
    /// Whether to automatically register slash commands on ready.
    pub auto_register_slash: bool,
    /// Hook global buat nangani [`FrameworkError`] (cooldown, permission, guild-only, error
    /// handler, dst) — setara `on_error` di poise. Set lewat [`FrameworkBuilder::on_error`].
    ///
    /// Kalau diset, error dikonsumsi hook dan `dispatch_*` balikin `Ok(())`, jadi bridge
    /// nggak ikut nge-log/handle lagi (nggak dobel). Kalau nggak diset, error dikembalikan
    /// apa adanya. `UnknownCommand` sengaja nggak lewat hook (itu bukan kegagalan command).
    pub on_error: Option<ErrorHook>,
    /// Shared, typed application state every command handler can read back
    /// via `Context::data::<T>()` — the equivalent of poise's generic `Data`
    /// type parameter. Set it with [`FrameworkBuilder::user_data`].
    pub user_data: Option<Arc<dyn Any + Send + Sync>>,
    /// Hook global yang jalan tepat sebelum handler command dipanggil.
    pub pre_command: Option<CommandHook>,
    /// Hook global yang jalan tepat sesudah handler command sukses (`Ok(())`).
    pub post_command: Option<CommandHook>,
    /// Check global yang jalan sebelum check per-command untuk setiap command.
    pub command_check: Option<CheckFn>,
    /// Kalau `true`, user yang terdaftar di `owners` melewati semua check kecuali `resolve_subcommand`.
    pub skip_checks_for_owners: bool,
    /// Default `allowed_mentions` untuk semua reply jika reply tidak mengaturnya sendiri.
    pub allowed_mentions: Option<CreateAllowedMentions>,
    /// Callback yang dipanggil di `Context::send()` untuk memodifikasi setiap outgoing reply.
    pub reply_callback: Option<ReplyCallback>,
    /// Kalau `true`, framework tidak otomatis mengecek/mencatat cooldown (dipakai jika logic cooldown manual).
    pub manual_cooldowns: bool,
    /// Kalau `true` dan cache aktif, `guild_only` juga memastikan guild ada di cache.
    pub require_cache_for_guild_check: bool,
    /// Kalau `true`, otomatis isi `owners` dari `get_current_application_info()` saat startup (`Ready`).
    pub initialize_owners: bool,
    /// Daftar role team Discord yang ikut dimasukkan ke `owners` saat `initialize_owners` aktif.
    /// Jika `None`, semua member team yang accepted (selain `ReadOnly`) ikut dimasukkan.
    pub initialized_team_roles: Option<Vec<TeamMemberRole>>,
}

impl std::fmt::Debug for FrameworkOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameworkOptions")
            .field("commands", &self.commands.iter().map(|c| &c.name).collect::<Vec<_>>())
            .field("prefix", &self.prefix)
            .field("prefix_options", &self.prefix_options)
            .field("owners", &self.owners)
            .field("auto_register_slash", &self.auto_register_slash)
            .field("user_data", &self.user_data.is_some())
            .field("on_error", &self.on_error.is_some())
            .field("pre_command", &self.pre_command.is_some())
            .field("post_command", &self.post_command.is_some())
            .field("command_check", &self.command_check.is_some())
            .field("skip_checks_for_owners", &self.skip_checks_for_owners)
            .field("manual_cooldowns", &self.manual_cooldowns)
            .field("require_cache_for_guild_check", &self.require_cache_for_guild_check)
            .field("initialize_owners", &self.initialize_owners)
            .finish_non_exhaustive()
    }
}

/// The main framework handle.
///
/// Build it with [`Framework::builder()`] and pass it to your Oxcord client.
///
/// ```rust,no_run,ignore
/// use oxcord_framework::{Framework, FrameworkOptions};
///
/// let framework = Framework::builder()
///     .options(FrameworkOptions {
///         prefix: Some("!".into()),
///         auto_register_slash: true,
///         ..Default::default()
///     })
///     .build();
/// ```
pub struct Framework {
    pub(crate) options: FrameworkOptions,
    pub(crate) dispatcher: Dispatcher,
}

impl std::fmt::Debug for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Framework").field("options", &self.options).finish_non_exhaustive()
    }
}

impl Framework {
    /// Create a new [`FrameworkBuilder`].
    #[must_use]
    pub fn builder() -> FrameworkBuilder {
        FrameworkBuilder::default()
    }

    /// Return a reference to the registered commands.
    #[must_use]
    pub fn commands(&self) -> &[Command] {
        &self.options.commands
    }

    /// Return the configured prefix, if any.
    #[must_use]
    pub fn prefix(&self) -> Option<&str> {
        self.options.prefix.as_deref()
    }

    /// Akses [`PrefixOptions`] yang dikonfigurasi.
    #[must_use]
    pub fn prefix_options(&self) -> &PrefixOptions {
        &self.options.prefix_options
    }

    /// Apakah [`FrameworkOptions::initialize_owners`] aktif.
    #[must_use]
    pub fn should_initialize_owners(&self) -> bool {
        self.options.initialize_owners
    }

    /// Isi `owners` otomatis dari `get_current_application_info()` Discord (owner tunggal maupun anggota team).
    ///
    /// # Errors
    ///
    /// Mengembalikan [`FrameworkError::Http`] jika request ke Discord gagal.
    pub async fn initialize_owners(&self, http: &Http) -> Result<(), FrameworkError> {
        let app_info = http
            .get_current_application_info()
            .await
            .map_err(|e| FrameworkError::Http(e.to_string()))?;
        let mut owners = self.dispatcher.owners().write();
        if let Some(owner) = app_info.owner {
            owners.insert(owner.id);
        }
        if let Some(team) = app_info.team {
            owners.insert(team.owner_user_id);
            for member in team.members {
                if member.membership_state != MembershipState::Accepted {
                    continue;
                }
                let allowed = match &self.options.initialized_team_roles {
                    Some(roles) => {
                        roles.iter().any(|r| format!("{r:?}") == format!("{:?}", member.role))
                    },
                    None => !matches!(member.role, TeamMemberRole::ReadOnly),
                };
                if allowed {
                    owners.insert(member.user.id);
                }
            }
        }
        Ok(())
    }

    /// Teruskan `ComponentInteraction` ke `InteractionBus` (dipanggil dari `v2_bridge`).
    pub fn notify_component(&self, interaction: &ComponentInteraction) {
        let _ = self.dispatcher.interaction_bus().component_tx.send(interaction.clone());
    }

    /// Teruskan `ModalInteraction` ke `InteractionBus` (dipanggil dari `v2_bridge`).
    pub fn notify_modal(&self, interaction: &ModalInteraction) {
        let _ = self.dispatcher.interaction_bus().modal_tx.send(interaction.clone());
    }

    /// Cocokkan prefix pesan dengan urutan: `dynamic_prefix` -> `prefix` -> `additional_prefixes` -> `mention_as_prefix`.
    async fn strip_any_prefix<'a>(
        &'a self,
        content: &'a str,
        guild_id: Option<GuildId>,
        author_id: UserId,
        bot_user_id: Option<UserId>,
    ) -> Result<Option<(&'a str, &'a str)>, FrameworkError> {
        if let Some(dyn_fn) = &self.options.prefix_options.dynamic_prefix {
            match dyn_fn(guild_id, author_id, self.options.user_data.clone()).await {
                Ok(Some(dyn_prefix)) => {
                    if let Some(rest) = content.strip_prefix(dyn_prefix.as_str()) {
                        let matched_len = dyn_prefix.len();
                        return Ok(Some((&content[..matched_len], rest)));
                    }
                },
                Ok(None) => {},
                Err(e) => {
                    return Err(FrameworkError::DynamicPrefix {
                        error: Box::new(e),
                        msg_content: content.to_owned(),
                    });
                },
            }
        }

        if let Some(prefix) = self.options.prefix.as_deref() {
            if let Some(rest) = content.strip_prefix(prefix) {
                return Ok(Some((prefix, rest)));
            }
        }

        for extra in &self.options.prefix_options.additional_prefixes {
            if let Some(rest) = content.strip_prefix(extra.as_str()) {
                return Ok(Some((extra.as_str(), rest)));
            }
        }

        if self.options.prefix_options.mention_as_prefix {
            if let Some(bot_id) = bot_user_id {
                let m1 = format!("<@{bot_id}>");
                let m2 = format!("<@!{bot_id}>");
                if let Some(rest) = content.strip_prefix(m1.as_str()) {
                    return Ok(Some((&content[..m1.len()], rest.trim_start())));
                }
                if let Some(rest) = content.strip_prefix(m2.as_str()) {
                    return Ok(Some((&content[..m2.len()], rest.trim_start())));
                }
            }
        }

        // Terakhir: pengguna noprefix boleh memanggil command tanpa awalan.
        if let Some(noprefix) = &self.options.prefix_options.noprefix {
            if noprefix(guild_id, author_id, bot_user_id, self.options.user_data.clone()).await {
                return Ok(Some((&content[..0], content)));
            }
        }

        Ok(None)
    }

    /// Try to dispatch a gateway message as a prefix command.
    ///
    /// Returns `None` if no prefix is configured, the message does not start
    /// with it, or the text after the prefix names no registered command
    /// (nothing is dispatched in any of those cases — this is the normal case
    /// for the vast majority of messages a bot sees).
    ///
    /// # Errors
    ///
    /// The inner `Result` is `Err` if a command was found and dispatch was
    /// attempted but failed (cooldown, permission check, or the handler
    /// itself returned an error).
    ///
    /// `member_permissions`/`app_permissions` are the invoking member's and
    /// the bot's own resolved permissions in `channel_id`, if the caller was
    /// able to compute them (typically from the gateway cache — Discord
    /// doesn't send resolved permissions on plain messages the way it does
    /// for interactions). Pass `None` when unavailable (no cache, DM,
    /// uncached guild/member); a command with
    /// [`crate::Command::required_permissions`]/`required_bot_permissions`
    /// then fails closed with [`FrameworkError::PermissionCheckUnavailable`]
    /// rather than running unchecked.
    ///
    /// `gateway` adalah context gateway mentah dari pemanggil (di `oxcord`:
    /// `Arc::new(ctx.clone())` dari `oxcord::client::Context`), yang nanti bisa diambil
    /// command lewat [`Context::gateway_context`]. `None` kalau nggak ada.
    #[allow(clippy::too_many_arguments)]
    pub async fn dispatch_message(
        &self,
        author: &User,
        channel_id: oxcord_model::id::ChannelId,
        guild_id: Option<oxcord_model::id::GuildId>,
        content: &str,
        http: Arc<Http>,
        member_permissions: Option<Permissions>,
        app_permissions: Option<Permissions>,
        gateway: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Option<Result<(), FrameworkError>> {
        let (matched_prefix, rest) =
            match self.strip_any_prefix(content, guild_id, author.id, None).await {
                Ok(val) => val?,
                Err(e) => return Some(Err(e)),
            };
        let mut parts = rest.splitn(2, char::is_whitespace);
        let command_name = parts.next().unwrap_or("").trim();
        if command_name.is_empty() {
            return None;
        }
        let args = parts.next().unwrap_or("").trim_start().to_owned();

        let data = InvocationData::Prefix {
            args: args.clone(),
            guild_id,
            member_permissions,
            app_permissions,
        };
        let framework_http =
            Arc::new(FrameworkHttp::for_prefix(http, channel_id).with_reply_defaults(
                self.options.reply_callback.clone(),
                self.options.allowed_mentions.clone(),
            ));
        let inv = Invocation {
            command_name,
            author,
            data: &data,
            http: framework_http,
            received_at: Instant::now(),
            gateway,
            is_nsfw: None,
            guild_in_cache: None,
            locale: None,
            author_member: None,
            invocation_string: if args.is_empty() {
                format!("{matched_prefix}{command_name}")
            } else {
                format!("{matched_prefix}{command_name} {args}")
            },
        };
        Some(self.dispatcher.dispatch(inv).await)
    }

    /// Versi lengkap dispatch pesan gateway yang mendukung semua [`PrefixOptions`]
    /// (`ignore_bots`, `execute_self_messages`, `ignore_thread_creation`, `mention_as_prefix`,
    /// `non_command_message`, `edit_tracker`, `nsfw_only`, `require_cache_for_guild_check`).
    #[allow(clippy::too_many_arguments)]
    pub async fn dispatch_full_message(
        &self,
        msg: &Message,
        http: Arc<Http>,
        bot_user_id: Option<UserId>,
        member_permissions: Option<Permissions>,
        app_permissions: Option<Permissions>,
        is_nsfw: Option<bool>,
        guild_in_cache: Option<bool>,
        reuse_message_id: Option<MessageId>,
        gateway: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Option<Result<(), FrameworkError>> {
        let is_self = bot_user_id.is_some_and(|id| id == msg.author.id);
        if is_self && !self.options.prefix_options.execute_self_messages {
            return None;
        }
        if msg.author.bot && !is_self && self.options.prefix_options.ignore_bots {
            return None;
        }
        if self.options.prefix_options.ignore_thread_creation
            && msg.kind == MessageType::ThreadCreated
        {
            return None;
        }

        let stripped = match self
            .strip_any_prefix(&msg.content, msg.guild_id, msg.author.id, bot_user_id)
            .await
        {
            Ok(val) => val,
            Err(e) => return Some(Err(e)),
        };

        let parsed = stripped.and_then(|(matched_prefix, rest)| {
            let mut parts = rest.splitn(2, char::is_whitespace);
            let cmd_name = parts.next().unwrap_or("").trim();
            if cmd_name.is_empty() {
                return None;
            }
            // Pastikan command-nya memang terdaftar; kalau bukan command, jatuhkan ke `non_command_message`.
            let found = self.dispatcher.find_command(cmd_name)?;
            let args = parts.next().unwrap_or("").trim_start().to_owned();
            Some((matched_prefix, cmd_name, found, args))
        });

        let Some((matched_prefix, command_name, cmd, args)) = parsed else {
            if let Some(non_cmd) = &self.options.prefix_options.non_command_message {
                if let Err(e) = non_cmd(msg.clone(), self.options.user_data.clone()).await {
                    return Some(Err(FrameworkError::NonCommandMessage {
                        error: Box::new(e),
                    }));
                }
            }
            return None;
        };

        let data = InvocationData::Prefix {
            args: args.clone(),
            guild_id: msg.guild_id,
            member_permissions,
            app_permissions,
        };
        let mut framework_http = FrameworkHttp::for_prefix(http, msg.channel_id)
            .with_message_id(msg.id)
            .with_reply_defaults(
                self.options.reply_callback.clone(),
                self.options.allowed_mentions.clone(),
            )
            .with_edit_tracker(self.options.prefix_options.edit_tracker.clone());
        if let Some(reuse_id) = reuse_message_id {
            framework_http = framework_http.with_reuse_message_id(reuse_id);
        }

        let inv = Invocation {
            command_name,
            author: &msg.author,
            data: &data,
            http: Arc::new(framework_http),
            received_at: Instant::now(),
            gateway,
            is_nsfw,
            guild_in_cache,
            locale: None,
            author_member: None,
            invocation_string: if args.is_empty() {
                format!("{matched_prefix}{command_name}")
            } else {
                format!("{matched_prefix}{command_name} {args}")
            },
        };

        if let Some(tracker) = &self.options.prefix_options.edit_tracker {
            if cmd.invoke_on_edit || cmd.track_deletion {
                tracker.track_command(msg.id, msg.channel_id, &cmd.name, None);
            }
        }

        Some(self.dispatcher.dispatch(inv).await)
    }

    /// Dispatch a gateway slash-command (atau context-menu) interaction.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::UnknownCommand`] if no command by that name
    /// is registered, or any other [`FrameworkError`] the dispatch pipeline
    /// produces (cooldown, permission check, handler error).
    pub async fn dispatch_slash(
        &self,
        command: &CommandInteraction,
        http: Arc<Http>,
        gateway: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), FrameworkError> {
        self.dispatch_slash_with_context(command, http, None, None, gateway).await
    }

    /// Sama seperti [`Self::dispatch_slash`], dengan info tambahan `is_nsfw` dan `guild_in_cache`
    /// yang dihitung bridge dari gateway cache.
    ///
    /// # Errors
    ///
    /// Mengembalikan [`FrameworkError`] jika command gagal dijalankan, permission/check gagal, atau terjadi error HTTP.
    pub async fn dispatch_slash_with_context(
        &self,
        command: &CommandInteraction,
        http: Arc<Http>,
        is_nsfw: Option<bool>,
        guild_in_cache: Option<bool>,
        gateway: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), FrameworkError> {
        let is_thread = command.channel.as_ref().is_some_and(|c| is_thread_channel(c.kind));
        let data = InvocationData::Slash {
            token: command.token.clone(),
            guild_id: command.guild_id,
            options: command.data.options.clone(),
            member_permissions: adjust_for_thread(
                command.member.as_ref().and_then(|m| m.permissions),
                is_thread,
            ),
            app_permissions: adjust_for_thread(command.app_permissions, is_thread),
            is_thread,
        };
        let framework_http = Arc::new(
            FrameworkHttp::for_slash(http, command.id, command.application_id, command.channel_id)
                .with_reply_defaults(
                    self.options.reply_callback.clone(),
                    self.options.allowed_mentions.clone(),
                ),
        );

        // Tangani context-menu command (`CommandType::User` / `CommandType::Message`).
        if matches!(command.data.kind, CommandType::User | CommandType::Message) {
            return self
                .dispatch_context_menu(
                    command,
                    &data,
                    framework_http,
                    is_nsfw,
                    guild_in_cache,
                    gateway,
                )
                .await;
        }

        let inv = Invocation {
            command_name: &command.data.name,
            author: &command.user,
            data: &data,
            http: framework_http,
            received_at: Instant::now(),
            gateway,
            is_nsfw,
            guild_in_cache,
            locale: Some(command.locale.clone()),
            author_member: command.member.as_deref().cloned(),
            invocation_string: format!("/{}", command.data.name),
        };
        self.dispatcher.dispatch(inv).await
    }

    /// Dispatch interaction context-menu (`CommandType::User` / `CommandType::Message`).
    async fn dispatch_context_menu(
        &self,
        command: &CommandInteraction,
        data: &InvocationData,
        framework_http: Arc<FrameworkHttp>,
        _is_nsfw: Option<bool>,
        _guild_in_cache: Option<bool>,
        gateway: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), FrameworkError> {
        let Some(cmd) = self.options.commands.iter().find(|c| {
            c.context_menu_action.is_some()
                && c.context_menu_name.as_deref().unwrap_or(&c.name) == command.data.name
        }) else {
            return Err(FrameworkError::UnknownInteraction {
                interaction_name: command.data.name.clone(),
            });
        };

        let ctx = Context {
            author: &command.user,
            channel_id: command.channel_id,
            guild_id: command.guild_id,
            invocation: data,
            http: framework_http,
            data: self.options.user_data.clone(),
            received_at: Instant::now(),
            gateway,
            command: Some(cmd),
            invocation_data: Arc::new(tokio::sync::Mutex::new(Box::new(()))),
            locale: Some(command.locale.clone()),
            author_member: command.member.as_deref().cloned(),
            invocation_string: format!("/{}", command.data.name),
            interaction_bus: Some(Arc::clone(self.dispatcher.interaction_bus())),
        };

        let result = match (cmd.context_menu_action, command.data.target()) {
            (Some(ContextMenuAction::User(f)), Some(ResolvedTarget::User(user, _))) => {
                f(ctx.clone(), user.clone()).await
            },
            (Some(ContextMenuAction::Message(f)), Some(ResolvedTarget::Message(msg))) => {
                f(ctx.clone(), msg.clone()).await
            },
            _ => Err(FrameworkError::CommandStructureMismatch {
                description: "context menu command target did not match command type",
            }),
        };

        if let Err(err) = result {
            let hook = cmd.on_error.as_ref().or(self.options.on_error.as_ref());
            if let Some(hook) = hook {
                hook(err, ErrorContext::from_ctx(&cmd.name, &ctx)).await;
                return Ok(());
            }
            return Err(err);
        }
        Ok(())
    }

    /// Whether [`FrameworkOptions::auto_register_slash`] is enabled.
    #[must_use]
    pub fn auto_register_slash(&self) -> bool {
        self.options.auto_register_slash
    }

    /// Build the Discord bulk-overwrite payload for every registered slash
    /// command (prefix-only commands are skipped). Each command's options
    /// come from the extra parameters its handler function declared.
    #[must_use]
    pub fn build_global_commands(&self) -> Vec<CreateCommand> {
        let mut out = Vec::new();
        for cmd in &self.options.commands {
            // 1. Registrasi slash command biasa (jika bukan prefix-only dan punya handler/subcommands).
            if !matches!(cmd.kind, CommandKind::Prefix)
                && (cmd.handler.is_some() || !cmd.subcommands.is_empty())
            {
                let mut builder = CreateCommand::new(&cmd.name).description(&cmd.description);
                for (locale, name) in &cmd.name_localizations {
                    builder = builder.name_localized(locale, name);
                }
                for (locale, desc) in &cmd.description_localizations {
                    builder = builder.description_localized(locale, desc);
                }
                if cmd.nsfw_only {
                    builder = builder.nsfw(true);
                }
                if let Some(perms) = cmd.default_member_permissions {
                    if !perms.is_empty() {
                        builder = builder.default_member_permissions(perms);
                    }
                }
                if let Some(install_ctx) = &cmd.install_context {
                    builder = builder.integration_types(install_ctx.clone());
                }
                if let Some(inter_ctx) = &cmd.interaction_context {
                    builder = builder.contexts(inter_ctx.clone());
                }
                if cmd.guild_only == Some(true) {
                    #[allow(deprecated)]
                    {
                        builder = builder.dm_permission(false);
                    }
                }
                let options = if cmd.subcommands.is_empty() {
                    command_options_as_slash(cmd)
                } else {
                    cmd.subcommands.iter().filter_map(command_as_subcommand_option).collect()
                };
                out.push(builder.set_options(options));
            }

            // 2. Registrasi context-menu command jika `context_menu_action` diset.
            if let Some(action) = cmd.context_menu_action {
                let cm_name = cmd.context_menu_name.as_deref().unwrap_or(&cmd.name);
                let kind = match action {
                    ContextMenuAction::User(_) => CommandType::User,
                    ContextMenuAction::Message(_) => CommandType::Message,
                };
                let mut builder = CreateCommand::new(cm_name).kind(kind);
                for (locale, name) in &cmd.name_localizations {
                    builder = builder.name_localized(locale, name);
                }
                if cmd.nsfw_only {
                    builder = builder.nsfw(true);
                }
                if let Some(perms) = cmd.default_member_permissions {
                    if !perms.is_empty() {
                        builder = builder.default_member_permissions(perms);
                    }
                }
                if let Some(install_ctx) = &cmd.install_context {
                    builder = builder.integration_types(install_ctx.clone());
                }
                if let Some(inter_ctx) = &cmd.interaction_context {
                    builder = builder.contexts(inter_ctx.clone());
                }
                if cmd.guild_only == Some(true) {
                    #[allow(deprecated)]
                    {
                        builder = builder.dm_permission(false);
                    }
                }
                out.push(builder);
            }
        }
        out
    }

    /// Register every slash command with Discord as a single bulk overwrite.
    ///
    /// Call this from your `EventHandler::ready` (or let the
    /// `oxcord::framework::v2_bridge` glue do it for you automatically when
    /// [`Self::auto_register_slash`] is set).
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Http`] if the request to Discord fails.
    pub async fn register_global_commands(&self, http: &Http) -> Result<(), FrameworkError> {
        let mut payload: Vec<serde_json::Value> = self
            .build_global_commands()
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()
            .map_err(|e| FrameworkError::Other(e.to_string()))?;
        // Bulk overwrite menghapus semua command yang tidak ikut dikirim, tapi Discord menolak
        // bila Entry Point command aplikasi (Activity) ikut terhapus. Jadi command bertipe
        // `PrimaryEntryPoint` yang sudah ada dibawa lagi apa adanya (hanya field yang boleh dikirim).
        let existing = http
            .get_global_commands()
            .await
            .map_err(|e| FrameworkError::Http(e.to_string()))?;
        for cmd in existing
            .iter()
            .filter(|c| c.kind == oxcord_model::application::CommandType::PrimaryEntryPoint)
        {
            payload.push(serde_json::json!({
                "id": cmd.id,
                "name": cmd.name,
                "description": cmd.description,
                "type": cmd.kind,
                "handler": cmd.handler,
            }));
        }
        http.create_global_commands(&payload)
            .await
            .map_err(|e| FrameworkError::Http(e.to_string()))?;
        Ok(())
    }

    /// Register semua slash command ke satu guild tertentu (berguna buat testing cepat atau `builtins::register`).
    ///
    /// # Errors
    ///
    /// Mengembalikan [`FrameworkError::Http`] jika request ke Discord gagal.
    pub async fn register_guild_commands(
        &self,
        http: &Http,
        guild_id: GuildId,
    ) -> Result<(), FrameworkError> {
        let commands = self.build_global_commands();
        http.create_guild_commands(guild_id, &commands)
            .await
            .map_err(|e| FrameworkError::Http(e.to_string()))?;
        Ok(())
    }

    /// Dispatch a gateway autocomplete interaction to the matching option's
    /// autocomplete handler, returning its string suggestions.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Other`] if the interaction carries no
    /// focused option, or if the matching command has no autocomplete
    /// handler registered for it; returns [`FrameworkError::UnknownCommand`]
    /// if no such command is registered.
    pub async fn dispatch_autocomplete(
        &self,
        command: &CommandInteraction,
        http: Arc<Http>,
        gateway: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<Vec<String>, FrameworkError> {
        let Some(focused) = find_focused_option(&command.data.options) else {
            return Err(FrameworkError::Other(
                "autocomplete interaction has no focused option".to_owned(),
            ));
        };
        let partial = match &focused.value {
            CommandDataOptionValue::Autocomplete {
                value, ..
            } => value.clone(),
            _ => unreachable!("filtered above"),
        };

        let is_thread = command.channel.as_ref().is_some_and(|c| is_thread_channel(c.kind));
        let data = InvocationData::Slash {
            token: command.token.clone(),
            guild_id: command.guild_id,
            options: command.data.options.clone(),
            member_permissions: adjust_for_thread(
                command.member.as_ref().and_then(|m| m.permissions),
                is_thread,
            ),
            app_permissions: adjust_for_thread(command.app_permissions, is_thread),
            is_thread,
        };
        let framework_http = Arc::new(FrameworkHttp::for_slash(
            http,
            command.id,
            command.application_id,
            command.channel_id,
        ));
        self.dispatcher
            .dispatch_autocomplete(
                &command.data.name,
                &focused.name,
                partial,
                &command.user,
                &data,
                framework_http,
                gateway,
            )
            .await
    }
}

/// Fluent builder for [`Framework`].
#[derive(Default)]
pub struct FrameworkBuilder {
    options: FrameworkOptions,
}

impl FrameworkBuilder {
    /// Set the full framework options.
    #[must_use]
    pub fn options(mut self, options: FrameworkOptions) -> Self {
        self.options = options;
        self
    }

    /// Register a single command.
    #[must_use]
    pub fn command(mut self, cmd: Command) -> Self {
        self.options.commands.push(cmd);
        self
    }

    /// Set the prefix.
    #[must_use]
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.options.prefix = Some(prefix.into());
        self
    }

    /// Add a bot owner.
    #[must_use]
    pub fn owner(mut self, id: UserId) -> Self {
        self.options.owners.insert(id);
        self
    }

    /// Enable automatic slash command registration on ready.
    #[must_use]
    pub fn auto_register_slash(mut self) -> Self {
        self.options.auto_register_slash = true;
        self
    }

    /// Set the shared, typed application state every command handler can
    /// read back via `Context::data::<T>()` (poise's `ctx.data()` for
    /// Oxcord). `T` is usually a struct of `Arc<Service>` fields.
    #[must_use]
    pub fn user_data<T: Any + Send + Sync + 'static>(mut self, data: T) -> Self {
        self.options.user_data = Some(Arc::new(data));
        self
    }

    /// Pasang hook global buat nangani [`FrameworkError`] — setara `on_error` poise.
    ///
    /// Hook dipanggil untuk error pre-flight (cooldown, guild-only, owner-only, permission)
    /// maupun error dari handler command, dan dapat [`ErrorContext`] yang bisa dipakai balas
    /// ke user (`err_ctx.ctx().say(..)`).
    ///
    /// ```rust,ignore
    /// Framework::builder().on_error(|err, ectx| Box::pin(async move {
    ///     let _ = ectx.ctx().say(format!("{err}")).await;
    /// }))
    /// ```
    #[must_use]
    pub fn on_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(FrameworkError, ErrorContext) -> Pin<Box<dyn Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        self.options.on_error = Some(Arc::new(hook));
        self
    }

    /// Set hook `pre_command` yang jalan sebelum setiap handler command.
    #[must_use]
    pub fn pre_command(mut self, hook: CommandHook) -> Self {
        self.options.pre_command = Some(hook);
        self
    }

    /// Set hook `post_command` yang jalan sesudah setiap handler command sukses.
    #[must_use]
    pub fn post_command(mut self, hook: CommandHook) -> Self {
        self.options.post_command = Some(hook);
        self
    }

    /// Set check global yang jalan sebelum check per-command.
    #[must_use]
    pub fn command_check(mut self, check: CheckFn) -> Self {
        self.options.command_check = Some(check);
        self
    }

    /// Skip semua pre-flight check untuk bot owner.
    #[must_use]
    pub fn skip_checks_for_owners(mut self, v: bool) -> Self {
        self.options.skip_checks_for_owners = v;
        self
    }

    /// Set default `allowed_mentions` untuk semua outgoing reply.
    #[must_use]
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
        self.options.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Set `reply_callback` untuk memodifikasi semua outgoing reply.
    #[must_use]
    pub fn reply_callback(mut self, cb: ReplyCallback) -> Self {
        self.options.reply_callback = Some(cb);
        self
    }

    /// Aktifkan mode `manual_cooldowns` (framework tidak otomatis cek/record cooldown).
    #[must_use]
    pub fn manual_cooldowns(mut self, v: bool) -> Self {
        self.options.manual_cooldowns = v;
        self
    }

    /// Wajibkan guild ada di cache untuk `guild_only` check.
    #[must_use]
    pub fn require_cache_for_guild_check(mut self, v: bool) -> Self {
        self.options.require_cache_for_guild_check = v;
        self
    }

    /// Aktifkan pengisian otomatis `owners` dari `get_current_application_info()` saat startup.
    #[must_use]
    pub fn initialize_owners(mut self, v: bool) -> Self {
        self.options.initialize_owners = v;
        self
    }

    /// Set daftar role team Discord yang ikut dimasukkan ke `owners` saat `initialize_owners` aktif.
    #[must_use]
    pub fn initialized_team_roles(mut self, roles: Vec<TeamMemberRole>) -> Self {
        self.options.initialized_team_roles = Some(roles);
        self
    }

    /// Set konfigurasi lanjutan [`PrefixOptions`].
    #[must_use]
    pub fn prefix_options(mut self, opts: PrefixOptions) -> Self {
        self.options.prefix_options = opts;
        self
    }

    /// Finalise and return the [`Framework`].
    #[must_use]
    pub fn build(self) -> Framework {
        let dispatcher = Dispatcher::new(&self.options);
        Framework {
            options: self.options,
            dispatcher,
        }
    }
}

#[cfg(test)]
mod noprefix_tests {
    use std::sync::Arc;

    use oxcord_model::id::UserId;

    use super::*;

    fn framework() -> Framework {
        let noprefix: crate::prefix::NoPrefixFn = Arc::new(|_, author, _, _| {
            Box::pin(async move { author == UserId::new(7) })
        });
        Framework::builder()
            .options(FrameworkOptions {
                prefix: Some("!".to_owned()),
                prefix_options: PrefixOptions {
                    noprefix: Some(noprefix),
                    ..Default::default()
                },
                ..Default::default()
            })
            .build()
    }

    #[tokio::test]
    async fn noprefix_user_matches_without_prefix_and_others_do_not() {
        let fw = framework();
        let hit = fw.strip_any_prefix("ping now", None, UserId::new(7), None).await.unwrap();
        assert_eq!(hit, Some(("", "ping now")));
        let miss = fw.strip_any_prefix("ping now", None, UserId::new(8), None).await.unwrap();
        assert_eq!(miss, None);
    }

    #[tokio::test]
    async fn normal_prefix_wins_over_noprefix() {
        let fw = framework();
        let hit = fw.strip_any_prefix("!ping", None, UserId::new(7), None).await.unwrap();
        assert_eq!(hit, Some(("!", "ping")));
    }
}
