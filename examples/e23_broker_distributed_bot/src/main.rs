//! # e23 — Distributed Broker Bot
//!
//! Demonstrates the OXCORD Message Broker architecture for enterprise-scale bots:
//!
//! - **Gateway Node Task**: Publishes incoming Discord events to broker topics.
//! - **Worker Node Task**: Subscribes to events and processes business logic without holding
//!   WebSocket connections.
//! - **Cross-Shard RPC**: Worker calls an RPC topic to query shard latency, and the Gateway
//!   responds automatically.
//!
//! ## Running
//! ```sh
//! cargo run --example e23_broker_distributed_bot
//! ```

use std::time::Duration;

use oxcord::broker::{Broker, BrokerMessage, InProcessBroker, RpcDispatcher};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct SimulatedMessageCreate {
    channel_id: u64,
    author: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(
            |_| "e23_broker_distributed_bot=info,oxcord_broker=info".parse().unwrap(),
        ))
        .init();

    info!("Initializing distributed bot cluster using InProcessBroker…");

    // In a multi-machine setup, replace InProcessBroker with NatsBroker::connect(...) or
    // RedisBroker::new(...)
    let broker = InProcessBroker::new();

    // -----------------------------------------------------------------------
    // Node 1: Worker Node (Command / Logic Processor)
    // -----------------------------------------------------------------------
    let worker_broker = broker.clone();
    let worker_handle = tokio::spawn(async move {
        info!("[Worker Node] Subscribing to topic: discord.events.message_create");
        let mut sub = worker_broker
            .subscribe("discord.events.message_create")
            .await
            .expect("worker subscribe failed");

        // Process incoming events
        while let Some(msg) = sub.next_message().await {
            let shard_id = msg.shard_id().unwrap_or(0);
            if let Ok(event) = msg.json::<SimulatedMessageCreate>() {
                info!(
                    "[Worker Node] Received Event from Shard {}: User '{}' said '{}' in channel {}",
                    shard_id, event.author, event.content, event.channel_id
                );
            }
        }
    });

    // -----------------------------------------------------------------------
    // Node 2: Gateway Node (Handles WebSocket and Shard State)
    // -----------------------------------------------------------------------
    let gateway_broker = broker.clone();
    let rpc_dispatcher = RpcDispatcher::new(gateway_broker.clone());

    // Register an RPC handler so workers can query shard latency
    rpc_dispatcher
        .register("rpc.shard.0.latency", |_req| async move {
            info!("[Gateway Shard 0] Received RPC ping, calculating latency…");
            Ok(b"38ms".to_vec())
        })
        .await;

    let _rpc_listener =
        rpc_dispatcher.listen("rpc.shard.0.latency").await.expect("listen RPC failed");

    // Wait a brief moment for subscriptions to be active
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Simulate Gateway receiving an event from Discord and blasting it to the broker
    info!("[Gateway Node] Simulating incoming MESSAGE_CREATE event on Shard 0…");
    let event_payload = SimulatedMessageCreate {
        channel_id: 1122334455,
        author: "Noxly".into(),
        content: "!ping".into(),
    };

    let event_msg = BrokerMessage::from_json("discord.events.message_create", &event_payload)?
        .with_shard(0)
        .with_event_name("MESSAGE_CREATE");

    gateway_broker.publish_message(&event_msg).await?;

    // Simulate Worker calling Gateway via RPC to get Shard 0 Latency
    tokio::time::sleep(Duration::from_millis(50)).await;
    info!("[Worker Node] Calling Gateway Shard 0 for latency via RPC…");
    let latency_bytes =
        broker.request("rpc.shard.0.latency", b"get_latency", Duration::from_secs(2)).await?;
    let latency_str = String::from_utf8_lossy(&latency_bytes);
    info!("[Worker Node] RPC Response received from Shard 0! Latency: {}", latency_str);

    // Clean shutdown simulation
    tokio::time::sleep(Duration::from_millis(100)).await;
    worker_handle.abort();
    info!("Example completed successfully!");

    Ok(())
}
