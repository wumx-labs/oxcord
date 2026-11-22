//! Top-level [`Framework`] builder and options.

use std::collections::HashSet;

use oxcord_model::id::UserId;

use crate::command::Command;

/// Global framework configuration.
#[derive(Default)]
pub struct FrameworkOptions {
    /// All registered commands.
    pub commands: Vec<Command>,
    /// Prefix used for text commands (e.g. `"!"`, `"?"`, `"."`).
    /// `None` disables prefix command support entirely.
    pub prefix: Option<String>,
    /// User IDs considered bot owners (bypass `owner_only` checks).
    pub owners: HashSet<UserId>,
    /// Whether to automatically register slash commands on ready.
    pub auto_register_slash: bool,
    /// Called when a command returns an error.
    #[allow(clippy::type_complexity)]
    pub on_error: Option<Box<dyn Fn(crate::FrameworkError) + Send + Sync>>,
}

impl std::fmt::Debug for FrameworkOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameworkOptions")
            .field("commands", &self.commands.iter().map(|c| &c.name).collect::<Vec<_>>())
            .field("prefix", &self.prefix)
            .field("owners", &self.owners)
            .field("auto_register_slash", &self.auto_register_slash)
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
#[derive(Debug)]
pub struct Framework {
    pub(crate) options: FrameworkOptions,
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

    /// Finalise and return the [`Framework`].
    #[must_use]
    pub fn build(self) -> Framework {
        Framework {
            options: self.options,
        }
    }
}
