//! Core abstraction traits that let a bot move from one process to many without being rewritten.
//!
//! Only the contracts and a few trivial implementations live here for now. Heavier backends
//! (Redis cache, proxy rate limiter, NATS broker, cross-process sharding) are added later behind
//! these same traits.
//!
//! The traits use `impl Future + Send` in return position, so they need no `async_trait`.

use std::future::Future;
use std::sync::Arc;

use futures::Stream;

use crate::Result;
use crate::model::event::Event;
use crate::model::guild::Guild;
use crate::model::id::{GuildId, ShardId};

/// Storage for data received from the gateway.
///
/// Note: this is distinct from the concrete [`crate::cache::Cache`] struct, which is the
/// in-memory implementation used by the client today.
pub trait Cache: Send + Sync {
    /// Looks up a guild by its ID.
    fn guild(&self, id: GuildId) -> impl Future<Output = Option<Arc<Guild>>> + Send;

    /// Applies a gateway event to the stored state.
    fn apply(&self, event: &Event) -> impl Future<Output = ()> + Send;
}

/// A cache that stores nothing. Useful for stateless workers.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoCache;

impl Cache for NoCache {
    fn guild(&self, _id: GuildId) -> impl Future<Output = Option<Arc<Guild>>> + Send {
        std::future::ready(None)
    }

    fn apply(&self, _event: &Event) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }
}

/// Decides when a REST request may be sent, and learns from the response headers.
///
/// Implementations may be local (in-process), remote (a shared proxy) or backed by a store.
#[cfg(feature = "http")]
pub trait RateLimiter: Send + Sync {
    /// Guard held while a request is in flight.
    type Permit: Send;

    /// Waits until a request on this bucket may be sent.
    fn acquire(
        &self,
        bucket: &crate::http::RatelimitingBucket,
    ) -> impl Future<Output = Self::Permit> + Send;

    /// Feeds the ratelimit headers of a response back into the limiter.
    fn update(
        &self,
        bucket: &crate::http::RatelimitingBucket,
        headers: &reqwest::header::HeaderMap,
    );
}

/// A message broker used to move gateway events from gateway processes to workers.
pub trait Broker: Send + Sync {
    /// Stream of raw payloads for a subscribed topic.
    type Subscription: Stream<Item = Vec<u8>> + Send + Unpin;

    /// Publishes a payload to a topic.
    fn publish(&self, topic: &str, payload: &[u8]) -> impl Future<Output = Result<()>> + Send;

    /// Subscribes to a topic.
    fn subscribe(&self, topic: &str) -> impl Future<Output = Result<Self::Subscription>> + Send;
}

/// Chooses which shards the current process runs.
pub trait ShardStrategy: Send + Sync {
    /// Returns the shard IDs this process is responsible for, given the total shard count.
    fn assign(&self, total: u32) -> Vec<ShardId>;
}

/// Runs every shard in the current process.
#[derive(Clone, Copy, Debug, Default)]
pub struct LocalShards;

impl ShardStrategy for LocalShards {
    fn assign(&self, total: u32) -> Vec<ShardId> {
        (0..total).map(ShardId).collect()
    }
}

/// Runs the shards `i` for which `i % process_count == process_index`.
#[derive(Clone, Copy, Debug)]
pub struct ProcessShards {
    /// Zero-based index of this process.
    pub process_index: u32,
    /// Total number of cooperating processes.
    pub process_count: u32,
}

impl ShardStrategy for ProcessShards {
    fn assign(&self, total: u32) -> Vec<ShardId> {
        if self.process_count == 0 {
            return Vec::new();
        }
        (0..total).filter(|i| i % self.process_count == self.process_index).map(ShardId).collect()
    }
}
