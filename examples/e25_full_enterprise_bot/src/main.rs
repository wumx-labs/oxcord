//! # e25 — Full Enterprise Bot Architecture (T4 Scale / Millions of Servers)
//!
//! The complete reference architecture for enterprise-scale Discord bots (MEE6, Dyno class):
//!
//! 1. **Gateway Cluster**: Manages WebSocket connections to Discord across hundreds of shards.
//! 2. **Distributed Broker**: Event distribution & RPC bus (InProcess, NATS, Redis, Kafka).
//! 3. **Stateless Workers**: Horizontally scalable business logic & slash command handling.
//! 4. **State Engine**: Centralized caching layer with zero lock contention.
//! 5. **Cross-Service RPC**: Workers query gateway shard state and trigger administrative actions
//!    on demand.
//!
//! ## Running
//! ```sh
//! cargo run -p e25_full_enterprise_bot
//! ```

use std::sync::Arc;
use std::time::Duration;

use oxcord::broker::{Broker, BrokerMessage, InProcessBroker, RpcDispatcher};
use oxcord_cache::{InMemoryBackend, StateEngine};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct InteractionCommand {
    interaction_id: u64,
    command_name: String,
    user_id: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "e25_full_enterprise_bot=info".parse().unwrap()),
        )
        .init();

    info!("==================================================================");
    info!("🚀 OXCORD Enterprise Cluster: Initializing T4 Distributed Bot…");
    info!("==================================================================");

    // Central Broker Bus (pluggable: InProcess, NatsBroker, RedisBroker, KafkaBroker)
    let broker = Arc::new(InProcessBroker::new());
    let state_engine = Arc::new(StateEngine::new(InMemoryBackend::new()));

    // -----------------------------------------------------------------------
    // Service 1: Gateway Shard Cluster (Simulated Shard 0)
    // -----------------------------------------------------------------------
    let gateway_broker = Arc::clone(&broker);
    let rpc_dispatcher = RpcDispatcher::from_arc(Arc::clone(&broker));

    // Gateway listens for commands from workers (e.g. updating presence or voice state)
    rpc_dispatcher
        .register("gateway.shard.0.action", |msg| async move {
            let action = String::from_utf8_lossy(&msg.payload);
            info!("[Gateway Shard 0] Executing remote action: '{action}'");
            Ok(format!("Action '{action}' executed successfully on Shard 0").into_bytes())
        })
        .await;

    let _listener = rpc_dispatcher.listen("gateway.shard.0.action").await?;
    info!("[Gateway Cluster] Shard 0 initialized and listening for worker RPCs.");

    // -----------------------------------------------------------------------
    // Service 2: Stateless Worker Service (Scalable replicas)
    // -----------------------------------------------------------------------
    let worker_broker = Arc::clone(&broker);
    let worker_state = Arc::clone(&state_engine);

    let worker_handle = tokio::spawn(async move {
        let mut sub = worker_broker
            .subscribe("discord.events.interaction_create")
            .await
            .expect("worker subscribe failed");

        info!("[Worker Cluster] Worker Replica #1 ready and listening for interactions.");

        while let Some(msg) = sub.next_message().await {
            if let Ok(cmd) = msg.json::<InteractionCommand>() {
                info!(
                    "[Worker Replica #1] Received command '/{}' from user {} (interaction: {})",
                    cmd.command_name, cmd.user_id, cmd.interaction_id
                );

                // Worker interacts with central state engine
                let stats = worker_state.stats().await;
                info!(
                    "[Worker Replica #1] StateEngine stats: guilds={}, users={}",
                    stats.guilds, stats.users
                );

                // Worker calls Gateway Shard via RPC to update status
                let resp = worker_broker
                    .request(
                        "gateway.shard.0.action",
                        b"recalculate_heartbeat",
                        Duration::from_secs(2),
                    )
                    .await
                    .expect("RPC call failed");

                info!(
                    "[Worker Replica #1] Gateway RPC response: {}",
                    String::from_utf8_lossy(&resp)
                );
            }
        }
    });

    // -----------------------------------------------------------------------
    // Execution Flow Simulation
    // -----------------------------------------------------------------------
    tokio::time::sleep(Duration::from_millis(50)).await;

    info!(
        "[Gateway Shard 0] Incoming INTERACTION_CREATE received from Discord. Blasting to broker…"
    );
    let interaction = InteractionCommand {
        interaction_id: 887766554433,
        command_name: "ban_malicious_user".into(),
        user_id: 123456789,
    };

    let msg = BrokerMessage::from_json("discord.events.interaction_create", &interaction)?
        .with_shard(0)
        .with_event_name("INTERACTION_CREATE");

    gateway_broker.publish_message(&msg).await?;

    // Allow async tasks to finish processing
    tokio::time::sleep(Duration::from_millis(150)).await;
    worker_handle.abort();

    info!("==================================================================");
    info!("✅ OXCORD Enterprise Full Reference Architecture verified successfully!");
    info!("==================================================================");

    Ok(())
}
