//! Command definition types.

use std::future::Future;
use std::pin::Pin;

use crate::context::Context;
use crate::error::FrameworkError;

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

/// A registered command and its metadata.
#[derive(Clone, Debug)]
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
    /// Minimum required permissions (slash commands).
    pub required_permissions: Option<oxcord_model::Permissions>,
}

impl Command {
    /// Create a new command builder.
    #[must_use]
    pub fn builder(name: impl Into<String>) -> CommandBuilder {
        CommandBuilder::new(name)
    }
}

/// Fluent builder for [`Command`].
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
}

impl CommandBuilder {
    /// Create a new builder with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CommandKind::Slash,
            ..Default::default()
        }
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

    /// Set the required permissions.
    #[must_use]
    pub fn required_permissions(mut self, perms: oxcord_model::Permissions) -> Self {
        self.required_permissions = Some(perms);
        self
    }

    /// Set the category label.
    #[must_use]
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = CommandCategory(Some(cat.into()));
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
        }
    }
}
