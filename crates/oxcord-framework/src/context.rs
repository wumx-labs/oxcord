//! The unified command context, covering both slash and prefix invocations.

use std::any::Any;
use std::sync::Arc;
use std::time::Instant;

use oxcord_builders::{
    Builder,
    CreateEmbed,
    CreateInteractionResponse,
    CreateInteractionResponseMessage,
    CreateMessage,
    EditInteractionResponse,
    EditMessage,
};
use oxcord_model::application::{CommandDataOption, CommandDataOptionValue, CommandOptionType};
use oxcord_model::id::{ApplicationId, ChannelId, GuildId, InteractionId, MessageId, RoleId, UserId};
use oxcord_model::user::User;
use oxcord_rest::Http;

use crate::error::FrameworkError;

/// Raw invocation data for the current command.
#[derive(Debug, Clone)]
pub enum InvocationData {
    /// Slash (application) command interaction token and data.
    Slash {
        /// The interaction token, used to reply.
        token: String,
        /// Guild the interaction was triggered in, if any.
        guild_id: Option<GuildId>,
        /// The options Discord sent with the command invocation.
        options: Vec<CommandDataOption>,
    },
    /// Prefix (text) command message content.
    Prefix {
        /// The raw message content after the command name.
        args: String,
        /// Guild the message was sent in, if any.
        guild_id: Option<GuildId>,
    },
}

/// A lightweight HTTP/interaction client handle passed into command handlers.
///
/// Wraps a real `Arc<Http>` so [`Context::say`] (and friends) can actually
/// reach Discord, while keeping the handler-facing API free of the full
/// `oxcord` facade crate.
pub struct FrameworkHttp {
    /// Interaction id, for slash commands only (needed to build a response).
    pub(crate) interaction_id: Option<InteractionId>,
    /// Application ID, needed to follow up interactions.
    #[allow(dead_code)]
    pub(crate) application_id: Option<ApplicationId>,
    /// Channel the invocation happened in.
    pub(crate) channel_id: ChannelId,
    /// The real HTTP handle used to talk to Discord.
    pub(crate) http: Arc<Http>,
}

impl FrameworkHttp {
    /// Build a handle for a slash (application) command invocation.
    #[must_use]
    pub fn for_slash(
        http: Arc<Http>,
        interaction_id: InteractionId,
        application_id: ApplicationId,
        channel_id: ChannelId,
    ) -> Self {
        Self {
            interaction_id: Some(interaction_id),
            application_id: Some(application_id),
            channel_id,
            http,
        }
    }

    /// Build a handle for a prefix (text) command invocation.
    #[must_use]
    pub fn for_prefix(http: Arc<Http>, channel_id: ChannelId) -> Self {
        Self {
            interaction_id: None,
            application_id: None,
            channel_id,
            http,
        }
    }
}

impl std::fmt::Debug for FrameworkHttp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameworkHttp")
            .field("interaction_id", &self.interaction_id)
            .field("channel_id", &self.channel_id)
            .finish_non_exhaustive()
    }
}

/// A value extracted from a command's arguments, regardless of whether the
/// command was invoked as a slash command (typed option) or a prefix command
/// (whitespace-separated token).
///
/// Implement this for your own types to accept them as command parameters.
/// Built-in implementations cover `String`, `bool`, `i64`, `f64`, `UserId`,
/// `ChannelId` and `RoleId`.
pub trait FrameworkArgument: Sized {
    /// Convert a slash command's resolved option value.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Other`] if the value is the wrong kind.
    fn from_option(value: &CommandDataOptionValue) -> Result<Self, FrameworkError>;

    /// Convert a single whitespace-separated token from a prefix invocation.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Other`] if the token cannot be parsed.
    fn from_token(token: &str) -> Result<Self, FrameworkError>;

    /// The Discord slash-command option type to register for a parameter of
    /// this Rust type. Used by the `#[oxcord_macros::command]` macro to
    /// build the command's option list for registration.
    fn option_type() -> CommandOptionType;
}

impl FrameworkArgument for String {
    fn from_option(value: &CommandDataOptionValue) -> Result<Self, FrameworkError> {
        match value {
            CommandDataOptionValue::String(s) => Ok(s.clone()),
            other => Err(FrameworkError::Other(format!("expected String, got {:?}", other.kind()))),
        }
    }

    fn from_token(token: &str) -> Result<Self, FrameworkError> {
        Ok(token.to_owned())
    }

    fn option_type() -> CommandOptionType {
        CommandOptionType::String
    }
}

impl FrameworkArgument for bool {
    fn from_option(value: &CommandDataOptionValue) -> Result<Self, FrameworkError> {
        match value {
            CommandDataOptionValue::Boolean(b) => Ok(*b),
            other => Err(FrameworkError::Other(format!("expected Boolean, got {:?}", other.kind()))),
        }
    }

    fn from_token(token: &str) -> Result<Self, FrameworkError> {
        match token {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            other => Err(FrameworkError::Other(format!("'{other}' is not a valid bool"))),
        }
    }

    fn option_type() -> CommandOptionType {
        CommandOptionType::Boolean
    }
}

impl FrameworkArgument for i64 {
    fn from_option(value: &CommandDataOptionValue) -> Result<Self, FrameworkError> {
        match value {
            CommandDataOptionValue::Integer(i) => Ok(*i),
            other => Err(FrameworkError::Other(format!("expected Integer, got {:?}", other.kind()))),
        }
    }

    fn from_token(token: &str) -> Result<Self, FrameworkError> {
        token.parse::<Self>().map_err(|_| FrameworkError::Other(format!("'{token}' is not a valid integer")))
    }

    fn option_type() -> CommandOptionType {
        CommandOptionType::Integer
    }
}

impl FrameworkArgument for f64 {
    fn from_option(value: &CommandDataOptionValue) -> Result<Self, FrameworkError> {
        match value {
            CommandDataOptionValue::Number(n) => Ok(*n),
            CommandDataOptionValue::Integer(i) => Ok(*i as Self),
            other => Err(FrameworkError::Other(format!("expected Number, got {:?}", other.kind()))),
        }
    }

    fn from_token(token: &str) -> Result<Self, FrameworkError> {
        token.parse::<Self>().map_err(|_| FrameworkError::Other(format!("'{token}' is not a valid number")))
    }

    fn option_type() -> CommandOptionType {
        CommandOptionType::Number
    }
}

impl FrameworkArgument for UserId {
    fn from_option(value: &CommandDataOptionValue) -> Result<Self, FrameworkError> {
        match value {
            CommandDataOptionValue::User(id) => Ok(*id),
            other => Err(FrameworkError::Other(format!("expected User, got {:?}", other.kind()))),
        }
    }

    fn from_token(token: &str) -> Result<Self, FrameworkError> {
        let digits = token.trim_start_matches("<@!").trim_start_matches("<@").trim_end_matches('>');
        digits
            .parse::<u64>()
            .map(Self::new)
            .map_err(|_| FrameworkError::Other(format!("'{token}' is not a valid user mention or id")))
    }

    fn option_type() -> CommandOptionType {
        CommandOptionType::User
    }
}

impl FrameworkArgument for ChannelId {
    fn from_option(value: &CommandDataOptionValue) -> Result<Self, FrameworkError> {
        match value {
            CommandDataOptionValue::Channel(id) => Ok(*id),
            other => Err(FrameworkError::Other(format!("expected Channel, got {:?}", other.kind()))),
        }
    }

    fn from_token(token: &str) -> Result<Self, FrameworkError> {
        let digits = token.trim_start_matches("<#").trim_end_matches('>');
        digits
            .parse::<u64>()
            .map(Self::new)
            .map_err(|_| FrameworkError::Other(format!("'{token}' is not a valid channel mention or id")))
    }

    fn option_type() -> CommandOptionType {
        CommandOptionType::Channel
    }
}

impl FrameworkArgument for RoleId {
    fn from_option(value: &CommandDataOptionValue) -> Result<Self, FrameworkError> {
        match value {
            CommandDataOptionValue::Role(id) => Ok(*id),
            other => Err(FrameworkError::Other(format!("expected Role, got {:?}", other.kind()))),
        }
    }

    fn from_token(token: &str) -> Result<Self, FrameworkError> {
        let digits = token.trim_start_matches("<@&").trim_end_matches('>');
        digits
            .parse::<u64>()
            .map(Self::new)
            .map_err(|_| FrameworkError::Other(format!("'{token}' is not a valid role mention or id")))
    }

    fn option_type() -> CommandOptionType {
        CommandOptionType::Role
    }
}

/// Something that can be turned into a [`Reply`] — plain text, or a
/// pre-built [`Reply`] itself (for embeds, or text + embed together).
pub struct Reply {
    content: Option<String>,
    embeds: Vec<CreateEmbed>,
}

impl Reply {
    /// An empty reply (add content/embeds with the builder methods).
    #[must_use]
    pub fn new() -> Self {
        Self {
            content: None,
            embeds: Vec::new(),
        }
    }

    /// Set the reply's text content.
    #[must_use]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Add a single embed to the reply.
    #[must_use]
    pub fn embed(mut self, embed: CreateEmbed) -> Self {
        self.embeds.push(embed);
        self
    }

    /// Set the reply's embeds, replacing any already added.
    #[must_use]
    pub fn embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = embeds;
        self
    }
}

impl Default for Reply {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for Reply {
    fn from(content: &str) -> Self {
        Self::new().content(content)
    }
}

impl From<String> for Reply {
    fn from(content: String) -> Self {
        Self::new().content(content)
    }
}

/// Where a previously-sent [`Reply`] can be edited.
enum ReplyTarget {
    Slash { token: String },
    Prefix { channel_id: ChannelId, message_id: MessageId },
}

/// A handle to a message [`Context::send`] already sent, so it can be
/// edited afterwards (mirrors poise's `ReplyHandle` / `message.edit(...)`).
pub struct ReplyHandle {
    http: Arc<FrameworkHttp>,
    target: ReplyTarget,
}

impl ReplyHandle {
    /// Edit the message this handle refers to.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Http`] if the Discord API call fails.
    pub async fn edit(&self, reply: impl Into<Reply>) -> Result<(), FrameworkError> {
        let reply = reply.into();
        match &self.target {
            ReplyTarget::Slash {
                token,
            } => {
                let mut builder = EditInteractionResponse::new();
                if let Some(content) = reply.content {
                    builder = builder.content(content);
                }
                if !reply.embeds.is_empty() {
                    builder = builder.embeds(reply.embeds);
                }
                builder
                    .execute(&*self.http.http, token.as_str())
                    .await
                    .map(|_| ())
                    .map_err(|e| FrameworkError::Http(e.to_string()))
            },
            ReplyTarget::Prefix {
                channel_id,
                message_id,
            } => {
                let mut builder = EditMessage::new();
                if let Some(content) = reply.content {
                    builder = builder.content(content);
                }
                if !reply.embeds.is_empty() {
                    builder = builder.embeds(reply.embeds);
                }
                builder
                    .execute(&*self.http.http, (*channel_id, *message_id, None))
                    .await
                    .map(|_| ())
                    .map_err(|e| FrameworkError::Http(e.to_string()))
            },
        }
    }
}

/// The unified context passed to every command handler.
///
/// Regardless of whether the command was invoked as a slash command or a prefix
/// message, all handlers receive a `Context<'_>`. Methods like [`Self::say`]
/// automatically pick the right Discord API path.
#[derive(Clone)]
pub struct Context<'a> {
    /// The user who invoked the command.
    pub author: &'a User,
    /// Channel the command was invoked in.
    pub channel_id: ChannelId,
    /// Guild the command was invoked in, if any.
    pub guild_id: Option<GuildId>,
    /// Raw invocation data (slash token or prefix args).
    pub invocation: &'a InvocationData,
    /// A lightweight handle used by helper methods.
    pub(crate) http: Arc<FrameworkHttp>,
    /// The framework-wide shared user data, if any was configured via
    /// `FrameworkOptions::user_data`/`FrameworkBuilder::user_data`.
    pub(crate) data: Option<Arc<dyn Any + Send + Sync>>,
    /// When this invocation was received by the dispatcher (used to measure
    /// bot-side processing latency, e.g. for a `ping` command).
    pub(crate) received_at: Instant,
}

impl Context<'_> {
    /// Reply to the user with a plain text message.
    ///
    /// For slash commands this sends an interaction response; for prefix
    /// commands it sends a regular channel message.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Http`] if the Discord API call fails.
    pub async fn say(&self, content: impl Into<String>) -> Result<(), FrameworkError> {
        self.send(Reply::new().content(content.into())).await.map(|_| ())
    }

    /// Send a reply (text and/or embeds), returning a [`ReplyHandle`] that
    /// can later [`ReplyHandle::edit`] the message — the same send-then-edit
    /// pattern poise's `ctx.send(...)` / `handle.edit(...)` offers.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Http`] if the Discord API call fails.
    pub async fn send(&self, reply: impl Into<Reply>) -> Result<ReplyHandle, FrameworkError> {
        let reply = reply.into();
        match self.invocation {
            InvocationData::Slash {
                token, ..
            } => {
                let interaction_id = self.http.interaction_id.ok_or_else(|| {
                    FrameworkError::Other("slash invocation missing interaction id".to_owned())
                })?;
                let mut message = CreateInteractionResponseMessage::new();
                if let Some(content) = reply.content {
                    message = message.content(content);
                }
                if !reply.embeds.is_empty() {
                    message = message.embeds(reply.embeds);
                }
                CreateInteractionResponse::Message(message)
                    .execute(&*self.http.http, (interaction_id, token.as_str()))
                    .await
                    .map_err(|e| FrameworkError::Http(e.to_string()))?;
                Ok(ReplyHandle {
                    http: Arc::clone(&self.http),
                    target: ReplyTarget::Slash {
                        token: token.clone(),
                    },
                })
            },
            InvocationData::Prefix {
                ..
            } => {
                let mut message = CreateMessage::new();
                if let Some(content) = reply.content {
                    message = message.content(content);
                }
                if !reply.embeds.is_empty() {
                    message = message.embeds(reply.embeds);
                }
                let sent = message
                    .execute(&*self.http.http, (self.channel_id, self.guild_id))
                    .await
                    .map_err(|e| FrameworkError::Http(e.to_string()))?;
                Ok(ReplyHandle {
                    http: Arc::clone(&self.http),
                    target: ReplyTarget::Prefix {
                        channel_id: self.channel_id,
                        message_id: sent.id,
                    },
                })
            },
        }
    }

    /// Access the framework-wide shared user data, downcast to `T`.
    ///
    /// Mirrors poise's `ctx.data()` — configure the value once via
    /// [`crate::FrameworkBuilder::user_data`]/[`crate::FrameworkOptions::user_data`]
    /// and every command handler can read it here.
    ///
    /// # Panics
    ///
    /// Panics if no user data was configured, or if it was configured as a
    /// type other than `T`.
    #[must_use]
    pub fn data<T: Any + Send + Sync>(&self) -> &T {
        self.data
            .as_ref()
            .expect("no user data configured (see FrameworkOptions::user_data)")
            .downcast_ref::<T>()
            .expect("Context::data::<T>() called with a type that doesn't match the configured user data")
    }

    /// When this invocation was received by the dispatcher, before any
    /// pre-flight checks (cooldown, permissions) or handler work ran.
    ///
    /// Subtracting this from `Instant::now()` right before replying gives
    /// the bot's own processing latency (the `bot_ms` a `ping` command
    /// typically reports).
    #[must_use]
    pub fn received_at(&self) -> Instant {
        self.received_at
    }

    /// Return the invoking user's ID.
    #[must_use]
    pub fn author_id(&self) -> UserId {
        self.author.id
    }

    /// Look up a named slash-command option and convert it to `T`.
    ///
    /// Returns `Ok(None)` if the option was not supplied (it was optional and
    /// omitted). Prefix invocations carry no named options — use
    /// [`Self::arg`] for those instead.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Other`] if the option was supplied with a
    /// value of the wrong kind.
    pub fn option<T: FrameworkArgument>(&self, name: &str) -> Result<Option<T>, FrameworkError> {
        match self.invocation {
            InvocationData::Slash {
                options, ..
            } => match options.iter().find(|o| o.name == name) {
                Some(opt) => T::from_option(&opt.value).map(Some),
                None => Ok(None),
            },
            InvocationData::Prefix {
                ..
            } => Ok(None),
        }
    }

    /// The raw text after the command name, for prefix invocations.
    ///
    /// Returns an empty string for slash invocations (use [`Self::option`]
    /// instead).
    #[must_use]
    pub fn raw_args(&self) -> &str {
        match self.invocation {
            InvocationData::Prefix {
                args, ..
            } => args.as_str(),
            InvocationData::Slash {
                ..
            } => "",
        }
    }

    /// The `n`th whitespace-separated token of a prefix invocation's
    /// arguments, converted to `T`. Returns `Ok(None)` if there are fewer
    /// than `n + 1` tokens, or for slash invocations.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Other`] if the token cannot be parsed as `T`.
    pub fn arg<T: FrameworkArgument>(&self, index: usize) -> Result<Option<T>, FrameworkError> {
        match self.invocation {
            InvocationData::Prefix {
                args, ..
            } => match args.split_whitespace().nth(index) {
                Some(token) => T::from_token(token).map(Some),
                None => Ok(None),
            },
            InvocationData::Slash {
                ..
            } => Ok(None),
        }
    }

    /// Extract an argument regardless of invocation kind: a named slash
    /// option for slash invocations, or the `index`th whitespace-separated
    /// token for prefix invocations.
    ///
    /// This is what the `#[oxcord_macros::command]` macro generates calls to
    /// for any command parameter beyond `ctx`.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Other`] if the value could not be converted
    /// to `T`.
    pub fn extract<T: FrameworkArgument>(&self, name: &str, index: usize) -> Result<Option<T>, FrameworkError> {
        match self.invocation {
            InvocationData::Slash {
                ..
            } => self.option(name),
            InvocationData::Prefix {
                ..
            } => self.arg(index),
        }
    }
}

impl std::fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("author", &self.author.id)
            .field("channel_id", &self.channel_id)
            .field("guild_id", &self.guild_id)
            .finish_non_exhaustive()
    }
}
