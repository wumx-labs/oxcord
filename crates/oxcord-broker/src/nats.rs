//! NATS driver for `Broker`.
//!
//! Available when the `nats` feature is enabled.

use std::time::Duration;

use async_trait::async_trait;
use futures_util::StreamExt;

use crate::envelope::BrokerMessage;
use crate::error::{BrokerError, Result};
use crate::traits::{Broker, Subscription};

/// NATS subscription wrapper.
pub struct NatsSubscription {
    subscriber: async_nats::Subscriber,
    topic: String,
}

#[async_trait]
impl Subscription for NatsSubscription {
    async fn next_message(&mut self) -> Option<BrokerMessage> {
        let nats_msg = self.subscriber.next().await?;
        let payload = nats_msg.payload.to_vec();

        // Deserialize envelope if available, else wrap raw
        let msg = serde_json::from_slice::<BrokerMessage>(&payload)
            .unwrap_or_else(|_| BrokerMessage::new(&self.topic, payload));
        Some(msg)
    }
}

/// A message broker backed by NATS.
#[derive(Clone)]
pub struct NatsBroker {
    client: async_nats::Client,
}

impl NatsBroker {
    /// Connects to a NATS server or cluster (e.g. `"127.0.0.1:4222"` or `"demo.nats.io"`).
    pub async fn connect(nats_url: &str) -> Result<Self> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| BrokerError::Transport(e.to_string()))?;
        Ok(Self {
            client,
        })
    }

    /// Creates a `NatsBroker` from an existing [`async_nats::Client`].
    pub fn from_client(client: async_nats::Client) -> Self {
        Self {
            client,
        }
    }
}

#[async_trait]
impl Broker for NatsBroker {
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        self.client
            .publish(topic.to_string(), payload.to_vec().into())
            .await
            .map_err(|e| BrokerError::Transport(e.to_string()))?;
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<Box<dyn Subscription>> {
        let subscriber = self
            .client
            .subscribe(topic.to_string())
            .await
            .map_err(|e| BrokerError::Transport(e.to_string()))?;

        Ok(Box::new(NatsSubscription {
            subscriber,
            topic: topic.to_string(),
        }))
    }

    async fn request(&self, topic: &str, payload: &[u8], timeout: Duration) -> Result<Vec<u8>> {
        // Native NATS request-reply with built-in timeout
        let fut = self.client.request(topic.to_string(), payload.to_vec().into());

        match tokio::time::timeout(timeout, fut).await {
            Ok(Ok(response)) => Ok(response.payload.to_vec()),
            Ok(Err(e)) => Err(BrokerError::Transport(e.to_string())),
            Err(_) => {
                Err(BrokerError::Timeout(format!("timeout waiting for NATS reply on {topic}")))
            },
        }
    }
}
