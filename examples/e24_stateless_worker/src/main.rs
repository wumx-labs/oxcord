//! # e24 — Stateless Worker Architecture
//!
//! Demonstrates a pure stateless worker process (T4 Scale / Millions of servers):
//!
//! - Connects **only** to the message broker (no WebSocket to Discord).
//! - Consumes events pushed by the Gateway Daemon.
//! - Feeds events into the central `StateEngine` without hitting the REST API.
//! - Can be scaled horizontally to hundreds of container instances seamlessly.
//!
//! ## Running
//! ```sh
//! cargo run -p e24_stateless_worker
//! ```

use std::time::Duration;

use oxcord::broker::{Broker, BrokerMessage, InProcessBroker};
use oxcord_cache::{InMemoryBackend, StateEngine};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct SimulatedGuildCreate {
    id: u64,
    name: String,
    member_count: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "e24_stateless_worker=info".parse().unwrap()),
        )
        .init();

    info!("Starting Stateless Worker Node (T4 Scale Architecture)…");

    // Shared broker bus (In-Process for demonstration, or NATS/Redis in multi-container setups)
    let broker = InProcessBroker::new();
    let state_engine = StateEngine::new(InMemoryBackend::new());

    let mut event_sub = broker.subscribe("discord.events.guild_create").await?;

    info!("[Worker Node] Subscribed to 'discord.events.guild_create'. Ready to process…");

    // Producer simulation task (representing the Gateway Daemon blasting events)
    let gateway_sim = broker.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;

        let payload = SimulatedGuildCreate {
            id: 9988776655,
            name: "OXCORD Developers".into(),
            member_count: 42_000,
        };

        let msg = BrokerMessage::from_json("discord.events.guild_create", &payload)
            .unwrap()
            .with_shard(3)
            .with_event_name("GUILD_CREATE");

        info!(
            "[Gateway Daemon Sim] Emitting GUILD_CREATE for guild '{}' on Shard 3…",
            payload.name
        );
        gateway_sim.publish_message(&msg).await.unwrap();
    });

    // Worker Event Processing Loop
    if let Some(msg) = event_sub.next_message().await {
        let shard = msg.shard_id().unwrap_or(0);
        let guild: SimulatedGuildCreate = msg.json()?;
        info!(
            "[Worker Node] Processed GUILD_CREATE: Guild '{}' (id: {}) with {} members on Shard {}",
            guild.name, guild.id, guild.member_count, shard
        );

        let stats = state_engine.stats().await;
        info!("[Worker Node] Live state engine stats: cached guilds = {}", stats.guilds);
    }

    info!("Stateless worker finished successfully!");
    Ok(())
}
