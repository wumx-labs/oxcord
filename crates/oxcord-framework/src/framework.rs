//! Top-level [`Framework`] builder and options.

use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use oxcord_builders::{CreateCommand, CreateCommandOption};
use oxcord_model::application::{CommandDataOptionValue, CommandInteraction};
use oxcord_model::id::UserId;
use oxcord_model::user::User;
use oxcord_rest::Http;

use crate::command::{Command, CommandKind};
use crate::context::{FrameworkHttp, InvocationData};
use crate::dispatch::{Dispatcher, Invocation};
use crate::error::FrameworkError;

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
    /// Shared, typed application state every command handler can read back
    /// via `Context::data::<T>()` — the equivalent of poise's generic `Data`
    /// type parameter. Set it with [`FrameworkBuilder::user_data`].
    pub user_data: Option<Arc<dyn Any + Send + Sync>>,
}

impl std::fmt::Debug for FrameworkOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameworkOptions")
            .field("commands", &self.commands.iter().map(|c| &c.name).collect::<Vec<_>>())
            .field("prefix", &self.prefix)
            .field("owners", &self.owners)
            .field("auto_register_slash", &self.auto_register_slash)
            .field("user_data", &self.user_data.is_some())
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
    pub async fn dispatch_message(
        &self,
        author: &User,
        channel_id: oxcord_model::id::ChannelId,
        guild_id: Option<oxcord_model::id::GuildId>,
        content: &str,
        http: Arc<Http>,
    ) -> Option<Result<(), FrameworkError>> {
        let prefix = self.options.prefix.as_deref()?;
        let rest = content.strip_prefix(prefix)?;
        let mut parts = rest.splitn(2, char::is_whitespace);
        let command_name = parts.next().unwrap_or("").trim();
        if command_name.is_empty() {
            return None;
        }
        let args = parts.next().unwrap_or("").trim_start().to_owned();

        let data = InvocationData::Prefix {
            args,
            guild_id,
        };
        let framework_http = Arc::new(FrameworkHttp::for_prefix(http, channel_id));
        let inv = Invocation {
            command_name,
            author,
            data: &data,
            http: framework_http,
            received_at: Instant::now(),
        };
        Some(self.dispatcher.dispatch(inv).await)
    }

    /// Dispatch a gateway slash-command interaction.
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
    ) -> Result<(), FrameworkError> {
        let data = InvocationData::Slash {
            token: command.token.clone(),
            guild_id: command.guild_id,
            options: command.data.options.clone(),
        };
        let framework_http =
            Arc::new(FrameworkHttp::for_slash(http, command.id, command.application_id, command.channel_id));
        let inv = Invocation {
            command_name: &command.data.name,
            author: &command.user,
            data: &data,
            http: framework_http,
            received_at: Instant::now(),
        };
        self.dispatcher.dispatch(inv).await
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
        self.options
            .commands
            .iter()
            .filter(|cmd| !matches!(cmd.kind, CommandKind::Prefix))
            .map(|cmd| {
                let mut builder = CreateCommand::new(&cmd.name).description(&cmd.description);
                if cmd.guild_only == Some(true) {
                    #[allow(deprecated)]
                    {
                        builder = builder.dm_permission(false);
                    }
                }
                let options = cmd
                    .options
                    .iter()
                    .map(|opt| {
                        CreateCommandOption::new(opt.kind, &opt.name, &opt.description)
                            .required(opt.required)
                            .set_autocomplete(cmd.autocomplete.contains_key(&opt.name))
                    })
                    .collect();
                builder.set_options(options)
            })
            .collect()
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
        let commands = self.build_global_commands();
        http.create_global_commands(&commands).await.map_err(|e| FrameworkError::Http(e.to_string()))?;
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
    ) -> Result<Vec<String>, FrameworkError> {
        let Some(focused) = command.data.options.iter().find(|opt| {
            matches!(opt.value, CommandDataOptionValue::Autocomplete { .. })
        }) else {
            return Err(FrameworkError::Other("autocomplete interaction has no focused option".to_owned()));
        };
        let partial = match &focused.value {
            CommandDataOptionValue::Autocomplete {
                value, ..
            } => value.clone(),
            _ => unreachable!("filtered above"),
        };

        let data = InvocationData::Slash {
            token: command.token.clone(),
            guild_id: command.guild_id,
            options: command.data.options.clone(),
        };
        let framework_http =
            Arc::new(FrameworkHttp::for_slash(http, command.id, command.application_id, command.channel_id));
        self.dispatcher
            .dispatch_autocomplete(&command.data.name, &focused.name, partial, &command.user, &data, framework_http)
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
