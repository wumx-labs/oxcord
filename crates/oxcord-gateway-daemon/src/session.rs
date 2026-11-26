//! Session persistence for the Gateway Daemon.
//!
//! Stores `session_id`, `resume_gateway_url`, and `last_sequence` to disk or memory
//! so restarts do not exhaust Discord's 1000 daily identify limit.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Gateway session metadata for a single shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardSession {
    /// Shard index (0-indexed).
    pub shard_id: u32,
    /// Discord session ID for resuming.
    pub session_id: String,
    /// Dedicated resume gateway URL sent in `Ready`.
    pub resume_gateway_url: String,
    /// Last dispatched sequence number received on this shard.
    pub last_sequence: u64,
    /// Unix timestamp when this session was last updated.
    pub updated_at: u64,
}

/// Thread-safe session store that persists sessions across daemon restarts.
#[derive(Debug, Clone)]
pub struct SessionStore {
    path: PathBuf,
    sessions: Arc<RwLock<HashMap<u32, ShardSession>>>,
}

impl SessionStore {
    /// Create a new session store targeting the given file path.
    ///
    /// If the file exists, it is parsed and existing sessions are pre-loaded.
    #[must_use]
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let mut initial_sessions = HashMap::new();

        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<HashMap<u32, ShardSession>>(&content) {
                    Ok(loaded) => {
                        info!(
                            path = %path.display(),
                            count = loaded.len(),
                            "Loaded existing gateway sessions from disk"
                        );
                        initial_sessions = loaded;
                    },
                    Err(e) => {
                        warn!(
                            path = %path.display(),
                            error = %e,
                            "Failed to parse session file; starting with empty session store"
                        );
                    },
                },
                Err(e) => {
                    warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to read session file"
                    );
                },
            }
        }

        Self {
            path,
            sessions: Arc::new(RwLock::new(initial_sessions)),
        }
    }

    /// Record or update a shard session from a `Ready` payload.
    pub fn record_ready(&self, shard_id: u32, session_id: String, resume_url: String) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);

        let mut lock = self.sessions.write();
        let session = lock.entry(shard_id).or_insert_with(|| ShardSession {
            shard_id,
            session_id: session_id.clone(),
            resume_gateway_url: resume_url.clone(),
            last_sequence: 0,
            updated_at: now,
        });

        session.session_id = session_id;
        session.resume_gateway_url = resume_url;
        session.updated_at = now;

        info!(
            shard_id = shard_id,
            session_id = %session.session_id,
            resume_url = %session.resume_gateway_url,
            "Recorded gateway session for shard"
        );
    }

    /// Update the last sequence number for a shard.
    #[allow(dead_code)]
    pub fn update_sequence(&self, shard_id: u32, seq: u64) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);

        let mut lock = self.sessions.write();
        if let Some(session) = lock.get_mut(&shard_id) {
            if seq > session.last_sequence {
                session.last_sequence = seq;
                session.updated_at = now;
            }
        }
    }

    /// Retrieve the session for a shard if available.
    #[must_use]
    #[allow(dead_code)]
    pub fn get(&self, shard_id: u32) -> Option<ShardSession> {
        self.sessions.read().get(&shard_id).cloned()
    }

    /// Return all active shard sessions.
    #[must_use]
    pub fn all(&self) -> Vec<ShardSession> {
        self.sessions.read().values().cloned().collect()
    }

    /// Persist all active sessions to the configured file path atomically.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let sessions = self.sessions.read().clone();
        let json = serde_json::to_string_pretty(&sessions).map_err(std::io::Error::other)?;

        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        // Write to temporary file and rename for atomic persistence
        let tmp_path = self.path.with_extension("tmp");
        fs::write(&tmp_path, json.as_bytes())?;
        fs::rename(&tmp_path, &self.path)?;

        info!(
            path = %self.path.display(),
            count = sessions.len(),
            "Persisted gateway sessions to disk successfully"
        );
        Ok(())
    }
}
