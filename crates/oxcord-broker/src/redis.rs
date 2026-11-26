//! Redis Pub/Sub driver for `Broker`.
//!
//! Available when the `redis` feature is enabled.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use futures_util::StreamExt;
use redis::AsyncCommands;
use tokio::sync::Mutex;

use crate::envelope::BrokerMessage;
use crate::error::{BrokerError, Result};
use crate::traits::{Broker, Subscription};

/// Redis Pub/Sub subscription wrapper.
pub struct RedisSubscription {
    topic: String,
    stream: Mutex<redis::aio::PubSubStream>,
}

#[async_trait]
impl Subscription for RedisSubscription {
    async fn next_message(&mut self) -> Option<BrokerMessage> {
        let mut stream = self.stream.lock().await;
        let msg = stream.next().await?;
        let payload: Vec<u8> = msg.get_payload_bytes().to_vec();

        let broker_msg = serde_json::from_slice::<BrokerMessage>(&payload)
            .unwrap_or_else(|_| BrokerMessage::new(&self.topic, payload));
        Some(broker_msg)
    }
}

/// A message broker backed by Redis Pub/Sub.
#[derive(Clone)]
pub struct RedisBroker {
    client: redis::Client,
    conn: redis::aio::MultiplexedConnection,
    correlation_seq: Arc<AtomicU64>,
}

impl RedisBroker {
    /// Connects to a Redis server with the given URL (e.g. `"redis://127.0.0.1:6379"`).
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client =
            redis::Client::open(redis_url).map_err(|e| BrokerError::Transport(e.to_string()))?;
        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| BrokerError::Transport(e.to_string()))?;

        Ok(Self {
            client,
            conn,
            correlation_seq: Arc::new(AtomicU64::new(1)),
        })
    }
}

#[async_trait]
impl Broker for RedisBroker {
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let mut conn = self.conn.clone();
        conn.publish::<_, _, ()>(topic, payload)
            .await
            .map_err(|e| BrokerError::Transport(e.to_string()))?;
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<Box<dyn Subscription>> {
        let mut pubsub = self
            .client
            .get_async_pubsub()
            .await
            .map_err(|e| BrokerError::Transport(e.to_string()))?;

        pubsub.subscribe(topic).await.map_err(|e| BrokerError::Transport(e.to_string()))?;

        let stream = pubsub.into_on_message();

        Ok(Box::new(RedisSubscription {
            topic: topic.to_string(),
            stream: Mutex::new(stream),
        }))
    }

    async fn request(&self, topic: &str, payload: &[u8], timeout: Duration) -> Result<Vec<u8>> {
        let seq = self.correlation_seq.fetch_add(1, Ordering::Relaxed);
        let reply_topic = format!("_redis.reply.{seq}");
        let correlation_id = format!("corr-{seq}");

        let mut sub = self.subscribe(&reply_topic).await?;

        let request_msg = BrokerMessage::new(topic, payload.to_vec())
            .with_reply_to(&reply_topic)
            .with_correlation_id(&correlation_id);

        self.publish_message(&request_msg).await?;

        let result = tokio::time::timeout(timeout, async {
            while let Some(msg) = sub.next_message().await {
                if msg.correlation_id() == Some(&correlation_id) {
                    return Ok(msg.payload);
                }
            }
            Err(BrokerError::SubscriptionClosed)
        })
        .await;

        match result {
            Ok(inner) => inner,
            Err(_) => Err(BrokerError::Timeout(format!(
                "timeout waiting for Redis reply on {topic} (corr: {correlation_id})"
            ))),
        }
    }
}
