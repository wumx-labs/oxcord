//! Cache sweeper — periodic cleanup of oversized or stale cache entries.
//!
//! Call [`Cache::sweep`] manually, or configure [`Settings::sweep_interval`] so that
//! the [`oxcord`] client spawns a background task that calls it automatically.

use std::collections::HashSet;

use oxcord_model::prelude::UserId;

use crate::Cache;

// ── Public stats type ─────────────────────────────────────────────────────────

/// Statistics returned by [`Cache::sweep`] describing how many entries were removed.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct SweepStats {
    /// Number of messages evicted from the message cache.
    pub messages_removed: usize,
    /// Number of users evicted from the user cache.
    pub users_removed: usize,
}

// ── Sweeper methods on Cache ──────────────────────────────────────────────────

impl Cache {
    /// Remove excess messages from every channel's message cache.
    ///
    /// For each channel, messages beyond [`Settings::max_messages`] (oldest first) are
    /// dropped. When `max_messages` is `0` nothing is removed.
    ///
    /// Returns the total number of messages removed.
    pub fn sweep_messages(&self) -> usize {
        let max = self.settings.read().max_messages;
        if max == 0 {
            return 0;
        }

        let mut removed = 0usize;

        // Iterate over channels that have a message queue.
        for mut queue_entry in self.message_queue.iter_mut() {
            let channel_id = *queue_entry.key();
            let queue = queue_entry.value_mut();

            // Drop the oldest messages until we're within the limit.
            while queue.len() > max {
                if let Some(msg_id) = queue.pop_front() {
                    if let Some(mut channel_msgs) = self.messages.get_mut(&channel_id) {
                        if channel_msgs.remove(&msg_id).is_some() {
                            removed += 1;
                        }
                    }
                }
            }
        }

        removed
    }

    /// Remove users from the user cache that are not members of any cached guild.
    ///
    /// "Orphaned" users — those not present as a member in *any* guild currently held
    /// in the cache — are evicted.  Additionally, if [`Settings::max_users`] is
    /// non-zero and the user cache still exceeds the limit after orphan removal, the
    /// oldest entries (by insertion order, approximated by iteration order) are also
    /// removed until the cache fits within the limit.
    ///
    /// Returns the number of users removed.
    pub fn sweep_users(&self) -> usize {
        let users_map = match &self.users.0 {
            Some(m) => m,
            None => return 0,
        };

        // Build the set of user IDs that appear in at least one guild's member list.
        let mut known_members: HashSet<UserId> = HashSet::new();
        for guild_entry in self.guilds.iter() {
            for user_id in guild_entry.members.keys() {
                known_members.insert(*user_id);
            }
        }

        // Also keep the bot's own user ID.
        let self_id = self.user.read().id;
        known_members.insert(self_id);

        // Collect orphaned user IDs.
        let orphans: Vec<UserId> = users_map
            .iter()
            .filter(|e| !known_members.contains(e.key()))
            .map(|e| *e.key())
            .collect();

        let mut removed = orphans.len();
        for id in orphans {
            users_map.remove(&id);
        }

        // Enforce max_users limit if set.
        let max_users = self.settings.read().max_users;
        if max_users > 0 {
            while users_map.len() > max_users {
                // DashMap has no guaranteed order; take any entry.
                if let Some(entry) = users_map.iter().next().map(|e| *e.key()) {
                    users_map.remove(&entry);
                    removed += 1;
                } else {
                    break;
                }
            }
        }

        removed
    }

    /// Run all sweep operations in one call.
    ///
    /// This is called by the background sweeper task when
    /// [`Settings::sweep_interval`] is configured.
    pub fn sweep(&self) -> SweepStats {
        SweepStats {
            messages_removed: self.sweep_messages(),
            users_removed: self.sweep_users(),
        }
    }
}
