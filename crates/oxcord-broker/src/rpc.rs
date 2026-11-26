//! High-level RPC (Remote Procedure Call) dispatcher.
//!
//! Enables distributed cross-shard communication (e.g., worker querying a specific shard
//! for its guild count, kicking a member on another shard, or fetching shard latency),
//! mirroring the pattern provided by `@discordjs/brokers`'s `RPCRedisBroker`.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::envelope::BrokerMessage;
use crate::error::Result;
use crate::traits::Broker;

/// Async callback handler for an RPC topic.
pub type RpcCallback = Box<
    dyn Fn(BrokerMessage) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send>> + Send + Sync,
>;

/// A dispatcher that listens for incoming RPC calls on subscribed topics and automatically
/// returns replies to the caller's `reply_to` topic.
#[derive(Clone)]
pub struct RpcDispatcher<B: Broker> {
    broker: Arc<B>,
    handlers: Arc<RwLock<HashMap<String, Arc<RpcCallback>>>>,
}

impl<B: Broker + 'static> RpcDispatcher<B> {
    /// Creates a new RPC dispatcher over the given broker.
    pub fn new(broker: B) -> Self {
        Self {
            broker: Arc::new(broker),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new RPC dispatcher from an existing `Arc<Broker>`.
    pub fn from_arc(broker: Arc<B>) -> Self {
        Self {
            broker,
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers an async handler for a given RPC topic.
    pub async fn register<F, Fut>(&self, topic: impl Into<String>, handler: F)
    where
        F: Fn(BrokerMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Vec<u8>>> + Send + 'static,
    {
        let mut handlers = self.handlers.write().await;
        let callback: RpcCallback = Box::new(move |msg| Box::pin(handler(msg)));
        handlers.insert(topic.into(), Arc::new(callback));
    }

    /// Spawns a background listener on the specified topic, automatically executing registered
    /// handlers and sending replies.
    pub async fn listen(&self, topic: &str) -> Result<tokio::task::JoinHandle<()>> {
        let mut sub = self.broker.subscribe(topic).await?;
        let handlers = Arc::clone(&self.handlers);
        let broker = Arc::clone(&self.broker);
        let topic_str = topic.to_string();

        let handle = tokio::spawn(async move {
            while let Some(msg) = sub.next_message().await {
                let handler_opt = {
                    let map = handlers.read().await;
                    map.get(&topic_str).cloned()
                };

                if let Some(handler) = handler_opt {
                    let broker = Arc::clone(&broker);
                    tokio::spawn(async move {
                        let is_rpc = msg.is_rpc_request();
                        match handler(msg.clone()).await {
                            Ok(reply_bytes) => {
                                if is_rpc {
                                    if let Err(e) = broker.reply(&msg, &reply_bytes).await {
                                        tracing::error!("Failed to send RPC reply: {e}");
                                    }
                                }
                            },
                            Err(e) => {
                                tracing::error!("RPC handler error on topic {}: {e}", msg.topic);
                            },
                        }
                    });
                }
            }
        });

        Ok(handle)
    }

    /// Sends an RPC call to a remote topic and awaits the reply.
    pub async fn call(
        &self,
        topic: &str,
        payload: &[u8],
        timeout: std::time::Duration,
    ) -> Result<Vec<u8>> {
        self.broker.request(topic, payload, timeout).await
    }
}
