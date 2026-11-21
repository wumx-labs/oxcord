use std::time::Duration;

/// An exponential backoff calculator with optional full jitter.
///
/// Implements exponential backoff according to the Full Jitter algorithm:
/// `sleep = rand_between(min_delay, min(max_delay, base_delay * factor^attempt))`
///
/// This prevents the "thundering herd" problem when multiple shards or connections
/// attempt to reconnect simultaneously after a network outage or Discord service restart.
#[derive(Clone, Debug)]
pub struct Backoff {
    current_delay: Duration,
    min_delay: Duration,
    max_delay: Duration,
    factor: f64,
    jitter: bool,
    attempts: u32,
}

impl Backoff {
    /// Creates a new `Backoff` with default parameters:
    /// - `min_delay`: 500ms
    /// - `max_delay`: 60s
    /// - `factor`: 2.0
    /// - `jitter`: true
    pub fn new() -> Self {
        Self {
            current_delay: Duration::from_millis(500),
            min_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            factor: 2.0,
            jitter: true,
            attempts: 0,
        }
    }

    /// Sets the minimum backoff delay.
    pub fn min_delay(mut self, min: Duration) -> Self {
        self.min_delay = min;
        if self.attempts == 0 {
            self.current_delay = min;
        }
        self
    }

    /// Sets the maximum backoff delay cap.
    pub fn max_delay(mut self, max: Duration) -> Self {
        self.max_delay = max;
        self
    }

    /// Sets the multiplier factor (default 2.0).
    pub fn factor(mut self, factor: f64) -> Self {
        self.factor = factor;
        self
    }

    /// Enables or disables jitter.
    pub fn jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Resets the backoff to its initial minimum delay and clears attempt count.
    pub fn reset(&mut self) {
        self.attempts = 0;
        self.current_delay = self.min_delay;
    }

    /// Returns the number of consecutive failed attempts.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    /// Computes the next backoff duration and advances the attempt counter.
    pub fn next_backoff(&mut self) -> Duration {
        self.attempts = self.attempts.saturating_add(1);

        let delay = self.current_delay;
        let delay_ms = self.current_delay.as_millis() as f64;
        let next_ms = (delay_ms * self.factor).min(self.max_delay.as_millis() as f64);
        self.current_delay = Duration::from_millis(next_ms as u64);

        if self.jitter {
            // Lightweight SplitMix64 PRNG seeded with current nanos + attempts
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(42) as u64;
            let mut x = seed.wrapping_add((self.attempts as u64).wrapping_mul(0x9e3779b97f4a7c15));
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
            x ^= x >> 31;
            let random_fraction = (x as f64) / (u64::MAX as f64);

            let jittered_ms = (delay.as_millis() as f64 * random_fraction) as u64;
            Duration::from_millis(jittered_ms).clamp(self.min_delay, self.max_delay)
        } else {
            delay
        }
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_progression_without_jitter() {
        let mut backoff = Backoff::new()
            .min_delay(Duration::from_millis(100))
            .max_delay(Duration::from_millis(1000))
            .factor(2.0)
            .jitter(false);

        assert_eq!(backoff.attempts(), 0);

        let d1 = backoff.next_backoff();
        assert_eq!(d1, Duration::from_millis(100));
        assert_eq!(backoff.attempts(), 1);

        let d2 = backoff.next_backoff();
        assert_eq!(d2, Duration::from_millis(200));

        let d3 = backoff.next_backoff();
        assert_eq!(d3, Duration::from_millis(400));

        let d4 = backoff.next_backoff();
        assert_eq!(d4, Duration::from_millis(800));

        let d5 = backoff.next_backoff();
        assert_eq!(d5, Duration::from_millis(1000)); // capped at max_delay

        backoff.reset();
        assert_eq!(backoff.attempts(), 0);
        assert_eq!(backoff.next_backoff(), Duration::from_millis(100));
    }

    #[test]
    fn test_backoff_with_jitter_within_bounds() {
        let mut backoff = Backoff::new()
            .min_delay(Duration::from_millis(50))
            .max_delay(Duration::from_millis(500))
            .jitter(true);

        for _ in 0..20 {
            let delay = backoff.next_backoff();
            assert!(delay >= Duration::from_millis(50));
            assert!(delay <= Duration::from_millis(500));
        }
    }
}
