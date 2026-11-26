//! Pelacakan edit & hapus pesan command prefix (setara `track_edits.rs` di poise).
//!
//! Menyimpan pasangan `(user_msg_id -> bot_response_msg_id)` dengan batas umur (`Duration`)
//! supaya bot bisa mengeksekusi ulang command saat pesan user diedit (`invoke_on_edit`),
//! mengedit balasan lama (`reuse_response`), atau menghapus balasan bot saat pesan asal dihapus
//! (`track_deletion`).

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use oxcord_model::id::{ChannelId, MessageId};
use parking_lot::Mutex;

/// Catatan satu invocation command prefix yang dilacak oleh [`EditTracker`].
#[derive(Debug, Clone)]
pub struct TrackedCommand {
    /// ID pesan invocation milik user.
    pub user_msg_id: MessageId,
    /// ID pesan balasan bot, kalau bot sudah sempat membalas.
    pub bot_response_id: Option<MessageId>,
    /// Channel tempat pesan berada.
    pub channel_id: ChannelId,
    /// Nama command yang dipanggil (untuk cek `invoke_on_edit`/`track_deletion`/`reuse_response`).
    pub command_name: String,
    /// Kapan entri ini dicatat (dipakai untuk purging entri kadaluarsa).
    pub created_at: Instant,
}

/// Pelacak pesan invocation prefix dan balasan bot terkait.
#[derive(Debug)]
pub struct EditTracker {
    /// Berapa lama invocation disimpan di memori.
    max_duration: Duration,
    /// Antrian entri terurut dari yang paling lama ke terbaru.
    entries: Mutex<VecDeque<TrackedCommand>>,
}

impl EditTracker {
    /// Buat [`EditTracker`] baru yang menyimpan entri selama `max_duration`.
    #[must_use]
    pub fn for_timespan(max_duration: Duration) -> Self {
        Self { max_duration, entries: Mutex::new(VecDeque::new()) }
    }

    /// Bersihkan entri yang sudah melewati `max_duration`.
    pub fn purge(&self) {
        let now = Instant::now();
        let mut entries = self.entries.lock();
        while let Some(front) = entries.front() {
            if now.duration_since(front.created_at) > self.max_duration {
                entries.pop_front();
            } else {
                break;
            }
        }
    }

    /// Catat invocation command prefix baru.
    pub fn track_command(
        &self,
        user_msg_id: MessageId,
        channel_id: ChannelId,
        command_name: impl Into<String>,
        bot_response_id: Option<MessageId>,
    ) {
        self.purge();
        let mut entries = self.entries.lock();
        if let Some(existing) = entries.iter_mut().find(|e| e.user_msg_id == user_msg_id) {
            if bot_response_id.is_some() {
                existing.bot_response_id = bot_response_id;
            }
            return;
        }
        entries.push_back(TrackedCommand {
            user_msg_id,
            bot_response_id,
            channel_id,
            command_name: command_name.into(),
            created_at: Instant::now(),
        });
    }

    /// Cari ID pesan balasan bot untuk `user_msg_id` tertentu.
    #[must_use]
    pub fn find_bot_response(&self, user_msg_id: MessageId) -> Option<MessageId> {
        self.purge();
        let entries = self.entries.lock();
        entries.iter().find(|e| e.user_msg_id == user_msg_id).and_then(|e| e.bot_response_id)
    }

    /// Update atau simpan ID balasan bot untuk `user_msg_id`.
    pub fn set_bot_response(&self, user_msg_id: MessageId, bot_response_id: MessageId) {
        self.purge();
        let mut entries = self.entries.lock();
        if let Some(entry) = entries.iter_mut().find(|e| e.user_msg_id == user_msg_id) {
            entry.bot_response_id = Some(bot_response_id);
        }
    }

    /// Proses event `MessageUpdate`: kembalikan entri yang cocok jika `user_msg_id` terlacak.
    #[must_use]
    pub fn process_message_update(&self, user_msg_id: MessageId) -> Option<TrackedCommand> {
        self.purge();
        let entries = self.entries.lock();
        entries.iter().find(|e| e.user_msg_id == user_msg_id).cloned()
    }

    /// Proses event `MessageDelete`: hapus entri dari tracker dan kembalikan jika ada.
    #[must_use]
    pub fn process_message_delete(&self, user_msg_id: MessageId) -> Option<TrackedCommand> {
        self.purge();
        let mut entries = self.entries.lock();
        let pos = entries.iter().position(|e| e.user_msg_id == user_msg_id)?;
        entries.remove(pos)
    }
}

impl Default for EditTracker {
    fn default() -> Self {
        // Default 1 jam, sama seperti poise.
        Self::for_timespan(Duration::from_secs(3600))
    }
}
