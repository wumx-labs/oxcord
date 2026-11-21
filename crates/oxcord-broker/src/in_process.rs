//! In-process broker implementation using Tokio broadcast channels.
//!
//! Ideal for unit tests, local multi-task architectures, and single-binary sharded bots
//! without requiring an external Redis or NATS server.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::{RwLock, broadcast};

use crate::envelope::BrokerMessage;
use crate::error::{BrokerError, Result};
use crate::traits::{Broker, Subscription};

const DEFAULT_CHANNEL_CAPACITY: usize = 1024;

/// In-process subscription backed by a [`tokio::sync::broadcast::Receiver`].
pub struct InProcessSubscription {
    receiver: broadcast::Receiver<BrokerMessage>,
}

#[async_trait]
impl Subscription for InProcessSubscription {
    async fn next_message(&mut self) -> Option<BrokerMessage> {
        loop {
            match self.receiver.recv().await {
                Ok(msg) => return Some(msg),
                Err(broadcast::error::RecvError::Lagged(missed)) => {
                    tracing::warn!("InProcessSubscription lagged, missed {missed} messages");
                    continue;
                },
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }
}

/// A zero-dependency in-process message broker using concurrent broadcast channels.
#[derive(Clone)]
pub struct InProcessBroker {
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<BrokerMessage>>>>,
    channel_capacity: usize,
    correlation_seq: Arc<AtomicU64>,
}

impl Default for InProcessBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl InProcessBroker {
    /// Creates a new `InProcessBroker` with default buffer capacity.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHANNEL_CAPACITY)
    }

    /// Creates a new `InProcessBroker` with custom broadcast channel capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            channel_capacity: capacity,
            correlation_seq: Arc::new(AtomicU64::new(1)),
        }
    }

    async fn get_or_create_sender(&self, topic: &str) -> broadcast::Sender<BrokerMessage> {
        let read = self.channels.read().await;
        if let Some(sender) = read.get(topic) {
            return sender.clone();
        }
        drop(read);

        let mut write = self.channels.write().await;
        write
            .entry(topic.to_string())
            .or_insert_with(|| broadcast::channel(self.channel_capacity).0)
            .clone()
    }
}

#[async_trait]
impl Broker for InProcessBroker {
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let msg = BrokerMessage::new(topic, payload.to_vec());
        self.publish_message(&msg).await
    }

    async fn publish_message(&self, message: &BrokerMessage) -> Result<()> {
        let sender = self.get_or_create_sender(&message.topic).await;
        // Ignore SendError if there are currently no active receivers
        let _ = sender.send(message.clone());
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<Box<dyn Subscription>> {
        let sender = self.get_or_create_sender(topic).await;
        let receiver = sender.subscribe();
        Ok(Box::new(InProcessSubscription {
            receiver,
        }))
    }

    async fn request(&self, topic: &str, payload: &[u8], timeout: Duration) -> Result<Vec<u8>> {
        let seq = self.correlation_seq.fetch_add(1, Ordering::Relaxed);
        let reply_topic = format!("_inproc.reply.{seq}");
        let correlation_id = format!("corr-{seq}");

        let mut sub = self.subscribe(&reply_topic).await?;

        let request_msg = BrokerMessage::new(topic, payload.to_vec())
            .with_reply_to(&reply_topic)
            .with_correlation_id(&correlation_id);

        self.publish_message(&request_msg).await?;

        // Await the response with timeout
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
                "timeout waiting for reply on {topic} (corr: {correlation_id})"
            ))),
        }
    }
}
