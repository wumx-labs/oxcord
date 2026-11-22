//! Redis implementation of [`KeyValueStore`].
//!
//! Provides distributed Discord state storage on top of Redis / Dragonfly / KeyDB.

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;

use super::StateBackendError;
use super::key_value::{KeyValueStateBackend, KeyValueStore};

/// A Redis-backed implementation of [`KeyValueStore`].
#[derive(Clone)]
pub struct RedisStore {
    mgr: ConnectionManager,
}

impl RedisStore {
    /// Connects to a Redis instance at the given URL (e.g. `"redis://127.0.0.1:6379"`).
    pub async fn new(redis_url: &str) -> Result<Self, StateBackendError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| StateBackendError::Storage(e.to_string()))?;
        let mgr = ConnectionManager::new(client)
            .await
            .map_err(|e| StateBackendError::Storage(e.to_string()))?;
        Ok(Self {
            mgr,
        })
    }

    /// Creates a `RedisStore` from an existing [`ConnectionManager`].
    pub fn from_manager(mgr: ConnectionManager) -> Self {
        Self {
            mgr,
        }
    }
}

#[async_trait]
impl KeyValueStore for RedisStore {
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, StateBackendError> {
        let mut conn = self.mgr.clone();
        let res: Option<Vec<u8>> =
            conn.get(key).await.map_err(|e| StateBackendError::Storage(e.to_string()))?;
        Ok(res)
    }

    async fn set_raw(
        &self,
        key: &str,
        value: &[u8],
        ttl_secs: Option<u64>,
    ) -> Result<(), StateBackendError> {
        let mut conn = self.mgr.clone();
        if let Some(ttl) = ttl_secs {
            let _: () = conn
                .set_ex(key, value, ttl)
                .await
                .map_err(|e| StateBackendError::Storage(e.to_string()))?;
        } else {
            let _: () = conn
                .set(key, value)
                .await
                .map_err(|e| StateBackendError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    async fn del_raw(&self, key: &str) -> Result<bool, StateBackendError> {
        let mut conn = self.mgr.clone();
        let count: usize =
            conn.del(key).await.map_err(|e| StateBackendError::Storage(e.to_string()))?;
        Ok(count > 0)
    }

    async fn keys_with_prefix(&self, prefix: &str) -> Result<Vec<String>, StateBackendError> {
        let mut conn = self.mgr.clone();
        let pattern = format!("{prefix}*");
        let keys: Vec<String> =
            conn.keys(pattern).await.map_err(|e| StateBackendError::Storage(e.to_string()))?;
        Ok(keys)
    }

    async fn count_prefix(&self, prefix: &str) -> Result<usize, StateBackendError> {
        let keys = self.keys_with_prefix(prefix).await?;
        Ok(keys.len())
    }

    async fn clear_prefix(&self, prefix: &str) -> Result<usize, StateBackendError> {
        let keys = self.keys_with_prefix(prefix).await?;
        if keys.is_empty() {
            return Ok(0);
        }
        let mut conn = self.mgr.clone();
        let count: usize =
            conn.del(&keys).await.map_err(|e| StateBackendError::Storage(e.to_string()))?;
        Ok(count)
    }
}

/// A convenient type alias for a [`KeyValueStateBackend`] backed by [`RedisStore`].
pub type RedisStateBackend = KeyValueStateBackend<RedisStore>;

impl RedisStateBackend {
    /// Connects to Redis and creates a [`RedisStateBackend`] with the default namespace `"oxcord"`.
    pub async fn connect(redis_url: &str) -> Result<Self, StateBackendError> {
        let store = RedisStore::new(redis_url).await?;
        Ok(Self::new(store))
    }

    /// Connects to Redis and creates a [`RedisStateBackend`] with a custom namespace prefix.
    pub async fn connect_with_prefix(
        redis_url: &str,
        prefix: impl Into<String>,
    ) -> Result<Self, StateBackendError> {
        let store = RedisStore::new(redis_url).await?;
        Ok(Self::with_prefix(store, prefix))
    }
}
