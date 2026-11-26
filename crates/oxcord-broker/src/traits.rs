//! Core abstractions for message brokers and subscriptions.

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use async_trait::async_trait;
use futures_core::Stream;
use serde::{Deserialize, Serialize};

use crate::envelope::BrokerMessage;
use crate::error::Result;

/// A subscription stream of messages from a broker topic.
#[async_trait]
pub trait Subscription: Send + Sync {
    /// Awaits the next incoming message on this subscription.
    /// Returns `None` if the subscription stream was closed or dropped.
    async fn next_message(&mut self) -> Option<BrokerMessage>;
}

/// Stream adapter that wraps a [`Subscription`] to implement [`Stream`].
pub struct SubscriptionStream {
    subscription: Box<dyn Subscription>,
}

impl SubscriptionStream {
    /// Wraps any [`Subscription`] into a [`Stream`].
    pub fn new(subscription: Box<dyn Subscription>) -> Self {
        Self {
            subscription,
        }
    }
}

impl Stream for SubscriptionStream {
    type Item = BrokerMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Polling wrapper for async trait next_message
        // Note: For high performance, we use Box::pin on the future or pin projection.
        let fut = self.subscription.next_message();
        tokio::pin!(fut);
        fut.poll(cx)
    }
}

/// The core asynchronous message broker interface.
///
/// Designed to enable seamless horizontal scaling for Discord bots:
/// - Gateway nodes publish Discord events to topics (e.g., `discord.events.message_create`).
/// - Worker nodes subscribe to topics and process commands or state updates.
/// - RPC (`request`) allows workers on different nodes to request data or invoke operations on
///   specific shards.
#[async_trait]
pub trait Broker: Send + Sync {
    /// Publishes a raw byte payload to the specified topic.
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()>;

    /// Publishes a structured [`BrokerMessage`] envelope to its topic.
    async fn publish_message(&self, message: &BrokerMessage) -> Result<()> {
        let serialized = serde_json::to_vec(message)?;
        self.publish(&message.topic, &serialized).await
    }

    /// Subscribes to incoming messages on the specified topic.
    async fn subscribe(&self, topic: &str) -> Result<Box<dyn Subscription>>;

    /// Sends an RPC request to a topic and awaits a single response until `timeout`.
    async fn request(&self, topic: &str, payload: &[u8], timeout: Duration) -> Result<Vec<u8>>;

    /// Helper to send an RPC request with JSON serialization and deserialization.
    async fn request_json<Req: Serialize + Sync, Resp: for<'de> Deserialize<'de>>(
        &self,
        topic: &str,
        request: &Req,
        timeout: Duration,
    ) -> Result<Resp>
    where
        Self: Sized,
    {
        let bytes = serde_json::to_vec(request)?;
        let resp_bytes = self.request(topic, &bytes, timeout).await?;
        let resp: Resp = serde_json::from_slice(&resp_bytes)?;
        Ok(resp)
    }

    /// Responds to an incoming RPC request using the reply-to information in the message envelope.
    async fn reply(&self, request: &BrokerMessage, response_payload: &[u8]) -> Result<()> {
        let reply_to = request.reply_to().ok_or_else(|| {
            crate::BrokerError::Other("cannot reply to message without reply_to header".into())
        })?;

        let mut reply_msg = BrokerMessage::new(reply_to, response_payload.to_vec());
        if let Some(corr_id) = request.correlation_id() {
            reply_msg = reply_msg.with_correlation_id(corr_id);
        }

        self.publish_message(&reply_msg).await
    }
}
