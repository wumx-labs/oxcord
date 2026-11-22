//! Global and per-route Discord rate limit tracking.
//!
//! # Design
//!
//! Discord rate limits are per *bucket* (a hash sent in the `X-RateLimit-Bucket`
//! response header). Multiple routes can share a bucket. This module tracks:
//!
//! 1. **Global** — 50 req/s across all routes (configurable via `GLOBAL_RATE_LIMIT`).
//! 2. **Per-bucket** — remaining calls and reset timestamp from Discord response headers.
//!
//! When a bucket is exhausted the proxy waits (`tokio::time::sleep_until`) before
//! forwarding the next request in that bucket, preventing 429s from reaching Discord.

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::debug;

/// Per-route-bucket state.
#[derive(Debug)]
pub struct BucketState {
    /// Remaining requests allowed in this window.
    pub remaining: i64,
    /// When the window resets (wall clock).
    pub reset_at: Option<Instant>,
}

/// Central rate limit coordinator.
#[derive(Debug)]
pub struct GlobalLimiter {
    /// Semaphore that caps global concurrency at `max_rps` slots.
    global: Arc<Semaphore>,
    /// Per-bucket window tracking keyed by Discord bucket hash.
    buckets: DashMap<String, BucketState>,
    /// Maximum concurrent in-flight global requests.
    #[allow(dead_code)]
    max_rps: u32,
}

impl GlobalLimiter {
    /// Create a new limiter with the given global cap.
    #[must_use]
    pub fn new(max_rps: u32) -> Self {
        Self {
            global: Arc::new(Semaphore::new(max_rps as usize)),
            buckets: DashMap::new(),
            max_rps,
        }
    }

    /// Acquire a global slot. Returned permit is released on drop.
    pub async fn acquire(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.global.acquire().await.expect("global limiter semaphore closed")
    }

    /// Check whether a known bucket must wait before the next request.
    ///
    /// If `remaining == 0` and `reset_at` is in the future, sleeps until reset.
    pub async fn wait_for_bucket(&self, bucket: &str) {
        let wait_until = {
            let entry = self.buckets.get(bucket);
            match entry.as_deref() {
                Some(state) if state.remaining <= 0 => state.reset_at,
                _ => None,
            }
        };

        if let Some(reset) = wait_until {
            let now = Instant::now();
            if reset > now {
                let wait = reset.duration_since(now) + Duration::from_millis(50);
                debug!(bucket = %bucket, wait_ms = %wait.as_millis(), "bucket exhausted — sleeping");
                sleep(wait).await;
            }
        }
    }

    /// Update bucket state from Discord response headers.
    pub fn update_from_headers(
        &self,
        bucket: Option<String>,
        remaining: Option<i64>,
        reset_after_secs: Option<f64>,
    ) {
        let Some(bucket) = bucket else { return };
        let remaining = remaining.unwrap_or(1);
        let reset_at = reset_after_secs.map(|s| Instant::now() + Duration::from_secs_f64(s));

        self.buckets.insert(bucket.clone(), BucketState {
            remaining,
            reset_at,
        });
        debug!(bucket = %bucket, remaining = %remaining, "bucket updated");
    }

    /// Maximum global requests-per-second cap.
    #[must_use]
    #[allow(dead_code)]
    pub fn max_rps(&self) -> u32 {
        self.max_rps
    }
}
