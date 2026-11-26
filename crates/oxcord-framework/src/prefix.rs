//! Konfigurasi lanjutan untuk prefix command (setara `PrefixFrameworkOptions` di poise).

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use oxcord_model::channel::Message;
use oxcord_model::id::{GuildId, UserId};

use crate::error::FrameworkError;
use crate::track_edits::EditTracker;

/// Callback dinamis untuk menentukan prefix per-guild/per-user secara async.
///
/// Menerima `(guild_id, author_id, user_data)` dan mengembalikan `Result<Option<String>, FrameworkError>`.
pub type DynamicPrefixFn = Arc<
    dyn Fn(
            Option<GuildId>,
            UserId,
            Option<Arc<dyn Any + Send + Sync>>,
        ) -> Pin<Box<dyn Future<Output = Result<Option<String>, FrameworkError>> + Send>>
        + Send
        + Sync,
>;

/// Callback "boleh tanpa prefix" (noprefix): `(guild_id, author_id, bot_id, user_data)` -> `true` bila
/// pesan author ini boleh langsung berupa nama command tanpa awalan.
///
/// Dicoba paling akhir, setelah semua prefix biasa gagal cocok (setara `stripped_dynamic_prefix`
/// yang mengembalikan string kosong di poise).
pub type NoPrefixFn = Arc<
    dyn Fn(
            Option<GuildId>,
            UserId,
            Option<UserId>,
            Option<Arc<dyn Any + Send + Sync>>,
        ) -> Pin<Box<dyn Future<Output = bool> + Send>>
        + Send
        + Sync,
>;

/// Callback yang dipanggil untuk setiap pesan yang bukan command prefix.
pub type NonCommandMessageFn = Arc<
    dyn Fn(
            Message,
            Option<Arc<dyn Any + Send + Sync>>,
        ) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send>>
        + Send
        + Sync,
>;

/// Opsi lengkap untuk prefix (text) command, setara `poise::PrefixFrameworkOptions`.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct PrefixOptions {
    /// Prefix tambahan selain `FrameworkOptions::prefix` utama.
    pub additional_prefixes: Vec<String>,
    /// Callback untuk menentukan prefix dinamis (misalnya per-guild dari database).
    pub dynamic_prefix: Option<DynamicPrefixFn>,
    /// Callback noprefix: pengguna tertentu boleh memanggil command tanpa awalan.
    pub noprefix: Option<NoPrefixFn>,
    /// Jika `true`, mention bot (`<@BOT_ID>` atau `<@!BOT_ID>`) dianggap sebagai prefix yang sah.
    pub mention_as_prefix: bool,
    /// Jika `true`, pencocokan nama command & alias prefix tidak membedakan huruf besar/kecil.
    pub case_insensitive_commands: bool,
    /// Jika `true` (default), abaikan pesan dari akun bot lain.
    pub ignore_bots: bool,
    /// Jika `true`, abaikan pesan sistem pembuatan thread (`ThreadCreated`).
    pub ignore_thread_creation: bool,
    /// Jika `true`, bot tetap mengeksekusi pesan command yang dikirim oleh dirinya sendiri.
    pub execute_self_messages: bool,
    /// Callback yang dijalankan untuk pesan biasa yang bukan command.
    pub non_command_message: Option<NonCommandMessageFn>,
    /// Pelacak edit & hapus pesan command prefix.
    pub edit_tracker: Option<Arc<EditTracker>>,
    /// Jika `true`, pesan yang diedit sebelum sempat dilacak tetap bisa dieksekusi sebagai command.
    pub execute_untracked_edits: bool,
    /// Jika `true`, abaikan edit kalau invocation asalnya belum pernah dibalas bot.
    pub ignore_edits_if_not_yet_responded: bool,
    /// Batas waktu sejak pesan dibuat agar edit tidak dilacak dianggap baru (`execute_untracked_edits`).
    pub tracking_initiation_window: Option<Duration>,
}

impl Default for PrefixOptions {
    fn default() -> Self {
        Self {
            additional_prefixes: Vec::new(),
            dynamic_prefix: None,
            noprefix: None,
            mention_as_prefix: false,
            case_insensitive_commands: false,
            ignore_bots: true,
            ignore_thread_creation: false,
            execute_self_messages: false,
            non_command_message: None,
            edit_tracker: None,
            execute_untracked_edits: false,
            ignore_edits_if_not_yet_responded: false,
            tracking_initiation_window: None,
        }
    }
}

impl std::fmt::Debug for PrefixOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrefixOptions")
            .field("additional_prefixes", &self.additional_prefixes)
            .field("dynamic_prefix", &self.dynamic_prefix.is_some())
            .field("noprefix", &self.noprefix.is_some())
            .field("mention_as_prefix", &self.mention_as_prefix)
            .field("case_insensitive_commands", &self.case_insensitive_commands)
            .field("ignore_bots", &self.ignore_bots)
            .field("ignore_thread_creation", &self.ignore_thread_creation)
            .field("execute_self_messages", &self.execute_self_messages)
            .field("non_command_message", &self.non_command_message.is_some())
            .field("edit_tracker", &self.edit_tracker.is_some())
            .field("execute_untracked_edits", &self.execute_untracked_edits)
            .field("ignore_edits_if_not_yet_responded", &self.ignore_edits_if_not_yet_responded)
            .field("tracking_initiation_window", &self.tracking_initiation_window)
            .finish()
    }
}
