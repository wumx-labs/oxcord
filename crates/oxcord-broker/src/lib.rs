//! # oxcord-broker
//!
//! Pluggable message broker abstraction, distributed sharding communication, and RPC
//! for large-scale Discord bots (T3/T4 scale).
//!
//! ## Overview
//!
//! For enterprise-scale bots (thousands of shards), a single process cannot handle all
//! WebSocket gateway connections and command processing. `oxcord-broker` decouples the
//! architecture into distributed microservices:
//!
//! 1. **Gateway Sharder Nodes**: Connect to Discord Gateway WebSocket, publish events to broker
//!    topics.
//! 2. **Message Broker**: NATS, Redis Pub/Sub, Kafka, or In-Process broadcast bus.
//! 3. **Worker Nodes**: Subscribe to event topics, handle state cache, and execute bot commands.
//! 4. **RPC Communication**: Cross-shard querying (e.g. asking Shard 42 for its voice status).
//!
//! ## Supported Drivers
//!
//! | Driver | Feature Flag | Use Case |
//! |---|---|---|
//! | [`InProcessBroker`] | *(Default)* | Testing, development, single-node multi-tasking |
//! | `RedisBroker` | `redis` | Multi-container setups with existing Redis/Dragonfly (Pub/Sub) |
//! | `NatsBroker` | `nats` | Ultra-high-throughput, sub-millisecond cloud microservices |
//! | `KafkaBroker` | `kafka` | Distributed streaming with Apache Kafka / Redpanda (`rskafka`) |

pub mod envelope;
pub mod error;
pub mod in_process;
pub mod rpc;
pub mod traits;

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "nats")]
pub mod nats;

#[cfg(feature = "kafka")]
pub mod kafka;

pub use envelope::{
    BrokerMessage,
    HEADER_CORRELATION_ID,
    HEADER_EVENT_NAME,
    HEADER_REPLY_TO,
    HEADER_SHARD_ID,
};
pub use error::{BrokerError, Result};
pub use in_process::InProcessBroker;
#[cfg(feature = "kafka")]
pub use kafka::KafkaBroker;
#[cfg(feature = "nats")]
pub use nats::NatsBroker;
#[cfg(feature = "redis")]
pub use redis::RedisBroker;
pub use rpc::RpcDispatcher;
pub use traits::{Broker, Subscription, SubscriptionStream};

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct PingPayload {
        nonce: u64,
        msg: String,
    }

    #[tokio::test]
    async fn test_in_process_publish_subscribe() {
        let broker = InProcessBroker::new();
        let topic = "discord.events.guild_create";

        let mut sub = broker.subscribe(topic).await.expect("subscribe failed");

        let payload = PingPayload {
            nonce: 12345,
            msg: "Guild Created".into(),
        };

        let msg = BrokerMessage::from_json(topic, &payload)
            .expect("from_json failed")
            .with_shard(7)
            .with_event_name("GUILD_CREATE");

        broker.publish_message(&msg).await.expect("publish failed");

        let received = sub.next_message().await.expect("received message");
        assert_eq!(received.topic, topic);
        assert_eq!(received.shard_id(), Some(7));
        assert_eq!(received.event_name(), Some("GUILD_CREATE"));

        let decoded: PingPayload = received.json().expect("decode json");
        assert_eq!(decoded, payload);
    }

    #[tokio::test]
    async fn test_in_process_rpc_roundtrip() {
        let broker = InProcessBroker::new();
        let topic = "rpc.shard.latency";

        let mut responder_sub = broker.subscribe(topic).await.expect("subscribe responder");
        let responder_broker = broker.clone();

        // Spawn a mock responder
        tokio::spawn(async move {
            if let Some(req) = responder_sub.next_message().await {
                // Reply with 42 ms latency
                responder_broker.reply(&req, b"42ms").await.expect("reply failed");
            }
        });

        // Caller sends RPC request and awaits response
        let resp = broker
            .request(topic, b"get_latency", Duration::from_secs(2))
            .await
            .expect("rpc request failed");

        assert_eq!(&resp, b"42ms");
    }

    #[tokio::test]
    async fn test_in_process_rpc_timeout() {
        let broker = InProcessBroker::new();
        let topic = "rpc.unresponsive.shard";

        // Nobody responds to this topic
        let result = broker.request(topic, b"ping", Duration::from_millis(50)).await;

        assert!(matches!(result, Err(BrokerError::Timeout(_))));
    }

    #[tokio::test]
    async fn test_rpc_dispatcher() {
        let broker = InProcessBroker::new();
        let topic = "rpc.member.kick";

        let dispatcher = RpcDispatcher::new(broker.clone());
        dispatcher
            .register(topic, |msg| async move {
                let text = String::from_utf8_lossy(&msg.payload);
                Ok(format!("Kicked user: {text}").into_bytes())
            })
            .await;

        let _listener = dispatcher.listen(topic).await.expect("listen failed");

        // Give the listener task a millisecond to yield
        tokio::task::yield_now().await;

        let reply =
            broker.request(topic, b"user_1001", Duration::from_secs(2)).await.expect("call failed");

        assert_eq!(String::from_utf8_lossy(&reply), "Kicked user: user_1001");
    }
}
