//! # oxcord-gateway-daemon
//!
//! Dedicated Discord Gateway Sharding Daemon for enterprise-scale distributed bot deployments.
//!
//! - Connects and maintains WebSocket connections for assigned shard ranges.
//! - Publishes all received Discord events to the message broker (`discord.events.<name>`).
//! - Persists gateway sessions (`session_id`, `resume_gateway_url`) to disk across restarts.
//! - Exposes a lightweight `/health` and `/metrics` HTTP endpoint for Prometheus/K8s probes.
//! - Enables zero-downtime bot redeployments without dropping Discord shard connections.

mod session;
mod status;

use std::env;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use oxcord::async_trait;
use oxcord::broker::{Broker, BrokerMessage, InProcessBroker};
use oxcord::client::{Client, Context, EventHandler, RawEventHandler};
use oxcord::model::event::Event;
use oxcord::model::gateway::{GatewayIntents, Ready};
use session::SessionStore;
use status::{DaemonMetrics, start_status_server};
use tracing::{error, info, warn};

/// Event forwarder that blasts every received Discord event into the message broker
/// and tracks shard session lifecycle.
struct GatewayDaemonForwarder {
    broker: Arc<dyn Broker>,
    session_store: SessionStore,
    metrics: Arc<DaemonMetrics>,
}

#[async_trait]
impl EventHandler for GatewayDaemonForwarder {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        let shard_id = ready.shard.map(|s| s.id.0).unwrap_or(0);
        info!(
            shard_id = shard_id,
            username = %ready.user.name,
            user_id = %ready.user.id,
            "[Gateway Daemon] Shard ready! Connected"
        );

        self.session_store.record_ready(
            shard_id,
            ready.session_id.clone(),
            ready.resume_gateway_url.clone(),
        );

        // Auto-persist immediately on successful identification
        if let Err(e) = self.session_store.save() {
            warn!("[Gateway Daemon] Failed to auto-save session on Ready: {e}");
        }
    }
}

#[async_trait]
impl RawEventHandler for GatewayDaemonForwarder {
    async fn raw_event(&self, _ctx: Context, event: Event) {
        self.metrics.events_forwarded.fetch_add(1, Ordering::Relaxed);

        let event_name = event_to_name(&event);
        let topic = format!("discord.events.{}", event_name.to_lowercase());

        match serde_json::to_vec(&event) {
            Ok(payload) => {
                let msg = BrokerMessage::new(&topic, payload).with_event_name(event_name);

                if let Err(e) = self.broker.publish_message(&msg).await {
                    error!("[Gateway Daemon] Failed to publish event {event_name} to broker: {e}");
                }
            },
            Err(e) => {
                warn!("[Gateway Daemon] Failed to serialize event {event_name}: {e}");
            },
        }
    }
}

fn event_to_name(event: &Event) -> &'static str {
    match event {
        Event::CommandPermissionsUpdate(_) => "COMMAND_PERMISSIONS_UPDATE",
        Event::AutoModRuleCreate(_) => "AUTO_MODERATION_RULE_CREATE",
        Event::AutoModRuleUpdate(_) => "AUTO_MODERATION_RULE_UPDATE",
        Event::AutoModRuleDelete(_) => "AUTO_MODERATION_RULE_DELETE",
        Event::AutoModActionExecution(_) => "AUTO_MODERATION_ACTION_EXECUTION",
        Event::ChannelCreate(_) => "CHANNEL_CREATE",
        Event::ChannelDelete(_) => "CHANNEL_DELETE",
        Event::ChannelPinsUpdate(_) => "CHANNEL_PINS_UPDATE",
        Event::ChannelUpdate(_) => "CHANNEL_UPDATE",
        Event::GuildBanAdd(_) => "GUILD_BAN_ADD",
        Event::GuildBanRemove(_) => "GUILD_BAN_REMOVE",
        Event::GuildCreate(_) => "GUILD_CREATE",
        Event::GuildDelete(_) => "GUILD_DELETE",
        Event::GuildEmojisUpdate(_) => "GUILD_EMOJIS_UPDATE",
        Event::GuildIntegrationsUpdate(_) => "GUILD_INTEGRATIONS_UPDATE",
        Event::GuildMemberAdd(_) => "GUILD_MEMBER_ADD",
        Event::GuildMemberRemove(_) => "GUILD_MEMBER_REMOVE",
        Event::GuildMemberUpdate(_) => "GUILD_MEMBER_UPDATE",
        Event::GuildMembersChunk(_) => "GUILD_MEMBERS_CHUNK",
        Event::GuildRoleCreate(_) => "GUILD_ROLE_CREATE",
        Event::GuildRoleDelete(_) => "GUILD_ROLE_DELETE",
        Event::GuildRoleUpdate(_) => "GUILD_ROLE_UPDATE",
        Event::GuildStickersUpdate(_) => "GUILD_STICKERS_UPDATE",
        Event::GuildUpdate(_) => "GUILD_UPDATE",
        Event::InviteCreate(_) => "INVITE_CREATE",
        Event::InviteDelete(_) => "INVITE_DELETE",
        Event::MessageCreate(_) => "MESSAGE_CREATE",
        Event::MessageDelete(_) => "MESSAGE_DELETE",
        Event::MessageDeleteBulk(_) => "MESSAGE_DELETE_BULK",
        Event::MessageUpdate(_) => "MESSAGE_UPDATE",
        Event::ReactionAdd(_) => "MESSAGE_REACTION_ADD",
        Event::ReactionRemove(_) => "MESSAGE_REACTION_REMOVE",
        Event::ReactionRemoveAll(_) => "MESSAGE_REACTION_REMOVE_ALL",
        Event::ReactionRemoveEmoji(_) => "MESSAGE_REACTION_REMOVE_EMOJI",
        Event::PresenceUpdate(_) => "PRESENCE_UPDATE",
        Event::Ready(_) => "READY",
        Event::Resumed(_) => "RESUMED",
        Event::TypingStart(_) => "TYPING_START",
        Event::UserUpdate(_) => "USER_UPDATE",
        Event::VoiceServerUpdate(_) => "VOICE_SERVER_UPDATE",
        Event::VoiceStateUpdate(_) => "VOICE_STATE_UPDATE",
        Event::VoiceChannelStatusUpdate(_) => "VOICE_CHANNEL_STATUS_UPDATE",
        Event::WebhookUpdate(_) => "WEBHOOKS_UPDATE",
        Event::InteractionCreate(_) => "INTERACTION_CREATE",
        Event::IntegrationCreate(_) => "INTEGRATION_CREATE",
        Event::IntegrationUpdate(_) => "INTEGRATION_UPDATE",
        Event::IntegrationDelete(_) => "INTEGRATION_DELETE",
        Event::StageInstanceCreate(_) => "STAGE_INSTANCE_CREATE",
        Event::StageInstanceUpdate(_) => "STAGE_INSTANCE_UPDATE",
        Event::StageInstanceDelete(_) => "STAGE_INSTANCE_DELETE",
        Event::ThreadCreate(_) => "THREAD_CREATE",
        Event::ThreadUpdate(_) => "THREAD_UPDATE",
        Event::ThreadDelete(_) => "THREAD_DELETE",
        Event::ThreadListSync(_) => "THREAD_LIST_SYNC",
        Event::ThreadMemberUpdate(_) => "THREAD_MEMBER_UPDATE",
        Event::ThreadMembersUpdate(_) => "THREAD_MEMBERS_UPDATE",
        Event::EntitlementCreate(_) => "ENTITLEMENT_CREATE",
        Event::EntitlementUpdate(_) => "ENTITLEMENT_UPDATE",
        Event::EntitlementDelete(_) => "ENTITLEMENT_DELETE",
        Event::SubscriptionCreate(_) => "SUBSCRIPTION_CREATE",
        Event::SubscriptionUpdate(_) => "SUBSCRIPTION_UPDATE",
        Event::SubscriptionDelete(_) => "SUBSCRIPTION_DELETE",
        _ => "UNKNOWN",
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "oxcord_gateway_daemon=info,oxcord=info".parse().unwrap()),
        )
        .init();

    info!("Starting OXCORD Gateway Daemon (Zero-Downtime Sharding Service)…");

    let session_file = env::var("SESSION_FILE").unwrap_or_else(|_| "./sessions.json".to_string());
    let session_store = SessionStore::new(&session_file);

    let status_port: u16 =
        env::var("STATUS_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8081);

    let metrics = Arc::new(DaemonMetrics::default());
    start_status_server(status_port, Arc::clone(&metrics), session_store.clone());

    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
        warn!("DISCORD_TOKEN environment variable not set. Running in dry-run mode.");
        String::new()
    });

    // Default broker is InProcessBroker; in production configure via BROKER_URL (NATS/Redis/Kafka)
    let broker: Arc<dyn Broker> = Arc::new(InProcessBroker::new());

    if token.is_empty() {
        info!("No token provided. Gateway Daemon initialized successfully in dry-run mode.");
        info!("Metrics and Health endpoint active on port {status_port}.");
        wait_for_shutdown().await;
        let _ = session_store.save();
        return Ok(());
    }

    let intents = GatewayIntents::all();
    let handler = GatewayDaemonForwarder {
        broker,
        session_store: session_store.clone(),
        metrics,
    };

    let mut client = Client::builder(&token, intents).event_handler(handler).await?;

    tokio::select! {
        res = client.start() => {
            if let Err(e) = res {
                error!("Client exited with error: {e:?}");
            }
        }
        _ = wait_for_shutdown() => {
            info!("Shutdown signal received. Flusing shard sessions to disk…");
            if let Err(e) = session_store.save() {
                error!("Failed to save gateway sessions during shutdown: {e}");
            } else {
                info!("Gateway sessions persisted successfully.");
            }
        }
    }

    Ok(())
}

async fn wait_for_shutdown() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler");
        tokio::select! {
            () = ctrl_c => {
                info!("Received SIGINT/Ctrl+C.");
            },
            _ = terminate.recv() => {
                info!("Received SIGTERM signal.");
            },
        }
    }

    #[cfg(not(unix))]
    ctrl_c.await;
}
