//! The unified command context, covering both slash and prefix invocations.

use std::any::Any;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Instant;

use oxcord_builders::create_poll::Ready as PollReady;
use oxcord_builders::{
    Builder,
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateComponent,
    CreateEmbed,
    CreateInteractionResponse,
    CreateInteractionResponseMessage,
    CreateMessage,
    CreatePoll,
    EditInteractionResponse,
    EditMessage,
};
use oxcord_model::application::{CommandDataOption, CommandDataOptionValue, CommandOptionType};
use oxcord_model::channel::{Channel, GuildChannel};
use oxcord_model::guild::{Member, PartialGuild};
use oxcord_model::id::{
    ApplicationId,
    ChannelId,
    GuildId,
    InteractionId,
    MessageId,
    RoleId,
    UserId,
};
use oxcord_model::user::User;
use oxcord_model::{Permissions, Timestamp};
use oxcord_rest::Http;

use crate::command::Command;
use crate::error::FrameworkError;

/// Callback untuk memodifikasi setiap [`Reply`] keluar sebelum dikirim ke Discord.
pub type ReplyCallback = Arc<dyn for<'a> Fn(Context<'a>, Reply) -> Reply + Send + Sync>;

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
        /// The invoking member's resolved permissions in this channel, as
        /// Discord sends them directly on the interaction (guild invocations
        /// only — `None` in DMs). Already accounts for channel overwrites and
        /// roles; no cache/REST lookup needed. Used by [`crate::Command::required_permissions`].
        member_permissions: Option<Permissions>,
        /// The bot's own resolved permissions in this channel, as Discord
        /// sends them directly on the interaction. Used by
        /// [`crate::Command::required_bot_permissions`].
        app_permissions: Option<Permissions>,
        /// Whether the interaction's channel is a thread (affects the
        /// `SEND_MESSAGES`/`SEND_MESSAGES_IN_THREADS` permission swap, same
        /// as poise does).
        is_thread: bool,
    },
    /// Prefix (text) command message content.
    Prefix {
        /// The raw message content after the command name.
        args: String,
        /// Guild the message was sent in, if any.
        guild_id: Option<GuildId>,
        /// The invoking member's resolved permissions in this channel,
        /// computed from the gateway cache by the caller (e.g. the
        /// `oxcord` facade's framework bridge) before dispatch, since
        /// Discord doesn't send resolved permissions on plain messages the
        /// way it does for interactions. `None` when no cache was
        /// available, the guild/channel/member weren't cached yet, or the
        /// message was a DM. Used by [`crate::Command::required_permissions`].
        member_permissions: Option<Permissions>,
        /// The bot's own resolved permissions in this channel, computed the
        /// same way as `member_permissions`. Used by
        /// [`crate::Command::required_bot_permissions`].
        app_permissions: Option<Permissions>,
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
    /// ID pesan asal untuk prefix command (dipakai buat `Reply::reply(true)` & `created_at()`).
    pub(crate) message_id: Option<MessageId>,
    /// Jika diset (dari `EditTracker` dengan `reuse_response`), `Context::send` bakal edit pesan ini alih-alih kirim baru.
    pub(crate) reuse_message_id: Option<MessageId>,
    /// Callback opsional untuk memodifikasi semua outgoing reply.
    pub(crate) reply_callback: Option<ReplyCallback>,
    /// Default `allowed_mentions` dari `FrameworkOptions` kalau reply nggak set sendiri.
    pub(crate) default_allowed_mentions: Option<CreateAllowedMentions>,
    /// Pelacak edit/hapus pesan prefix, kalau dikonfigurasi.
    pub(crate) edit_tracker: Option<Arc<crate::track_edits::EditTracker>>,
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
            message_id: None,
            reuse_message_id: None,
            reply_callback: None,
            default_allowed_mentions: None,
            edit_tracker: None,
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
            message_id: None,
            reuse_message_id: None,
            reply_callback: None,
            default_allowed_mentions: None,
            edit_tracker: None,
            http,
        }
    }

    /// Set `edit_tracker` untuk mencatat ID balasan bot otomatis saat `Context::send()` dipanggil.
    #[must_use]
    pub fn with_edit_tracker(
        mut self,
        tracker: Option<Arc<crate::track_edits::EditTracker>>,
    ) -> Self {
        self.edit_tracker = tracker;
        self
    }

    /// Set ID pesan asal untuk prefix invocation.
    #[must_use]
    pub fn with_message_id(mut self, message_id: MessageId) -> Self {
        self.message_id = Some(message_id);
        self
    }

    /// Set ID pesan balasan lama yang akan di-edit ulang (`reuse_response`).
    #[must_use]
    pub fn with_reuse_message_id(mut self, message_id: MessageId) -> Self {
        self.reuse_message_id = Some(message_id);
        self
    }

    /// Set `reply_callback` dan default `allowed_mentions` dari `FrameworkOptions`.
    #[must_use]
    pub fn with_reply_defaults(
        mut self,
        reply_callback: Option<ReplyCallback>,
        default_allowed_mentions: Option<CreateAllowedMentions>,
    ) -> Self {
        self.reply_callback = reply_callback;
        self.default_allowed_mentions = default_allowed_mentions;
        self
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
            other => {
                Err(FrameworkError::Other(format!("expected Boolean, got {:?}", other.kind())))
            },
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
            other => {
                Err(FrameworkError::Other(format!("expected Integer, got {:?}", other.kind())))
            },
        }
    }

    fn from_token(token: &str) -> Result<Self, FrameworkError> {
        token
            .parse::<Self>()
            .map_err(|_| FrameworkError::Other(format!("'{token}' is not a valid integer")))
    }

    fn option_type() -> CommandOptionType {
        CommandOptionType::Integer
    }
}

impl FrameworkArgument for f64 {
    fn from_option(value: &CommandDataOptionValue) -> Result<Self, FrameworkError> {
        match value {
            CommandDataOptionValue::Number(n) => Ok(*n),
            #[allow(clippy::cast_precision_loss)]
            CommandDataOptionValue::Integer(i) => Ok(*i as Self),
            other => Err(FrameworkError::Other(format!("expected Number, got {:?}", other.kind()))),
        }
    }

    fn from_token(token: &str) -> Result<Self, FrameworkError> {
        token
            .parse::<Self>()
            .map_err(|_| FrameworkError::Other(format!("'{token}' is not a valid number")))
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
        digits.parse::<u64>().map(Self::new).map_err(|_| {
            FrameworkError::Other(format!("'{token}' is not a valid user mention or id"))
        })
    }

    fn option_type() -> CommandOptionType {
        CommandOptionType::User
    }
}

impl FrameworkArgument for ChannelId {
    fn from_option(value: &CommandDataOptionValue) -> Result<Self, FrameworkError> {
        match value {
            CommandDataOptionValue::Channel(id) => Ok(*id),
            other => {
                Err(FrameworkError::Other(format!("expected Channel, got {:?}", other.kind())))
            },
        }
    }

    fn from_token(token: &str) -> Result<Self, FrameworkError> {
        let digits = token.trim_start_matches("<#").trim_end_matches('>');
        digits.parse::<u64>().map(Self::new).map_err(|_| {
            FrameworkError::Other(format!("'{token}' is not a valid channel mention or id"))
        })
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
        digits.parse::<u64>().map(Self::new).map_err(|_| {
            FrameworkError::Other(format!("'{token}' is not a valid role mention or id"))
        })
    }

    fn option_type() -> CommandOptionType {
        CommandOptionType::Role
    }
}

/// Something that can be turned into a [`Reply`] — plain text, or a
/// pre-built [`Reply`] itself (for embeds, or text + embed together).
#[allow(clippy::struct_field_names)]
pub struct Reply {
    content: Option<String>,
    embeds: Vec<CreateEmbed>,
    ephemeral: Option<bool>,
    attachments: Vec<CreateAttachment>,
    components: Option<Vec<CreateActionRow>>,
    components_v2: Option<Vec<CreateComponent>>,
    allowed_mentions: Option<CreateAllowedMentions>,
    poll: Option<CreatePoll<PollReady>>,
    reply: bool,
}

impl Reply {
    /// An empty reply (add content/embeds with the builder methods).
    #[must_use]
    pub fn new() -> Self {
        Self {
            content: None,
            embeds: Vec::new(),
            ephemeral: None,
            attachments: Vec::new(),
            components: None,
            components_v2: None,
            allowed_mentions: None,
            poll: None,
            reply: false,
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

    /// Mark the reply as ephemeral (visible only to the invoking user).
    ///
    /// Slash-command only — Discord has no ephemeral concept for prefix
    /// (regular message) replies, so this is a no-op for those. Only takes
    /// effect on the *first* response to an interaction (the initial
    /// [`Context::send`]/[`Context::say`]/[`Context::defer`]); it can't be
    /// changed by a later [`ReplyHandle::edit`].
    #[must_use]
    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = Some(ephemeral);
        self
    }

    /// Tambah satu file attachment ke reply.
    #[must_use]
    pub fn attachment(mut self, attachment: CreateAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Set daftar attachment reply (menggantikan yang sebelumnya).
    #[must_use]
    pub fn attachments(mut self, attachments: Vec<CreateAttachment>) -> Self {
        self.attachments = attachments;
        self
    }

    /// Set action row components (tombol / select menu) pada reply.
    #[must_use]
    pub fn components(mut self, components: Vec<CreateActionRow>) -> Self {
        self.components = Some(components);
        self
    }

    /// Set `allowed_mentions` khusus untuk reply ini (override default dari `FrameworkOptions`).
    #[must_use]
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Pasang poll Discord native pada reply.
    #[must_use]
    pub fn poll(mut self, poll: CreatePoll<PollReady>) -> Self {
        self.poll = Some(poll);
        self
    }

    /// Kirim sebagai pesan Components V2 (Container, TextDisplay, Section, dst.). Otomatis
    /// menyalakan flag `IS_COMPONENTS_V2`; Discord melarang `content`/`embeds` di pesan V2,
    /// jadi keduanya diabaikan saat V2 dipakai.
    #[must_use]
    pub fn components_v2(mut self, components: Vec<CreateComponent>) -> Self {
        self.components_v2 = Some(components);
        self
    }

    /// Jika `true`, pesan prefix akan dikirim sebagai balasan (reply-to) ke pesan asal user.
    #[must_use]
    pub fn reply(mut self, reply: bool) -> Self {
        self.reply = reply;
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
    /// Ambil `MessageId` jika balasan ini punya ID pesan langsung (prefix command).
    #[must_use]
    pub fn message_id(&self) -> Option<MessageId> {
        match &self.target {
            ReplyTarget::Prefix {
                message_id, ..
            } => Some(*message_id),
            ReplyTarget::Slash {
                ..
            } => None,
        }
    }

    /// Edit the message this handle refers to.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Http`] if the Discord API call fails.
    pub async fn edit(&self, reply: impl Into<Reply>) -> Result<(), FrameworkError> {
        let reply = reply.into();
        let allowed_mentions =
            reply.allowed_mentions.or_else(|| self.http.default_allowed_mentions.clone());
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
                for attachment in reply.attachments {
                    builder = builder.new_attachment(attachment);
                }
                if let Some(v2) = reply.components_v2 {
                    builder = builder.components_v2(v2);
                }
                if let Some(components) = reply.components {
                    builder = builder.components(components);
                }
                if let Some(am) = allowed_mentions {
                    builder = builder.allowed_mentions(am);
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
                for attachment in reply.attachments {
                    builder = builder.new_attachment(attachment);
                }
                if let Some(v2) = reply.components_v2 {
                    builder = builder.components_v2(v2);
                }
                if let Some(components) = reply.components {
                    builder = builder.components(components);
                }
                if let Some(am) = allowed_mentions {
                    builder = builder.allowed_mentions(am);
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

/// Bus internal untuk meneruskan component & modal interaction dari bridge ke `Context`
/// (dipakai oleh `builtins::paginate`, `builtins::register`, dan `Modal::execute`).
#[derive(Debug)]
pub struct InteractionBus {
    /// Channel broadcast untuk `ComponentInteraction` yang masuk dari gateway.
    pub component_tx:
        tokio::sync::broadcast::Sender<oxcord_model::application::ComponentInteraction>,
    /// Channel broadcast untuk `ModalInteraction` yang masuk dari gateway.
    pub modal_tx: tokio::sync::broadcast::Sender<oxcord_model::application::ModalInteraction>,
}

impl InteractionBus {
    /// Buat `InteractionBus` baru dengan kapasitas buffer default.
    #[must_use]
    pub fn new() -> Self {
        let (component_tx, _) = tokio::sync::broadcast::channel(128);
        let (modal_tx, _) = tokio::sync::broadcast::channel(64);
        Self {
            component_tx,
            modal_tx,
        }
    }
}

impl Default for InteractionBus {
    fn default() -> Self {
        Self::new()
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
    /// Context gateway mentah dari sisi pemanggil (di-bungkus `Any` supaya crate ini nggak perlu
    /// depend ke `oxcord`). Lihat [`Context::gateway_context`].
    pub(crate) gateway: Option<Arc<dyn Any + Send + Sync>>,
    /// Command yang sedang dijalankan (setelah subcommand di-resolve).
    pub(crate) command: Option<&'a Command>,
    /// State custom yang cuma hidup selama satu invocation ini.
    pub(crate) invocation_data: Arc<tokio::sync::Mutex<Box<dyn Any + Send + Sync>>>,
    /// Locale user dari slash interaction (selalu `None` buat prefix).
    pub(crate) locale: Option<String>,
    /// Data member pemanggil dari slash interaction (kalau ada).
    pub(crate) author_member: Option<Member>,
    /// String invocation lengkap buat logging/debug (`invocation_string()`).
    pub(crate) invocation_string: String,
    /// Bus interaksi untuk collector komponen / modal bawaan.
    pub(crate) interaction_bus: Option<Arc<InteractionBus>>,
}

impl Context<'_> {
    /// Escape hatch ke context gateway asli (di `oxcord` itu `oxcord::client::Context`: `http`,
    /// `shard`, `data`, dan `cache` kalau fitur cache aktif) — setara `ctx.serenity_context()`
    /// di poise. Dipakai buat `ctx.cache()`, `ctx.guild()`, collector komponen, atau
    /// `latency()` (setara `ctx.ping()`).
    ///
    /// `T` harus tipe persis yang dioper bridge (`oxcord::client::Context`); kalau beda, atau
    /// framework dijalankan tanpa bridge, hasilnya `None`.
    ///
    /// ```rust,ignore
    /// let gw = ctx.gateway_context::<oxcord::client::Context>().unwrap();
    /// let ws_latency = gw.latency().await;
    /// ```
    #[must_use]
    pub fn gateway_context<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.gateway.as_ref()?.downcast_ref::<T>()
    }

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
        let mut reply = reply.into();
        if let Some(cb) = &self.http.reply_callback {
            reply = cb(self.clone(), reply);
        }
        let ephemeral =
            reply.ephemeral.unwrap_or_else(|| self.command.is_some_and(|c| c.ephemeral));
        let allowed_mentions =
            reply.allowed_mentions.or_else(|| self.http.default_allowed_mentions.clone());

        match self.invocation {
            InvocationData::Slash {
                token, ..
            } => {
                let interaction_id = self.http.interaction_id.ok_or_else(|| {
                    FrameworkError::Other("slash invocation missing interaction id".to_owned())
                })?;
                let mut message = CreateInteractionResponseMessage::new().ephemeral(ephemeral);
                if let Some(content) = reply.content {
                    message = message.content(content);
                }
                if !reply.embeds.is_empty() {
                    message = message.embeds(reply.embeds);
                }
                if !reply.attachments.is_empty() {
                    message = message.files(reply.attachments);
                }
                if let Some(v2) = reply.components_v2 {
                    message = message.components_v2(v2);
                }
                if let Some(components) = reply.components {
                    message = message.components(components);
                }
                if let Some(am) = allowed_mentions {
                    message = message.allowed_mentions(am);
                }
                if let Some(poll) = reply.poll {
                    message = message.poll(poll);
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
                // Kalau `reuse_response` aktif dan ada balasan lama dari `EditTracker`, edit pesan itu aja.
                if let Some(reuse_id) = self.http.reuse_message_id {
                    let mut edit = EditMessage::new();
                    if let Some(content) = reply.content {
                        edit = edit.content(content);
                    }
                    if !reply.embeds.is_empty() {
                        edit = edit.embeds(reply.embeds);
                    }
                    for attachment in reply.attachments {
                        edit = edit.new_attachment(attachment);
                    }
                    if let Some(v2) = reply.components_v2 {
                        edit = edit.components_v2(v2);
                    }
                    if let Some(components) = reply.components {
                        edit = edit.components(components);
                    }
                    if let Some(am) = allowed_mentions {
                        edit = edit.allowed_mentions(am);
                    }
                    edit.execute(&*self.http.http, (self.channel_id, reuse_id, None))
                        .await
                        .map_err(|e| FrameworkError::Http(e.to_string()))?;
                    return Ok(ReplyHandle {
                        http: Arc::clone(&self.http),
                        target: ReplyTarget::Prefix {
                            channel_id: self.channel_id,
                            message_id: reuse_id,
                        },
                    });
                }

                let mut message = CreateMessage::new();
                if let Some(content) = reply.content {
                    message = message.content(content);
                }
                if !reply.embeds.is_empty() {
                    message = message.embeds(reply.embeds);
                }
                if !reply.attachments.is_empty() {
                    message = message.files(reply.attachments);
                }
                if let Some(v2) = reply.components_v2 {
                    message = message.components_v2(v2);
                }
                if let Some(components) = reply.components {
                    message = message.components(components);
                }
                if let Some(am) = allowed_mentions {
                    message = message.allowed_mentions(am);
                }
                if let Some(poll) = reply.poll {
                    message = message.poll(poll);
                }
                if reply.reply {
                    if let Some(msg_id) = self.http.message_id {
                        message = message.reference_message((self.channel_id, msg_id));
                    }
                }
                let sent = message
                    .execute(&*self.http.http, (self.channel_id, self.guild_id))
                    .await
                    .map_err(|e| FrameworkError::Http(e.to_string()))?;
                if let (Some(user_msg_id), Some(tracker)) =
                    (self.http.message_id, &self.http.edit_tracker)
                {
                    tracker.set_bot_response(user_msg_id, sent.id);
                }
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

    /// Acknowledge the invocation without a visible reply yet, so a command
    /// that needs more than Discord's ~3 second interaction timeout can
    /// still finish and reply afterwards with [`Self::send`]/[`Self::say`]
    /// (Discord turns the "thinking…" state into an edit of that first
    /// reply automatically).
    ///
    /// Slash-only: for a prefix invocation this instead just sends a typing
    /// indicator (the closest prefix equivalent — there is no interaction to
    /// defer), and never fails the command if that fails.
    ///
    /// # Errors
    ///
    /// Returns [`FrameworkError::Http`] if the Discord API call fails
    /// (slash invocations only).
    pub async fn defer(&self, ephemeral: bool) -> Result<(), FrameworkError> {
        match self.invocation {
            InvocationData::Slash {
                token, ..
            } => {
                let interaction_id = self.http.interaction_id.ok_or_else(|| {
                    FrameworkError::Other("slash invocation missing interaction id".to_owned())
                })?;
                let message = CreateInteractionResponseMessage::new().ephemeral(ephemeral);
                CreateInteractionResponse::Defer(message)
                    .execute(&*self.http.http, (interaction_id, token.as_str()))
                    .await
                    .map_err(|e| FrameworkError::Http(e.to_string()))?;
                Ok(())
            },
            InvocationData::Prefix {
                ..
            } => {
                let _ = self.http.http.broadcast_typing(self.channel_id).await;
                Ok(())
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

    /// The raw Discord REST client, for commands that need to make their own
    /// HTTP calls (e.g. timing a round trip for an "API latency" figure, or
    /// anything [`Self::say`]/[`Self::send`] doesn't cover yet).
    #[must_use]
    pub fn http(&self) -> &Http {
        &self.http.http
    }

    /// Ambil metadata [`Command`] yang sedang dijalankan (kalau ada).
    #[must_use]
    pub fn command(&self) -> Option<&Command> {
        self.command
    }

    /// Baca `custom_data` yang dipasang di [`Command`] saat ini, di-downcast ke `T`.
    #[must_use]
    pub fn command_custom_data<T: Any + Send + Sync + 'static>(&self) -> Option<&T> {
        self.command?.custom_data.as_ref()?.downcast_ref::<T>()
    }

    /// Timestamp pembuatan invocation di sisi Discord (dari `InteractionId` atau `MessageId`).
    #[must_use]
    pub fn created_at(&self) -> Timestamp {
        if let Some(id) = self.http.interaction_id {
            return id.created_at();
        }
        if let Some(id) = self.http.message_id {
            return id.created_at();
        }
        Timestamp::now()
    }

    /// Locale user dari Discord pada slash command (selalu `None` untuk prefix command).
    #[must_use]
    pub fn locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }

    /// Rekonstruksi string invocation lengkap buat kebutuhan logging/debug.
    #[must_use]
    pub fn invocation_string(&self) -> &str {
        &self.invocation_string
    }

    /// Simpan data custom yang cuma hidup selama SATU invocation ini.
    pub async fn set_invocation_data<T: Any + Send + Sync + 'static>(&self, data: T) {
        let mut guard = self.invocation_data.lock().await;
        *guard = Box::new(data);
    }

    /// Baca kembali data custom per-invocation yang sebelumnya diset lewat [`Self::set_invocation_data`].
    pub async fn invocation_data<T: Any + Send + Sync + 'static>(
        &self,
    ) -> Option<tokio::sync::MappedMutexGuard<'_, T>> {
        let guard = self.invocation_data.lock().await;
        tokio::sync::MutexGuard::try_map(guard, |any| any.downcast_mut::<T>()).ok()
    }

    /// Ambil `GuildChannel` tempat command dipanggil lewat REST API Discord (kalau channel-nya adalah guild channel).
    pub async fn guild_channel(&self) -> Option<GuildChannel> {
        match self.http.http.get_channel(self.channel_id).await.ok()? {
            Channel::Guild(ch) => Some(ch),
            _ => None,
        }
    }

    /// Ambil `PartialGuild` tempat command dipanggil lewat REST API Discord (kalau di dalam guild).
    pub async fn partial_guild(&self) -> Option<PartialGuild> {
        let guild_id = self.guild_id?;
        self.http.http.get_guild(guild_id).await.ok()
    }

    /// Ambil `Member` pemanggil command (langsung dari interaction untuk slash, atau lewat REST untuk prefix).
    pub async fn author_member(&self) -> Option<Cow<'_, Member>> {
        if let Some(member) = &self.author_member {
            return Some(Cow::Borrowed(member));
        }
        let guild_id = self.guild_id?;
        let member = self.http.http.get_member(guild_id, self.author.id).await.ok()?;
        Some(Cow::Owned(member))
    }

    /// Jalankan ulang handler command yang sama persis (dipakai misalnya oleh `invoke_on_edit` atau retry).
    ///
    /// # Errors
    ///
    /// Mengembalikan [`FrameworkError`] jika command tidak punya handler atau handler gagal.
    pub async fn rerun(&self) -> Result<(), FrameworkError> {
        let Some(cmd) = self.command else {
            return Err(FrameworkError::Other("no command attached to context".to_owned()));
        };
        let Some(handler) = &cmd.handler else {
            return Err(FrameworkError::Other(format!("'{}' has no handler", cmd.name)));
        };
        handler.call(self.clone()).await
    }

    /// Subscribe ke stream `ComponentInteraction` dari `InteractionBus` (kalau tersedia).
    #[must_use]
    pub fn subscribe_components(
        &self,
    ) -> Option<tokio::sync::broadcast::Receiver<oxcord_model::application::ComponentInteraction>>
    {
        self.interaction_bus.as_ref().map(|b| b.component_tx.subscribe())
    }

    /// Subscribe ke stream `ModalInteraction` dari `InteractionBus` (kalau tersedia).
    #[must_use]
    pub fn subscribe_modals(
        &self,
    ) -> Option<tokio::sync::broadcast::Receiver<oxcord_model::application::ModalInteraction>> {
        self.interaction_bus.as_ref().map(|b| b.modal_tx.subscribe())
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
    pub fn extract<T: FrameworkArgument>(
        &self,
        name: &str,
        index: usize,
    ) -> Result<Option<T>, FrameworkError> {
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

/// Pembungkus milik-sendiri (owned) dari [`Context`] buat dipakai hook `on_error`.
///
/// `Context` meminjam `author`/`invocation`, jadi nggak bisa dibawa masuk future `'static`
/// yang dikembalikan hook. `ErrorContext` nyimpen salinannya; panggil [`ErrorContext::ctx`]
/// buat dapat `Context` biasa (`say`, `send`, `defer`, dst).
#[derive(Clone)]
pub struct ErrorContext {
    /// Nama command (leaf, setelah subcommand di-resolve) yang lagi dijalankan.
    pub command_name: String,
    /// User yang manggil command.
    pub author: User,
    /// Channel tempat command dipanggil.
    pub channel_id: ChannelId,
    /// Guild tempat command dipanggil, kalau ada.
    pub guild_id: Option<GuildId>,
    pub(crate) invocation: InvocationData,
    pub(crate) http: Arc<FrameworkHttp>,
    pub(crate) data: Option<Arc<dyn Any + Send + Sync>>,
    pub(crate) received_at: Instant,
    pub(crate) gateway: Option<Arc<dyn Any + Send + Sync>>,
    pub(crate) invocation_data: Arc<tokio::sync::Mutex<Box<dyn Any + Send + Sync>>>,
    pub(crate) locale: Option<String>,
    pub(crate) author_member: Option<Member>,
    pub(crate) invocation_string: String,
    pub(crate) interaction_bus: Option<Arc<InteractionBus>>,
}

impl ErrorContext {
    pub(crate) fn from_ctx(command_name: &str, ctx: &Context<'_>) -> Self {
        Self {
            command_name: command_name.to_owned(),
            author: ctx.author.clone(),
            channel_id: ctx.channel_id,
            guild_id: ctx.guild_id,
            invocation: ctx.invocation.clone(),
            http: Arc::clone(&ctx.http),
            data: ctx.data.clone(),
            received_at: ctx.received_at,
            gateway: ctx.gateway.clone(),
            invocation_data: Arc::clone(&ctx.invocation_data),
            locale: ctx.locale.clone(),
            author_member: ctx.author_member.clone(),
            invocation_string: ctx.invocation_string.clone(),
            interaction_bus: ctx.interaction_bus.clone(),
        }
    }

    /// Bentuk [`Context`] biasa dari salinan ini, buat balas error ke user.
    #[must_use]
    pub fn ctx(&self) -> Context<'_> {
        Context {
            author: &self.author,
            channel_id: self.channel_id,
            guild_id: self.guild_id,
            invocation: &self.invocation,
            http: Arc::clone(&self.http),
            data: self.data.clone(),
            received_at: self.received_at,
            gateway: self.gateway.clone(),
            command: None,
            invocation_data: Arc::clone(&self.invocation_data),
            locale: self.locale.clone(),
            author_member: self.author_member.clone(),
            invocation_string: self.invocation_string.clone(),
            interaction_bus: self.interaction_bus.clone(),
        }
    }
}

impl std::fmt::Debug for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorContext")
            .field("command_name", &self.command_name)
            .field("author", &self.author.id)
            .field("channel_id", &self.channel_id)
            .finish_non_exhaustive()
    }
}
