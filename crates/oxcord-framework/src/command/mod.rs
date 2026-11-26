//! Command definition types.

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use oxcord_model::application::{CommandOptionType, InstallationContext, InteractionContext};
use oxcord_model::channel::Message;
use oxcord_model::user::User;

use crate::context::Context;
use crate::cooldown::CooldownConfig;
use crate::error::FrameworkError;
use crate::framework::ErrorHook;

// ---------------------------------------------------------------------------
// Handler wrappers — resolve the HRTB `for<'a> Fn(Context<'a>)` issue on
// stable Rust by storing plain fn pointers instead of boxed closures.
// ---------------------------------------------------------------------------

/// Raw async function type for command handlers.
///
/// Write your command as a plain `async fn`, then wrap it:
/// ```rust,ignore
/// pub fn my_cmd_fn<'a>(ctx: Context<'a>) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send + 'a>> {
///     Box::pin(my_cmd(ctx))
/// }
/// ```
/// The `#[oxcord_macros::command]` attribute generates this boilerplate for you.
pub type CommandFn =
    for<'a> fn(
        Context<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send + 'a>>;

/// A registered command handler (wraps a [`CommandFn`] pointer).
#[derive(Clone, Copy)]
pub struct CommandHandler(pub CommandFn);

impl CommandHandler {
    /// Invoke the handler with the given context.
    pub fn call<'a>(
        &self,
        ctx: Context<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send + 'a>> {
        (self.0)(ctx)
    }
}

impl std::fmt::Debug for CommandHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CommandHandler(<fn>)")
    }
}

/// Raw async function type for autocomplete handlers.
pub type AutocompleteFn = for<'a> fn(
    Context<'a>,
    String,
) -> Pin<
    Box<dyn Future<Output = Result<Vec<String>, FrameworkError>> + Send + 'a>,
>;

/// Tipe fungsi async buat check per-command atau global `command_check`.
pub type CheckFn =
    for<'a> fn(
        Context<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<bool, FrameworkError>> + Send + 'a>>;

/// Tipe fungsi async buat context-menu command bertipe User.
pub type UserContextMenuFn =
    for<'a> fn(
        Context<'a>,
        User,
    ) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send + 'a>>;

/// Tipe fungsi async buat context-menu command bertipe Message.
pub type MessageContextMenuFn =
    for<'a> fn(
        Context<'a>,
        Message,
    ) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send + 'a>>;

/// Aksi yang dijalankan saat context-menu command (klik kanan user/pesan) dipanggil.
#[derive(Clone, Copy)]
pub enum ContextMenuAction {
    /// Context-menu pada User.
    User(UserContextMenuFn),
    /// Context-menu pada Message.
    Message(MessageContextMenuFn),
}

impl std::fmt::Debug for ContextMenuAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User(_) => f.write_str("ContextMenuAction::User(<fn>)"),
            Self::Message(_) => f.write_str("ContextMenuAction::Message(<fn>)"),
        }
    }
}

/// A registered autocomplete handler.
#[derive(Clone, Copy)]
pub struct AutocompleteHandler(pub AutocompleteFn);

impl AutocompleteHandler {
    /// Invoke the handler.
    pub fn call<'a>(
        &self,
        ctx: Context<'a>,
        partial: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, FrameworkError>> + Send + 'a>> {
        (self.0)(ctx, partial)
    }
}

impl std::fmt::Debug for AutocompleteHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AutocompleteHandler(<fn>)")
    }
}

/// Whether a command supports slash, prefix, or both invocation styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommandKind {
    /// Slash (application) command only.
    #[default]
    Slash,
    /// Prefix (text) command only.
    Prefix,
    /// Both slash and prefix.
    Both,
}

/// Optional grouping label for the help command and docs.
#[derive(Debug, Clone, Default)]
pub struct CommandCategory(pub Option<String>);

/// A slash-command option's registration metadata — what gets sent to
/// Discord when (re-)registering commands. Generated automatically by the
/// `#[oxcord_macros::command]` macro from each extra function parameter.
#[derive(Debug, Clone)]
pub struct CommandOptionSpec {
    /// The option's name, exactly as the parameter is named.
    pub name: String,
    /// One-line description shown in Discord's command picker.
    pub description: String,
    /// The Discord option type (string, integer, user, channel, ...).
    pub kind: CommandOptionType,
    /// Whether Discord should require the user to supply this option.
    /// `false` for `Option<T>` parameters.
    pub required: bool,
}

/// A registered command and its metadata.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct Command {
    /// The primary name used to invoke the command.
    pub name: String,
    /// One-line description shown in Discord's command picker.
    pub description: String,
    /// Whether this is slash, prefix, or both.
    pub kind: CommandKind,
    /// Aliases used for prefix invocation only.
    pub aliases: Vec<String>,
    /// Sub-commands. If non-empty, this is a command group.
    pub subcommands: Vec<Command>,
    /// The async function that runs when the command is invoked.
    pub handler: Option<CommandHandler>,
    /// Per-option autocomplete handlers, keyed by option name.
    pub autocomplete: std::collections::HashMap<String, AutocompleteHandler>,
    /// Optional grouping for the help page.
    pub category: CommandCategory,
    /// Whether guild-only. `None` means Discord's own default.
    pub guild_only: Option<bool>,
    /// Whether owner-only (checked by [`FrameworkOptions::owners`]).
    pub owner_only: bool,
    /// Permissions the invoking user must have. Checked on slash invocations
    /// using the permissions Discord sends directly on the interaction (no
    /// cache/REST lookup needed); **not yet enforced for prefix invocations**
    /// (would need member-permission computation via the cache, not done
    /// yet — see `oxcord-framework`'s README/changelog).
    pub required_permissions: Option<oxcord_model::Permissions>,
    /// Permissions the bot itself must have in the invoking channel. Same
    /// slash-only enforcement caveat as [`Self::required_permissions`].
    pub required_bot_permissions: Option<oxcord_model::Permissions>,
    /// The slash-command options to register with Discord, generated from
    /// this command's extra function parameters.
    pub options: Vec<CommandOptionSpec>,
    /// Automatic per-command cooldown, applied without the caller needing to
    /// call [`crate::dispatch::Dispatcher::register_cooldown`] manually.
    pub cooldown: Option<CooldownConfig>,
    /// Kalau `true`, command cuma boleh jalan di DM (kebalikan `guild_only`).
    pub dm_only: bool,
    /// Kalau `true`, command cuma boleh jalan di channel NSFW.
    pub nsfw_only: bool,
    /// Daftar check per-command; semuanya harus return `Ok(true)` biar command jalan.
    pub checks: Vec<CheckFn>,
    /// Hook error khusus buat command ini aja (override `FrameworkOptions::on_error`).
    pub on_error: Option<ErrorHook>,
    /// Data custom bebas per-command (misalnya metadata tambahan buat help command).
    pub custom_data: Option<Arc<dyn Any + Send + Sync>>,
    /// Apakah pesan command prefix yang diedit user bakal memicu eksekusi ulang command.
    pub invoke_on_edit: bool,
    /// Apakah balasan bot ikut dihapus kalau pesan command prefix asalnya dihapus.
    pub track_deletion: bool,
    /// Apakah eksekusi ulang (karena edit) mengedit balasan bot sebelumnya alih-alih kirim baru.
    pub reuse_response: bool,
    /// Apakah bot otomatis kirim indikator typing saat command prefix mulai diproses.
    pub broadcast_typing: bool,
    /// Default `ephemeral` untuk semua reply dari command ini kalau `Reply::ephemeral()` nggak diset manual.
    pub ephemeral: bool,
    /// Konteks instalasi Discord (`Guild` / `User`) saat registrasi slash command.
    pub install_context: Option<Vec<InstallationContext>>,
    /// Konteks interaksi Discord (`Guild` / `BotDm` / `PrivateChannel`) saat registrasi slash command.
    pub interaction_context: Option<Vec<InteractionContext>>,
    /// Default member permissions yang ditampilkan di UI Discord (bukan cek internal framework).
    pub default_member_permissions: Option<oxcord_model::Permissions>,
    /// Kalau `true`, sembunyikan command ini dari output help generator.
    pub hide_in_help: bool,
    /// Teks bantuan panjang khusus command ini buat help generator.
    pub help_text: Option<String>,
    /// Lokalisasi nama command (`(locale, localized_name)`) buat registrasi slash command.
    pub name_localizations: Vec<(String, String)>,
    /// Lokalisasi deskripsi command (`(locale, localized_description)`) buat registrasi slash command.
    pub description_localizations: Vec<(String, String)>,
    /// Kalau `true` dan command ini punya subcommands, pemanggilan tanpa subcommand bakal error `SubcommandRequired`.
    pub subcommand_required: bool,
    /// Nama tampilan untuk context-menu command (kalau beda dari `name`).
    pub context_menu_name: Option<String>,
    /// Handler context-menu command (klik kanan User / Message).
    pub context_menu_action: Option<ContextMenuAction>,
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("kind", &self.kind)
            .field("aliases", &self.aliases)
            .field("subcommands", &self.subcommands)
            .field("guild_only", &self.guild_only)
            .field("dm_only", &self.dm_only)
            .field("nsfw_only", &self.nsfw_only)
            .field("owner_only", &self.owner_only)
            .field("subcommand_required", &self.subcommand_required)
            .field("on_error", &self.on_error.is_some())
            .field("custom_data", &self.custom_data.is_some())
            .field("context_menu_action", &self.context_menu_action)
            .finish_non_exhaustive()
    }
}

impl Command {
    /// Create a new command builder.
    #[must_use]
    pub fn builder(name: impl Into<String>) -> CommandBuilder {
        CommandBuilder::new(name)
    }
}

/// Fluent builder for [`Command`].
#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct CommandBuilder {
    name: String,
    description: String,
    kind: CommandKind,
    aliases: Vec<String>,
    subcommands: Vec<Command>,
    handler: Option<CommandHandler>,
    autocomplete: std::collections::HashMap<String, AutocompleteHandler>,
    category: CommandCategory,
    guild_only: Option<bool>,
    owner_only: bool,
    required_permissions: Option<oxcord_model::Permissions>,
    required_bot_permissions: Option<oxcord_model::Permissions>,
    options: Vec<CommandOptionSpec>,
    cooldown: Option<CooldownConfig>,
    dm_only: bool,
    nsfw_only: bool,
    checks: Vec<CheckFn>,
    on_error: Option<ErrorHook>,
    custom_data: Option<Arc<dyn Any + Send + Sync>>,
    invoke_on_edit: bool,
    track_deletion: bool,
    reuse_response: bool,
    broadcast_typing: bool,
    ephemeral: bool,
    install_context: Option<Vec<InstallationContext>>,
    interaction_context: Option<Vec<InteractionContext>>,
    default_member_permissions: Option<oxcord_model::Permissions>,
    hide_in_help: bool,
    help_text: Option<String>,
    name_localizations: Vec<(String, String)>,
    description_localizations: Vec<(String, String)>,
    subcommand_required: bool,
    context_menu_name: Option<String>,
    context_menu_action: Option<ContextMenuAction>,
}

impl CommandBuilder {
    /// Create a new builder with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), kind: CommandKind::Slash, ..Default::default() }
    }

    /// Set the command description.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set the command kind (slash/prefix/both).
    #[must_use]
    pub fn kind(mut self, kind: CommandKind) -> Self {
        self.kind = kind;
        self
    }

    /// Add prefix aliases.
    #[must_use]
    pub fn aliases(mut self, aliases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.aliases = aliases.into_iter().map(Into::into).collect();
        self
    }

    /// Set the handler using a plain [`CommandFn`] function pointer.
    ///
    /// This is used by the `#[oxcord_macros::command]` attribute to register
    /// the generated wrapper function. When building a command manually, prefer
    /// using a function that matches the [`CommandFn`] signature.
    #[must_use]
    pub fn handler(mut self, f: CommandFn) -> Self {
        self.handler = Some(CommandHandler(f));
        self
    }

    /// Add an autocomplete handler for a named option.
    #[must_use]
    pub fn autocomplete_for(mut self, option_name: impl Into<String>, f: AutocompleteFn) -> Self {
        self.autocomplete.insert(option_name.into(), AutocompleteHandler(f));
        self
    }

    /// Add a subcommand.
    #[must_use]
    pub fn subcommand(mut self, cmd: Command) -> Self {
        self.subcommands.push(cmd);
        self
    }

    /// Mark as guild-only.
    #[must_use]
    pub fn guild_only(mut self, v: bool) -> Self {
        self.guild_only = Some(v);
        self
    }

    /// Mark as owner-only.
    #[must_use]
    pub fn owner_only(mut self, v: bool) -> Self {
        self.owner_only = v;
        self
    }

    /// Set the permissions the invoking user must have.
    #[must_use]
    pub fn required_permissions(mut self, perms: oxcord_model::Permissions) -> Self {
        self.required_permissions = Some(perms);
        self
    }

    /// Set the permissions the bot itself must have in the invoking channel.
    #[must_use]
    pub fn required_bot_permissions(mut self, perms: oxcord_model::Permissions) -> Self {
        self.required_bot_permissions = Some(perms);
        self
    }

    /// Set the category label.
    #[must_use]
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = CommandCategory(Some(cat.into()));
        self
    }

    /// Add a slash-command option to register with Discord.
    #[must_use]
    pub fn option(mut self, option: CommandOptionSpec) -> Self {
        self.options.push(option);
        self
    }

    /// Set every slash-command option to register with Discord at once.
    #[must_use]
    pub fn options(mut self, options: Vec<CommandOptionSpec>) -> Self {
        self.options = options;
        self
    }

    /// Apply an automatic per-command cooldown.
    #[must_use]
    pub fn cooldown(mut self, cooldown: CooldownConfig) -> Self {
        self.cooldown = Some(cooldown);
        self
    }

    /// Tandai command sebagai DM-only (cuma boleh jalan di DM).
    #[must_use]
    pub fn dm_only(mut self, v: bool) -> Self {
        self.dm_only = v;
        self
    }

    /// Tandai command sebagai NSFW-only.
    #[must_use]
    pub fn nsfw_only(mut self, v: bool) -> Self {
        self.nsfw_only = v;
        self
    }

    /// Tambah satu check per-command (bisa dipanggil berkali-kali).
    #[must_use]
    pub fn check(mut self, f: CheckFn) -> Self {
        self.checks.push(f);
        self
    }

    /// Pasang error hook khusus buat command ini (override `FrameworkOptions::on_error`).
    #[must_use]
    pub fn on_error<F>(mut self, hook: F) -> Self
    where
        F: Fn(
                FrameworkError,
                crate::context::ErrorContext,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        self.on_error = Some(Arc::new(hook));
        self
    }

    /// Simpan custom data bebas per-command (bisa dibaca lewat `Context::command_custom_data`).
    #[must_use]
    pub fn custom_data<T: Any + Send + Sync + 'static>(mut self, data: T) -> Self {
        self.custom_data = Some(Arc::new(data));
        self
    }

    /// Aktifkan/nonaktifkan eksekusi ulang saat pesan prefix diedit.
    #[must_use]
    pub fn invoke_on_edit(mut self, v: bool) -> Self {
        self.invoke_on_edit = v;
        self
    }

    /// Aktifkan/nonaktifkan penghapusan balasan bot saat pesan prefix asalnya dihapus.
    #[must_use]
    pub fn track_deletion(mut self, v: bool) -> Self {
        self.track_deletion = v;
        self
    }

    /// Aktifkan/nonaktifkan reuse response (edit balasan lama saat command di-rerun karena edit).
    #[must_use]
    pub fn reuse_response(mut self, v: bool) -> Self {
        self.reuse_response = v;
        self
    }

    /// Aktifkan/nonaktifkan indikator typing otomatis saat command prefix diproses.
    #[must_use]
    pub fn broadcast_typing(mut self, v: bool) -> Self {
        self.broadcast_typing = v;
        self
    }

    /// Set default `ephemeral` untuk semua reply dari command ini.
    #[must_use]
    pub fn ephemeral(mut self, v: bool) -> Self {
        self.ephemeral = v;
        self
    }

    /// Set daftar `InstallationContext` untuk registrasi slash command.
    #[must_use]
    pub fn install_context(mut self, contexts: Vec<InstallationContext>) -> Self {
        self.install_context = Some(contexts);
        self
    }

    /// Set daftar `InteractionContext` untuk registrasi slash command.
    #[must_use]
    pub fn interaction_context(mut self, contexts: Vec<InteractionContext>) -> Self {
        self.interaction_context = Some(contexts);
        self
    }

    /// Set `default_member_permissions` untuk tampilan UI Discord (bukan cek internal framework).
    #[must_use]
    pub fn default_member_permissions(mut self, perms: oxcord_model::Permissions) -> Self {
        self.default_member_permissions = Some(perms);
        self
    }

    /// Sembunyikan atau tampilkan command ini di help generator.
    #[must_use]
    pub fn hide_in_help(mut self, v: bool) -> Self {
        self.hide_in_help = v;
        self
    }

    /// Set teks bantuan panjang untuk help generator.
    #[must_use]
    pub fn help_text(mut self, text: impl Into<String>) -> Self {
        self.help_text = Some(text.into());
        self
    }

    /// Tambah satu lokalisasi nama command.
    #[must_use]
    pub fn name_localization(mut self, locale: impl Into<String>, name: impl Into<String>) -> Self {
        self.name_localizations.push((locale.into(), name.into()));
        self
    }

    /// Set semua lokalisasi nama command sekaligus.
    #[must_use]
    pub fn name_localizations(
        mut self,
        locs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.name_localizations = locs.into_iter().map(|(l, n)| (l.into(), n.into())).collect();
        self
    }

    /// Tambah satu lokalisasi deskripsi command.
    #[must_use]
    pub fn description_localization(
        mut self,
        locale: impl Into<String>,
        desc: impl Into<String>,
    ) -> Self {
        self.description_localizations.push((locale.into(), desc.into()));
        self
    }

    /// Set semua lokalisasi deskripsi command sekaligus.
    #[must_use]
    pub fn description_localizations(
        mut self,
        locs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.description_localizations =
            locs.into_iter().map(|(l, d)| (l.into(), d.into())).collect();
        self
    }

    /// Wajibkan pemilihan subcommand jika command ini punya subcommands.
    #[must_use]
    pub fn subcommand_required(mut self, v: bool) -> Self {
        self.subcommand_required = v;
        self
    }

    /// Set nama tampilan context-menu command.
    #[must_use]
    pub fn context_menu_name(mut self, name: impl Into<String>) -> Self {
        self.context_menu_name = Some(name.into());
        self
    }

    /// Pasang handler context-menu untuk klik kanan User.
    #[must_use]
    pub fn user_context_menu(mut self, f: UserContextMenuFn) -> Self {
        self.context_menu_action = Some(ContextMenuAction::User(f));
        self
    }

    /// Pasang handler context-menu untuk klik kanan Message.
    #[must_use]
    pub fn message_context_menu(mut self, f: MessageContextMenuFn) -> Self {
        self.context_menu_action = Some(ContextMenuAction::Message(f));
        self
    }

    /// Build the [`Command`].
    #[must_use]
    pub fn build(self) -> Command {
        Command {
            name: self.name,
            description: self.description,
            kind: self.kind,
            aliases: self.aliases,
            subcommands: self.subcommands,
            handler: self.handler,
            autocomplete: self.autocomplete,
            category: self.category,
            guild_only: self.guild_only,
            owner_only: self.owner_only,
            required_permissions: self.required_permissions,
            required_bot_permissions: self.required_bot_permissions,
            options: self.options,
            cooldown: self.cooldown,
            dm_only: self.dm_only,
            nsfw_only: self.nsfw_only,
            checks: self.checks,
            on_error: self.on_error,
            custom_data: self.custom_data,
            invoke_on_edit: self.invoke_on_edit,
            track_deletion: self.track_deletion,
            reuse_response: self.reuse_response,
            broadcast_typing: self.broadcast_typing,
            ephemeral: self.ephemeral,
            install_context: self.install_context,
            interaction_context: self.interaction_context,
            default_member_permissions: self.default_member_permissions,
            hide_in_help: self.hide_in_help,
            help_text: self.help_text,
            name_localizations: self.name_localizations,
            description_localizations: self.description_localizations,
            subcommand_required: self.subcommand_required,
            context_menu_name: self.context_menu_name,
            context_menu_action: self.context_menu_action,
        }
    }
}
