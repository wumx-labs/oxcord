//! Command dispatch logic.
//!
//! [`Dispatcher`] receives raw invocation data and routes it to the
//! matching [`Command`] handler, running pre-flight checks (cooldowns,
//! guild-only, owner-only, permissions) before calling the user function.

use std::any::Any;
use std::collections::HashSet;
use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use std::time::Instant;

use oxcord_model::application::CommandDataOptionValue;
use oxcord_model::guild::Member;
use oxcord_model::id::UserId;
use parking_lot::RwLock;
use tracing::{debug, warn};

use crate::command::{CheckFn, Command};
use crate::context::{Context, ErrorContext, FrameworkHttp, InteractionBus, InvocationData};
use crate::cooldown::CooldownManager;
use crate::error::FrameworkError;
use crate::framework::{CommandHook, ErrorHook, FrameworkOptions};

/// Future wrapper yang menangkap panic dari handler command supaya bot nggak ikut crash.
struct CatchUnwindFuture<F>(AssertUnwindSafe<F>);

impl<F: Future> Future for CatchUnwindFuture<F> {
    type Output = Result<F::Output, Box<dyn Any + Send + 'static>>;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        // SAFETY: kita cuma memproyeksikan pin ke inner future F di dalam AssertUnwindSafe.
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.0.0) };
        match catch_unwind(AssertUnwindSafe(|| inner.poll(cx))) {
            Ok(Poll::Ready(val)) => Poll::Ready(Ok(val)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(panic_payload) => Poll::Ready(Err(panic_payload)),
        }
    }
}

fn extract_panic_payload(payload: &Box<dyn Any + Send + 'static>) -> Option<String> {
    if let Some(s) = payload.downcast_ref::<&str>() {
        Some((*s).to_owned())
    } else {
        payload.downcast_ref::<String>().cloned()
    }
}

/// Internal invocation request built by [`Framework`][crate::Framework] before
/// handing off to the dispatcher.
pub struct Invocation<'a> {
    /// The command name exactly as invoked (without prefix).
    pub command_name: &'a str,
    /// The user who invoked the command.
    pub author: &'a oxcord_model::user::User,
    /// Raw invocation data.
    pub data: &'a InvocationData,
    /// A lightweight HTTP handle.
    pub http: Arc<FrameworkHttp>,
    /// When the invocation was received, before any dispatch work started.
    pub received_at: Instant,
    /// Context gateway mentah dari pemanggil (lihat [`Context::gateway_context`]).
    pub gateway: Option<Arc<dyn Any + Send + Sync>>,
    /// Apakah channel tempat invocation terjadi bertipe NSFW (`None` jika tidak diketahui).
    pub is_nsfw: Option<bool>,
    /// Apakah guild tempat invocation berada ada di cache (`None` jika fitur cache tidak aktif).
    pub guild_in_cache: Option<bool>,
    /// Locale user dari slash interaction (`None` untuk prefix).
    pub locale: Option<String>,
    /// Data member pemanggil dari slash interaction (kalau ada).
    pub author_member: Option<Member>,
    /// String invocation lengkap buat logging/debug.
    pub invocation_string: String,
}

/// Routes invocations to registered commands.
#[allow(clippy::struct_excessive_bools)]
pub struct Dispatcher {
    commands: Vec<Command>,
    owners: Arc<RwLock<HashSet<UserId>>>,
    cooldowns: dashmap::DashMap<String, CooldownManager>,
    user_data: Option<Arc<dyn Any + Send + Sync>>,
    on_error: Option<ErrorHook>,
    pre_command: Option<CommandHook>,
    post_command: Option<CommandHook>,
    command_check: Option<CheckFn>,
    skip_checks_for_owners: bool,
    manual_cooldowns: bool,
    require_cache_for_guild_check: bool,
    case_insensitive_commands: bool,
    interaction_bus: Arc<InteractionBus>,
}

impl Dispatcher {
    /// Build a new dispatcher from framework options.
    ///
    /// Any command built with [`crate::command::CommandBuilder::cooldown`]
    /// (including via the `#[oxcord_macros::command]` macro) has its cooldown
    /// manager registered automatically — no manual
    /// [`Self::register_cooldown`] call needed.
    #[must_use]
    pub fn new(options: &FrameworkOptions) -> Self {
        let cooldowns = dashmap::DashMap::new();
        for cmd in &options.commands {
            Self::register_command_cooldowns(cmd, &cooldowns);
        }
        Self {
            commands: options.commands.clone(),
            owners: Arc::new(RwLock::new(options.owners.clone())),
            cooldowns,
            user_data: options.user_data.clone(),
            on_error: options.on_error.clone(),
            pre_command: options.pre_command.clone(),
            post_command: options.post_command.clone(),
            command_check: options.command_check,
            skip_checks_for_owners: options.skip_checks_for_owners,
            manual_cooldowns: options.manual_cooldowns,
            require_cache_for_guild_check: options.require_cache_for_guild_check,
            case_insensitive_commands: options.prefix_options.case_insensitive_commands,
            interaction_bus: Arc::new(InteractionBus::new()),
        }
    }

    fn register_command_cooldowns(
        cmd: &Command,
        cooldowns: &dashmap::DashMap<String, CooldownManager>,
    ) {
        if let Some(config) = cmd.cooldown.clone() {
            cooldowns.insert(cmd.name.clone(), CooldownManager::new(config));
        }
        for sub in &cmd.subcommands {
            Self::register_command_cooldowns(sub, cooldowns);
        }
    }

    /// Akses daftar owner yang bisa di-update saat runtime (misal oleh `initialize_owners`).
    #[must_use]
    pub fn owners(&self) -> &Arc<RwLock<HashSet<UserId>>> {
        &self.owners
    }

    /// Akses `InteractionBus` untuk meneruskan component/modal interaction dari bridge.
    #[must_use]
    pub fn interaction_bus(&self) -> &Arc<InteractionBus> {
        &self.interaction_bus
    }

    /// Dispatch an invocation, running all pre-flight checks.
    ///
    /// # Errors
    ///
    /// Returns the first [`FrameworkError`] encountered (cooldown, permission
    /// check, or a propagated handler error).
    pub async fn dispatch(&self, inv: Invocation<'_>) -> Result<(), FrameworkError> {
        let top = self
            .find_command(inv.command_name)
            .ok_or_else(|| FrameworkError::UnknownCommand(inv.command_name.to_owned()))?;

        // --- Resolve subcommands, if any ---
        //
        // A command's own options are only meaningful for its leaf command, so
        // this descends `cmd.subcommands` as far as the invocation picks a
        // branch (slash: Discord's `SubCommand`/`SubCommandGroup` option
        // nesting; prefix: whitespace-separated tokens), rewriting a local copy
        // of the invocation data to carry only the leaf's own options/args.
        // Falls back to the group command itself (which normally has no
        // handler, producing a clear "no handler" error below) if no branch
        // matches.
        let mut resolved_data = inv.data.clone();
        let cmd = Self::resolve_subcommand_with_case(
            top,
            &mut resolved_data,
            self.case_insensitive_commands,
        );
        let inv_data = &resolved_data;

        let guild_id = match inv_data {
            InvocationData::Slash { guild_id, .. } | InvocationData::Prefix { guild_id, .. } => {
                *guild_id
            },
        };

        // Context dibentuk DI AWAL (sebelum pre-flight check apa pun), supaya hook `on_error`
        // selalu punya sesuatu buat balas ke Discord — termasuk waktu cooldown/permission/
        // guild-only yang gagal sebelum handler sempat dipanggil.
        let ctx = Context {
            author: inv.author,
            channel_id: inv.http.channel_id,
            guild_id,
            invocation: inv_data,
            http: inv.http,
            data: self.user_data.clone(),
            received_at: inv.received_at,
            gateway: inv.gateway,
            command: Some(cmd),
            invocation_data: Arc::new(tokio::sync::Mutex::new(Box::new(()))),
            locale: inv.locale,
            author_member: inv.author_member,
            invocation_string: inv.invocation_string,
            interaction_bus: Some(Arc::clone(&self.interaction_bus)),
        };

        match self.run(cmd, ctx.clone(), inv.is_nsfw, inv.guild_in_cache).await {
            Ok(()) => Ok(()),
            Err(error) => self.report_error(error, cmd, &ctx).await,
        }
    }

    /// Serahkan error ke hook `on_error` per-command kalau ada, kalau nggak ada fallback ke
    /// `FrameworkOptions::on_error` global. Kalau keduanya nggak ada, kembalikan lagi biar
    /// pemanggil (bridge) yang nge-log.
    async fn report_error(
        &self,
        error: FrameworkError,
        cmd: &Command,
        ctx: &Context<'_>,
    ) -> Result<(), FrameworkError> {
        let hook = cmd.on_error.as_ref().or(self.on_error.as_ref());
        match hook {
            Some(hook) => {
                hook(error, ErrorContext::from_ctx(&cmd.name, ctx)).await;
                Ok(())
            },
            None => Err(error),
        }
    }

    /// Pre-flight check sesuai urutan poise:
    /// `owner_only` -> (skip_checks_for_owners?) -> `guild_only` -> `dm_only` -> `nsfw_only`
    /// -> `permissions` -> `checks` (global lalu per-command) -> `cooldown`,
    /// lalu `pre_command` -> `handler` (dengan catch_unwind) -> `post_command`.
    async fn run(
        &self,
        cmd: &Command,
        ctx: Context<'_>,
        is_nsfw: Option<bool>,
        guild_in_cache: Option<bool>,
    ) -> Result<(), FrameworkError> {
        // Jika grup punya subcommand dan `subcommand_required`, wajib pilih subcommand.
        if !cmd.subcommands.is_empty() && cmd.subcommand_required {
            return Err(FrameworkError::SubcommandRequired);
        }

        let guild_id = ctx.guild_id;
        let channel_id = ctx.channel_id;
        let inv_data = ctx.invocation;
        let is_owner = self.owners.read().contains(&ctx.author.id);

        // --- Pre-flight: owner-only (paling awal, sama seperti poise) ---
        if cmd.owner_only && !is_owner {
            return Err(FrameworkError::OwnerOnly);
        }

        let skip_checks = self.skip_checks_for_owners && is_owner;

        if !skip_checks {
            // --- Pre-flight: guild-only ---
            if cmd.guild_only == Some(true) {
                if guild_id.is_none() {
                    return Err(FrameworkError::GuildOnly);
                }
                if self.require_cache_for_guild_check && guild_in_cache == Some(false) {
                    return Err(FrameworkError::GuildOnly);
                }
            }

            // --- Pre-flight: dm-only ---
            if cmd.dm_only && guild_id.is_some() {
                return Err(FrameworkError::DmOnly);
            }

            // --- Pre-flight: nsfw-only ---
            if cmd.nsfw_only && is_nsfw != Some(true) {
                return Err(FrameworkError::NsfwOnly);
            }

            // --- Pre-flight: permissions ---
            if cmd.required_permissions.is_some() || cmd.required_bot_permissions.is_some() {
                let (member_permissions, app_permissions) = match inv_data {
                    InvocationData::Slash { member_permissions, app_permissions, .. }
                    | InvocationData::Prefix { member_permissions, app_permissions, .. } => {
                        (*member_permissions, *app_permissions)
                    },
                };
                if let Some(required) = cmd.required_permissions {
                    let Some(have) = member_permissions else {
                        return Err(FrameworkError::PermissionCheckUnavailable);
                    };
                    let missing = required - have;
                    if !missing.is_empty() {
                        return Err(FrameworkError::MissingUserPermissions { missing });
                    }
                }
                if let Some(required) = cmd.required_bot_permissions {
                    let Some(have) = app_permissions else {
                        return Err(FrameworkError::PermissionCheckUnavailable);
                    };
                    let missing = required - have;
                    if !missing.is_empty() {
                        return Err(FrameworkError::MissingBotPermissions { missing });
                    }
                }
            }

            // --- Pre-flight: checks (global `command_check` dulu, baru `cmd.checks`) ---
            for check_fn in self.command_check.iter().chain(cmd.checks.iter()) {
                match check_fn(ctx.clone()).await {
                    Ok(true) => {},
                    Ok(false) => return Err(FrameworkError::CommandCheckFailed(None)),
                    Err(e) => return Err(FrameworkError::CommandCheckFailed(Some(Box::new(e)))),
                }
            }

            // --- Pre-flight: cooldown ---
            if !self.manual_cooldowns {
                if let Some(mgr) = self.cooldowns.get(&cmd.name) {
                    if let Some(remaining) = mgr.check(ctx.author.id, guild_id, channel_id) {
                        return Err(FrameworkError::Cooldown {
                            remaining_secs: remaining.as_secs_f64(),
                        });
                    }
                }

                // Catat invocation sebelum jalan (cegah spam double-trigger).
                if let Some(mgr) = self.cooldowns.get(&cmd.name) {
                    mgr.record(ctx.author.id, guild_id, channel_id);
                }
            }
        }

        // --- Broadcast typing (prefix-only) ---
        if cmd.broadcast_typing && matches!(inv_data, InvocationData::Prefix { .. }) {
            let _ = ctx.http.http.broadcast_typing(channel_id).await;
        }

        // --- Dispatch ---
        let Some(handler) = &cmd.handler else {
            warn!(command = %cmd.name, "command has no handler registered");
            return Err(FrameworkError::Other(format!("'{}' has no handler", cmd.name)));
        };

        if let Some(pre) = &self.pre_command {
            pre(ctx.clone()).await;
        }

        debug!(command = %cmd.name, "dispatching command");
        let result = match CatchUnwindFuture(AssertUnwindSafe(handler.call(ctx.clone()))).await {
            Ok(res) => res,
            Err(panic_payload) => {
                Err(FrameworkError::CommandPanic { payload: extract_panic_payload(&panic_payload) })
            },
        };

        if result.is_ok() {
            if let Some(post) = &self.post_command {
                post(ctx).await;
            }
        }

        result
    }

    /// Descend `cmd.subcommands` as far as the invocation selects a branch,
    /// rewriting `data`'s options/args in place to the resolved leaf's own.
    /// See [`Self::dispatch`] for why this exists.
    fn resolve_subcommand<'c>(cmd: &'c Command, data: &mut InvocationData) -> &'c Command {
        Self::resolve_subcommand_with_case(cmd, data, false)
    }

    fn resolve_subcommand_with_case<'c>(
        mut cmd: &'c Command,
        data: &mut InvocationData,
        case_insensitive: bool,
    ) -> &'c Command {
        loop {
            if cmd.subcommands.is_empty() {
                return cmd;
            }
            match data {
                InvocationData::Slash { options, .. } => {
                    let Some(opt) = options.first() else {
                        return cmd;
                    };
                    let inner = match &opt.value {
                        CommandDataOptionValue::SubCommand(inner)
                        | CommandDataOptionValue::SubCommandGroup(inner) => inner.clone(),
                        _ => return cmd,
                    };
                    let Some(sub) = cmd.subcommands.iter().find(|s| s.name == opt.name) else {
                        return cmd;
                    };
                    *options = inner;
                    cmd = sub;
                },
                InvocationData::Prefix { args, .. } => {
                    let mut parts = args.trim_start().splitn(2, char::is_whitespace);
                    let Some(candidate) = parts.next().filter(|s| !s.is_empty()) else {
                        return cmd;
                    };
                    let matches_name = |target: &str| {
                        if case_insensitive {
                            target.eq_ignore_ascii_case(candidate)
                        } else {
                            target == candidate
                        }
                    };
                    let Some(sub) = cmd.subcommands.iter().find(|s| {
                        matches_name(&s.name) || s.aliases.iter().any(|a| matches_name(a))
                    }) else {
                        return cmd;
                    };
                    *args = parts.next().unwrap_or("").trim_start().to_owned();
                    cmd = sub;
                },
            }
        }
    }

    /// Run the autocomplete handler registered for a named option of a
    /// command, if any, returning its string suggestions.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::UnknownCommand`] if no such command is
    /// registered, or [`FrameworkError::Other`] if that command has no
    /// autocomplete handler for `option_name`.
    #[allow(clippy::too_many_arguments)]
    pub async fn dispatch_autocomplete<'a>(
        &self,
        command_name: &str,
        option_name: &str,
        partial: String,
        author: &'a oxcord_model::user::User,
        data: &'a InvocationData,
        http: Arc<FrameworkHttp>,
        gateway: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<Vec<String>, FrameworkError> {
        let top = self
            .find_command(command_name)
            .ok_or_else(|| FrameworkError::UnknownCommand(command_name.to_owned()))?;
        let mut resolved_data = data.clone();
        let cmd = Self::resolve_subcommand(top, &mut resolved_data);
        let data = &resolved_data;
        let Some(handler) = cmd.autocomplete.get(option_name) else {
            return Err(FrameworkError::Other(format!(
                "'{command_name}' has no autocomplete handler for '{option_name}'"
            )));
        };
        let guild_id = match data {
            InvocationData::Slash { guild_id, .. } | InvocationData::Prefix { guild_id, .. } => {
                *guild_id
            },
        };
        let ctx = Context {
            author,
            channel_id: http.channel_id,
            guild_id,
            invocation: data,
            http,
            data: self.user_data.clone(),
            received_at: Instant::now(),
            gateway,
            command: Some(cmd),
            invocation_data: Arc::new(tokio::sync::Mutex::new(Box::new(()))),
            locale: None,
            author_member: None,
            invocation_string: format!("/{command_name}"),
            interaction_bus: Some(Arc::clone(&self.interaction_bus)),
        };
        handler.call(ctx, partial).await
    }

    /// Register a cooldown manager for a command by name.
    pub fn register_cooldown(&self, command_name: impl Into<String>, mgr: CooldownManager) {
        self.cooldowns.insert(command_name.into(), mgr);
    }

    /// Cari command berdasarkan nama atau alias (menghormati `case_insensitive_commands` untuk prefix).
    #[must_use]
    pub fn find_command(&self, name: &str) -> Option<&Command> {
        self.commands.iter().find(|c| {
            if self.case_insensitive_commands {
                c.name.eq_ignore_ascii_case(name)
                    || c.aliases.iter().any(|a| a.eq_ignore_ascii_case(name))
            } else {
                c.name == name || c.aliases.iter().any(|a| a == name)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandKind;

    fn leaf(name: &str) -> Command {
        Command::builder(name).kind(CommandKind::Both).build()
    }

    fn group(name: &str, subcommands: Vec<Command>) -> Command {
        let mut builder = Command::builder(name).kind(CommandKind::Both);
        for sub in subcommands {
            builder = builder.subcommand(sub);
        }
        builder.build()
    }

    #[test]
    fn prefix_resolves_one_level_of_subcommand_and_strips_the_token() {
        let cmd = group("config", vec![leaf("set"), leaf("get")]);
        let mut data = InvocationData::Prefix {
            args: "set foo bar".to_owned(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };
        let resolved = Dispatcher::resolve_subcommand(&cmd, &mut data);
        assert_eq!(resolved.name, "set");
        match data {
            InvocationData::Prefix { args, .. } => assert_eq!(args, "foo bar"),
            InvocationData::Slash { .. } => panic!("expected Prefix"),
        }
    }

    #[test]
    fn prefix_falls_back_to_group_when_no_branch_matches() {
        let cmd = group("config", vec![leaf("set")]);
        let mut data = InvocationData::Prefix {
            args: "nonexistent".to_owned(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };
        let resolved = Dispatcher::resolve_subcommand(&cmd, &mut data);
        assert_eq!(resolved.name, "config");
    }

    #[test]
    fn prefix_resolves_through_a_subcommand_group_two_levels_deep() {
        let cmd = group("config", vec![group("prefix", vec![leaf("set"), leaf("get")])]);
        let mut data = InvocationData::Prefix {
            args: "prefix set !".to_owned(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };
        let resolved = Dispatcher::resolve_subcommand(&cmd, &mut data);
        assert_eq!(resolved.name, "set");
        match data {
            InvocationData::Prefix { args, .. } => assert_eq!(args, "!"),
            InvocationData::Slash { .. } => panic!("expected Prefix"),
        }
    }

    #[test]
    fn leaf_command_with_no_subcommands_is_returned_unchanged() {
        let cmd = leaf("ping");
        let mut data = InvocationData::Prefix {
            args: String::new(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };
        let resolved = Dispatcher::resolve_subcommand(&cmd, &mut data);
        assert_eq!(resolved.name, "ping");
    }

    // --- Hook on_error ---

    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Mutex;
    use std::time::Duration;

    use oxcord_model::id::ChannelId;
    use oxcord_rest::Http;

    use crate::cooldown::{CooldownConfig, CooldownScope};

    fn ok_handler(
        _ctx: Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }

    fn cooldown_command() -> Command {
        Command::builder("ping")
            .kind(CommandKind::Both)
            .cooldown(CooldownConfig::new(CooldownScope::User, Duration::from_secs(60)))
            .handler(ok_handler)
            .build()
    }

    /// Jalankan satu invocation prefix `ping` oleh user yang sama.
    async fn invoke_ping(dispatcher: &Dispatcher) -> Result<(), FrameworkError> {
        let author = oxcord_model::user::User::default();
        let data = InvocationData::Prefix {
            args: String::new(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };
        let http = Arc::new(FrameworkHttp::for_prefix(
            Arc::new(Http::new("test-token")),
            ChannelId::new(1),
        ));
        dispatcher
            .dispatch(Invocation {
                command_name: "ping",
                author: &author,
                data: &data,
                http,
                received_at: Instant::now(),
                gateway: None,
                is_nsfw: None,
                guild_in_cache: None,
                locale: None,
                author_member: None,
                invocation_string: "!ping".to_owned(),
            })
            .await
    }

    #[tokio::test]
    async fn on_error_hook_receives_cooldown_error_and_dispatch_returns_ok() {
        let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let seen_in_hook = Arc::clone(&seen);
        let options = FrameworkOptions {
            commands: vec![cooldown_command()],
            on_error: Some(Arc::new(move |error, ectx| {
                let seen = Arc::clone(&seen_in_hook);
                Box::pin(async move {
                    let label = match error {
                        FrameworkError::Cooldown { .. } => {
                            format!("cooldown:{}", ectx.command_name)
                        },
                        other => format!("other:{other}"),
                    };
                    seen.lock().expect("mutex").push(label);
                })
            })),
            ..Default::default()
        };
        let dispatcher = Dispatcher::new(&options);

        // Pemanggilan pertama lolos, yang kedua kena cooldown -> hook dipanggil.
        assert!(invoke_ping(&dispatcher).await.is_ok());
        assert!(seen.lock().expect("mutex").is_empty());
        // Hook sudah nangani error-nya, jadi dispatch balikin Ok (nggak dobel handle di bridge).
        assert!(invoke_ping(&dispatcher).await.is_ok());
        assert_eq!(*seen.lock().expect("mutex"), vec!["cooldown:ping".to_owned()]);
    }

    #[tokio::test]
    async fn without_on_error_hook_the_error_is_returned_as_before() {
        let options = FrameworkOptions { commands: vec![cooldown_command()], ..Default::default() };
        let dispatcher = Dispatcher::new(&options);
        assert!(invoke_ping(&dispatcher).await.is_ok());
        assert!(matches!(invoke_ping(&dispatcher).await, Err(FrameworkError::Cooldown { .. })));
    }

    // --- Helper & unit test baru untuk paritas poise ---

    fn make_inv<'a>(
        command_name: &'a str,
        author: &'a oxcord_model::user::User,
        data: &'a InvocationData,
        is_nsfw: Option<bool>,
        guild_in_cache: Option<bool>,
    ) -> Invocation<'a> {
        let http = Arc::new(FrameworkHttp::for_prefix(
            Arc::new(Http::new("test-token")),
            ChannelId::new(1),
        ));
        Invocation {
            command_name,
            author,
            data,
            http,
            received_at: Instant::now(),
            gateway: None,
            is_nsfw,
            guild_in_cache,
            locale: None,
            author_member: None,
            invocation_string: format!("!{command_name}"),
        }
    }

    #[tokio::test]
    async fn dm_only_allows_dm_and_rejects_guild() {
        let cmd = Command::builder("secret")
            .kind(CommandKind::Both)
            .dm_only(true)
            .handler(ok_handler)
            .build();
        let dispatcher =
            Dispatcher::new(&FrameworkOptions { commands: vec![cmd], ..Default::default() });
        let author = oxcord_model::user::User::default();

        let dm_data = InvocationData::Prefix {
            args: String::new(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };
        assert!(
            dispatcher.dispatch(make_inv("secret", &author, &dm_data, None, None)).await.is_ok()
        );

        let guild_data = InvocationData::Prefix {
            args: String::new(),
            guild_id: Some(oxcord_model::id::GuildId::new(42)),
            member_permissions: None,
            app_permissions: None,
        };
        assert!(matches!(
            dispatcher.dispatch(make_inv("secret", &author, &guild_data, None, None)).await,
            Err(FrameworkError::DmOnly)
        ));
    }

    #[tokio::test]
    async fn nsfw_only_requires_nsfw_true() {
        let cmd = Command::builder("spicy")
            .kind(CommandKind::Both)
            .nsfw_only(true)
            .handler(ok_handler)
            .build();
        let dispatcher =
            Dispatcher::new(&FrameworkOptions { commands: vec![cmd], ..Default::default() });
        let author = oxcord_model::user::User::default();
        let data = InvocationData::Prefix {
            args: String::new(),
            guild_id: Some(oxcord_model::id::GuildId::new(42)),
            member_permissions: None,
            app_permissions: None,
        };

        assert!(matches!(
            dispatcher.dispatch(make_inv("spicy", &author, &data, None, None)).await,
            Err(FrameworkError::NsfwOnly)
        ));
        assert!(matches!(
            dispatcher.dispatch(make_inv("spicy", &author, &data, Some(false), None)).await,
            Err(FrameworkError::NsfwOnly)
        ));
        assert!(
            dispatcher.dispatch(make_inv("spicy", &author, &data, Some(true), None)).await.is_ok()
        );
    }

    fn check_pass(
        _ctx: Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<bool, FrameworkError>> + Send + '_>> {
        Box::pin(async { Ok(true) })
    }

    fn check_fail(
        _ctx: Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<bool, FrameworkError>> + Send + '_>> {
        Box::pin(async { Ok(false) })
    }

    #[tokio::test]
    async fn checks_and_global_command_check_order() {
        let cmd_fail = Command::builder("c1")
            .kind(CommandKind::Both)
            .check(check_pass)
            .check(check_fail)
            .handler(ok_handler)
            .build();
        let cmd_pass = Command::builder("c2")
            .kind(CommandKind::Both)
            .check(check_pass)
            .handler(ok_handler)
            .build();

        let dispatcher_global_fail = Dispatcher::new(&FrameworkOptions {
            commands: vec![cmd_pass.clone()],
            command_check: Some(check_fail),
            ..Default::default()
        });
        let author = oxcord_model::user::User::default();
        let data = InvocationData::Prefix {
            args: String::new(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };
        assert!(matches!(
            dispatcher_global_fail.dispatch(make_inv("c2", &author, &data, None, None)).await,
            Err(FrameworkError::CommandCheckFailed(None))
        ));

        let dispatcher_local_fail = Dispatcher::new(&FrameworkOptions {
            commands: vec![cmd_fail, cmd_pass],
            command_check: Some(check_pass),
            ..Default::default()
        });
        assert!(matches!(
            dispatcher_local_fail.dispatch(make_inv("c1", &author, &data, None, None)).await,
            Err(FrameworkError::CommandCheckFailed(None))
        ));
        assert!(
            dispatcher_local_fail
                .dispatch(make_inv("c2", &author, &data, None, None))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn skip_checks_for_owners_bypasses_guild_nsfw_and_checks() {
        let mut owners = HashSet::new();
        owners.insert(UserId::new(999));

        let cmd = Command::builder("admin")
            .kind(CommandKind::Both)
            .guild_only(true)
            .nsfw_only(true)
            .check(check_fail)
            .handler(ok_handler)
            .build();

        let dispatcher = Dispatcher::new(&FrameworkOptions {
            commands: vec![cmd],
            owners,
            skip_checks_for_owners: true,
            ..Default::default()
        });

        let owner_user = oxcord_model::user::User { id: UserId::new(999), ..Default::default() };
        let normal_user = oxcord_model::user::User { id: UserId::new(111), ..Default::default() };

        let dm_data = InvocationData::Prefix {
            args: String::new(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };

        assert!(
            dispatcher
                .dispatch(make_inv("admin", &owner_user, &dm_data, Some(false), None))
                .await
                .is_ok()
        );
        assert!(matches!(
            dispatcher.dispatch(make_inv("admin", &normal_user, &dm_data, Some(false), None)).await,
            Err(FrameworkError::GuildOnly)
        ));
    }

    #[tokio::test]
    async fn subcommand_required_rejects_bare_group_and_allows_subcommand() {
        let sub = Command::builder("set").kind(CommandKind::Both).handler(ok_handler).build();
        let grp = Command::builder("cfg")
            .kind(CommandKind::Both)
            .subcommand(sub)
            .subcommand_required(true)
            .build();
        let dispatcher =
            Dispatcher::new(&FrameworkOptions { commands: vec![grp], ..Default::default() });
        let author = oxcord_model::user::User::default();

        let bare = InvocationData::Prefix {
            args: String::new(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };
        assert!(matches!(
            dispatcher.dispatch(make_inv("cfg", &author, &bare, None, None)).await,
            Err(FrameworkError::SubcommandRequired)
        ));

        let with_sub = InvocationData::Prefix {
            args: "set".to_owned(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };
        assert!(dispatcher.dispatch(make_inv("cfg", &author, &with_sub, None, None)).await.is_ok());
    }

    fn err_handler(
        _ctx: Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send + '_>> {
        Box::pin(async { Err(FrameworkError::Other("boom".to_owned())) })
    }

    fn panic_handler(
        _ctx: Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send + '_>> {
        Box::pin(async {
            panic!("handler exploded");
        })
    }

    #[tokio::test]
    async fn pre_and_post_command_hooks_and_per_command_on_error_and_panic_catch() {
        let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let log_pre = Arc::clone(&log);
        let log_post = Arc::clone(&log);
        let log_cmd_err = Arc::clone(&log);
        let log_global_err = Arc::clone(&log);

        let ok_cmd = Command::builder("ok").kind(CommandKind::Both).handler(ok_handler).build();
        let fail_cmd = Command::builder("fail")
            .kind(CommandKind::Both)
            .handler(err_handler)
            .on_error(move |err, ectx| {
                let l = Arc::clone(&log_cmd_err);
                Box::pin(async move {
                    l.lock().expect("mutex").push(format!("cmd_err:{}:{err}", ectx.command_name));
                })
            })
            .build();
        let panic_cmd =
            Command::builder("boom").kind(CommandKind::Both).handler(panic_handler).build();

        let dispatcher = Dispatcher::new(&FrameworkOptions {
            commands: vec![ok_cmd, fail_cmd, panic_cmd],
            pre_command: Some(Arc::new(move |ctx| {
                let l = Arc::clone(&log_pre);
                Box::pin(async move {
                    l.lock()
                        .expect("mutex")
                        .push(format!("pre:{}", ctx.command().expect("command").name));
                })
            })),
            post_command: Some(Arc::new(move |ctx| {
                let l = Arc::clone(&log_post);
                Box::pin(async move {
                    l.lock()
                        .expect("mutex")
                        .push(format!("post:{}", ctx.command().expect("command").name));
                })
            })),
            on_error: Some(Arc::new(move |err, ectx| {
                let l = Arc::clone(&log_global_err);
                Box::pin(async move {
                    l.lock()
                        .expect("mutex")
                        .push(format!("global_err:{}:{err}", ectx.command_name));
                })
            })),
            ..Default::default()
        });

        let author = oxcord_model::user::User::default();
        let data = InvocationData::Prefix {
            args: String::new(),
            guild_id: None,
            member_permissions: None,
            app_permissions: None,
        };

        assert!(dispatcher.dispatch(make_inv("ok", &author, &data, None, None)).await.is_ok());
        assert!(dispatcher.dispatch(make_inv("fail", &author, &data, None, None)).await.is_ok());
        assert!(dispatcher.dispatch(make_inv("boom", &author, &data, None, None)).await.is_ok());

        let entries = log.lock().expect("mutex").clone();
        assert_eq!(
            entries,
            vec![
                "pre:ok".to_owned(),
                "post:ok".to_owned(),
                "pre:fail".to_owned(),
                "cmd_err:fail:boom".to_owned(),
                "pre:boom".to_owned(),
                "global_err:boom:Command panicked: handler exploded".to_owned(),
            ]
        );
    }

    #[tokio::test]
    async fn manual_cooldowns_and_require_cache_for_guild_check() {
        let cmd = Command::builder("mc")
            .kind(CommandKind::Both)
            .guild_only(true)
            .cooldown(CooldownConfig::new(CooldownScope::User, Duration::from_secs(60)))
            .handler(ok_handler)
            .build();
        let dispatcher = Dispatcher::new(&FrameworkOptions {
            commands: vec![cmd],
            manual_cooldowns: true,
            require_cache_for_guild_check: true,
            ..Default::default()
        });

        let author = oxcord_model::user::User::default();
        let guild_data = InvocationData::Prefix {
            args: String::new(),
            guild_id: Some(oxcord_model::id::GuildId::new(42)),
            member_permissions: None,
            app_permissions: None,
        };

        // Guild tidak ada di cache -> GuildOnly
        assert!(matches!(
            dispatcher.dispatch(make_inv("mc", &author, &guild_data, None, Some(false))).await,
            Err(FrameworkError::GuildOnly)
        ));

        // Guild ada di cache, manual_cooldowns = true -> lolos berkali-kali
        assert!(
            dispatcher
                .dispatch(make_inv("mc", &author, &guild_data, None, Some(true)))
                .await
                .is_ok()
        );
        assert!(
            dispatcher
                .dispatch(make_inv("mc", &author, &guild_data, None, Some(true)))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn custom_data_and_invocation_data_and_prefix_options_and_edit_tracker() {
        use crate::prefix::PrefixOptions;
        use crate::track_edits::EditTracker;

        fn inspect_handler(
            ctx: Context<'_>,
        ) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send + '_>> {
            Box::pin(async move {
                assert_eq!(ctx.command_custom_data::<&'static str>(), Some(&"meta-v1"));
                ctx.set_invocation_data(42usize).await;
                let val = ctx.invocation_data::<usize>().await;
                assert_eq!(val.as_deref(), Some(&42usize));
                Ok(())
            })
        }

        let cmd = Command::builder("Inspect")
            .kind(CommandKind::Prefix)
            .custom_data("meta-v1")
            .invoke_on_edit(true)
            .handler(inspect_handler)
            .build();

        let tracker = Arc::new(EditTracker::for_timespan(Duration::from_secs(60)));
        let fw = crate::Framework::builder()
            .options(FrameworkOptions {
                commands: vec![cmd],
                prefix: Some("!".to_owned()),
                prefix_options: PrefixOptions {
                    additional_prefixes: vec!["?".to_owned()],
                    case_insensitive_commands: true,
                    edit_tracker: Some(Arc::clone(&tracker)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .build();

        let msg = oxcord_model::channel::Message {
            id: oxcord_model::id::MessageId::new(12345),
            channel_id: ChannelId::new(1),
            content: "?INSPECT".to_owned(),
            ..Default::default()
        };
        let res = fw
            .dispatch_full_message(
                &msg,
                Arc::new(Http::new("test-token")),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;
        assert!(matches!(res, Some(Ok(()))));

        let update_entry = tracker.process_message_update(msg.id);
        assert!(update_entry.is_some());
        tracker.set_bot_response(msg.id, oxcord_model::id::MessageId::new(67890));
        assert_eq!(
            tracker.find_bot_response(msg.id),
            Some(oxcord_model::id::MessageId::new(67890))
        );
        assert!(tracker.process_message_delete(msg.id).is_some());
        assert!(tracker.find_bot_response(msg.id).is_none());
    }

    #[test]
    fn build_global_commands_includes_metadata_and_context_menu() {
        use oxcord_model::Permissions;
        use oxcord_model::application::{CommandType, InstallationContext, InteractionContext};

        fn cm_user(
            _ctx: Context<'_>,
            _u: oxcord_model::user::User,
        ) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        let slash = Command::builder("info")
            .description("Info cmd")
            .nsfw_only(true)
            .default_member_permissions(Permissions::MANAGE_MESSAGES)
            .install_context(vec![InstallationContext::Guild, InstallationContext::User])
            .interaction_context(vec![InteractionContext::Guild])
            .name_localization("id", "info-id")
            .description_localization("id", "Perintah info")
            .handler(ok_handler)
            .build();

        let cm = Command::builder("userinfo")
            .context_menu_name("User Info")
            .user_context_menu(cm_user)
            .build();

        let fw = crate::Framework::builder()
            .options(FrameworkOptions { commands: vec![slash, cm], ..Default::default() })
            .build();

        let built = fw.build_global_commands();
        assert_eq!(built.len(), 2);
        let json0 = serde_json::to_value(&built[0]).expect("serialize");
        assert_eq!(json0["nsfw"], true);
        assert_eq!(json0["name_localizations"]["id"], "info-id");
        assert_eq!(json0["description_localizations"]["id"], "Perintah info");
        let json1 = serde_json::to_value(&built[1]).expect("serialize");
        assert_eq!(json1["name"], "User Info");
        assert_eq!(json1["type"], serde_json::to_value(CommandType::User).expect("serialize"));
    }
}
