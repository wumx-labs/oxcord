#[cfg(feature = "model")]
use crate::builder::{Builder, CreateCommand};
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::id::CommandId;
use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait CommandExt: Sized {
    #[cfg(feature = "model")]
    /// Create a global [`Command`], overriding an existing one with the same name if it exists.
    ///
    /// When a created [`Command`] is used, the [`InteractionCreate`] event will be emitted.
    ///
    /// **Note**: Global commands may take up to an hour to be updated in the user slash commands
    /// list. If an outdated command data is sent by a user, discord will consider it as an error
    /// and then will instantly update that command.
    ///
    /// As such, it is recommended that guild application commands be used for testing purposes.
    ///
    /// # Examples
    ///
    /// Create a simple ping command:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http: Arc<Http> = unimplemented!();
    /// use oxcord::builder::CreateCommand;
    /// use oxcord::model::application::Command;
    /// use oxcord::model::id::ApplicationId;
    ///
    /// let builder = CreateCommand::new("ping").description("A simple ping command");
    /// let _ = Command::create_global_command(&http, builder).await;
    /// # }
    /// ```
    ///
    /// Create a command that echoes what is inserted:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http: Arc<Http> = unimplemented!();
    /// use oxcord::builder::{CreateCommand, CreateCommandOption as CreateOption};
    /// use oxcord::model::application::{Command, CommandOptionType};
    /// use oxcord::model::id::ApplicationId;
    ///
    /// let builder =
    ///     CreateCommand::new("echo").description("Makes the bot send a message").add_option(
    ///         CreateOption::new(CommandOptionType::String, "message", "The message to send")
    ///             .required(true),
    ///     );
    /// let _ = Command::create_global_command(&http, builder).await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    ///
    /// [`InteractionCreate`]: crate::client::EventHandler::interaction_create
    async fn create_global_command(
        cache_http: impl CacheHttp,
        builder: CreateCommand,
    ) -> Result<Command>;
    #[cfg(feature = "model")]
    /// Override all global application commands.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::create_global_command`].
    async fn set_global_commands(
        http: impl AsRef<Http>,
        commands: Vec<CreateCommand>,
    ) -> Result<Vec<Command>>;
    #[cfg(feature = "model")]
    /// Edit a global command, given its Id.
    ///
    /// # Errors
    ///
    /// See [`CreateCommand::execute`] for a list of possible errors.
    async fn edit_global_command(
        cache_http: impl CacheHttp,
        command_id: CommandId,
        builder: CreateCommand,
    ) -> Result<Command>;
    #[cfg(feature = "model")]
    /// Gets all global commands.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    async fn get_global_commands(http: impl AsRef<Http>) -> Result<Vec<Command>>;
    #[cfg(feature = "model")]
    /// Gets all global commands with localizations.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    async fn get_global_commands_with_localizations(http: impl AsRef<Http>)
    -> Result<Vec<Command>>;
    #[cfg(feature = "model")]
    /// Gets a global command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    async fn get_global_command(http: impl AsRef<Http>, command_id: CommandId) -> Result<Command>;
    #[cfg(feature = "model")]
    /// Deletes a global command by its Id.
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    async fn delete_global_command(http: impl AsRef<Http>, command_id: CommandId) -> Result<()>;
}
impl CommandExt for Command {
    #[cfg(feature = "model")]
    async fn create_global_command(
        cache_http: impl CacheHttp,
        builder: CreateCommand,
    ) -> Result<Command> {
        builder.execute(cache_http, (None, None)).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn set_global_commands(
        http: impl AsRef<Http>,
        commands: Vec<CreateCommand>,
    ) -> Result<Vec<Command>> {
        http.as_ref().create_global_commands(&commands).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn edit_global_command(
        cache_http: impl CacheHttp,
        command_id: CommandId,
        builder: CreateCommand,
    ) -> Result<Command> {
        builder.execute(cache_http, (None, Some(command_id))).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn get_global_commands(http: impl AsRef<Http>) -> Result<Vec<Command>> {
        http.as_ref().get_global_commands().await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn get_global_commands_with_localizations(
        http: impl AsRef<Http>,
    ) -> Result<Vec<Command>> {
        http.as_ref().get_global_commands_with_localizations().await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn get_global_command(http: impl AsRef<Http>, command_id: CommandId) -> Result<Command> {
        http.as_ref().get_global_command(command_id).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn delete_global_command(http: impl AsRef<Http>, command_id: CommandId) -> Result<()> {
        http.as_ref().delete_global_command(command_id).await.map_err(From::from)
    }
}
