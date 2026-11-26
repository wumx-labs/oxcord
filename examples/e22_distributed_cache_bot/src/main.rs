//! # e22 — Distributed Cache Bot
//!
//! Demonstrates the OXCORD `StateEngine` with `InMemoryBackend`:
//!
//! - Processes Discord Gateway events into the `StateEngine`
//! - Queries guild/channel/member state without hitting the REST API
//! - Reports live cache statistics
//!
//! ## Commands (prefix `~`)
//! | Command          | Description                              |
//! |------------------|------------------------------------------|
//! | `~stats`         | Print cache statistics                   |
//! | `~guild`         | Show current guild info from cache       |
//! | `~member <id>`   | Look up a guild member by user ID        |
//! | `~channel <id>`  | Look up a channel by ID                  |
//!
//! ## Running
//! ```sh
//! DISCORD_TOKEN=your_token cargo run --example e22_distributed_cache_bot
//! ```

use std::env;
use std::sync::Arc;

use oxcord::async_trait;
use oxcord::client::{Client, Context, EventHandler};
use oxcord::model::channel::Message;
use oxcord::model::gateway::{GatewayIntents, Ready};
use oxcord::model::id::{ChannelId, UserId};
use oxcord::ops::MessageExt;
use oxcord::prelude::TypeMapKey;
use oxcord_cache::{InMemoryBackend, StateEngine};
use tokio::sync::RwLock;
use tracing::info;

// ---------------------------------------------------------------------------
// Shared bot state
// ---------------------------------------------------------------------------

struct BotState {
    /// Central state engine backed by an in-memory concurrent store.
    state: StateEngine,
}

impl BotState {
    fn new() -> Self {
        Self {
            state: StateEngine::new(InMemoryBackend::new()),
        }
    }
}

struct BotStateKey;
impl TypeMapKey for BotStateKey {
    type Value = Arc<RwLock<BotState>>;
}

// ---------------------------------------------------------------------------
// Event handler
// ---------------------------------------------------------------------------

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Logged in as {} (id: {})", ready.user.name, ready.user.id,);

        // Cache the current user via StateEngine.
        let data = ctx.data.read().await;
        if let Some(bot_state) = data.get::<BotStateKey>() {
            let state = bot_state.read().await;
            // Seed current user manually — no raw_event hook needed.
            state.state.backend().set_current_user(ready.user.clone()).await;
            let stats = state.state.stats().await;
            info!(
                "Initial cache — guilds: {}, channels: {}, users: {}",
                stats.guilds, stats.channels, stats.users,
            );
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let content = msg.content.trim();
        if !content.starts_with('~') {
            return;
        }

        let mut parts = content[1..].splitn(2, ' ');
        let cmd = parts.next().unwrap_or("").trim();
        let arg = parts.next().unwrap_or("").trim();

        match cmd {
            "stats" => handle_stats(&ctx, &msg).await,
            "guild" => handle_guild(&ctx, &msg).await,
            "member" => handle_member(&ctx, &msg, arg).await,
            "channel" => handle_channel(&ctx, &msg, arg).await,
            _ => {},
        }
    }
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

async fn handle_stats(ctx: &Context, msg: &Message) {
    let data = ctx.data.read().await;
    let Some(bot_state) = data.get::<BotStateKey>() else { return };
    let state = bot_state.read().await;
    let s = state.state.stats().await;

    let _ = msg
        .reply(
            ctx,
            format!(
                "📊 **Cache Stats**\n\
                 • Guilds   : `{}`\n\
                 • Channels : `{}`\n\
                 • Users    : `{}`",
                s.guilds, s.channels, s.users,
            ),
        )
        .await;
}

async fn handle_guild(ctx: &Context, msg: &Message) {
    let Some(guild_id) = msg.guild_id else {
        let _ = msg.reply(ctx, "❌ Guild-only command.").await;
        return;
    };

    let data = ctx.data.read().await;
    let Some(bot_state) = data.get::<BotStateKey>() else { return };
    let state = bot_state.read().await;

    match state.state.get_guild(guild_id).await {
        Some(g) => {
            let _ = msg
                .reply(
                    ctx,
                    format!(
                        "🏰 **Guild (from cache)**\n\
                         • Name  : `{}`\n\
                         • ID    : `{}`\n\
                         • Owner : `{}`\n\
                         • Members cached : `{}`",
                        g.name,
                        g.id,
                        g.owner_id,
                        g.members.len(),
                    ),
                )
                .await;
        },
        None => {
            let _ = msg.reply(ctx, "⚠️ Guild not found in cache (not yet received?).").await;
        },
    }
}

async fn handle_member(ctx: &Context, msg: &Message, arg: &str) {
    let Some(guild_id) = msg.guild_id else {
        let _ = msg.reply(ctx, "❌ Guild-only command.").await;
        return;
    };

    let raw_id: u64 = match arg.parse() {
        Ok(v) if v > 0 => v,
        _ => {
            let _ = msg.reply(ctx, "Usage: `~member <user_id>`").await;
            return;
        },
    };
    let user_id = UserId::new(raw_id);

    let data = ctx.data.read().await;
    let Some(bot_state) = data.get::<BotStateKey>() else { return };
    let state = bot_state.read().await;

    match state.state.get_member(guild_id, user_id).await {
        Some(m) => {
            let _ = msg
                .reply(
                    ctx,
                    format!(
                        "👤 **Member (from cache)**\n\
                         • Username : `{}`\n\
                         • Nick     : `{}`\n\
                         • Roles    : `{}`",
                        m.user.name,
                        m.nick.as_deref().unwrap_or("(none)"),
                        m.roles.len(),
                    ),
                )
                .await;
        },
        None => {
            let _ = msg.reply(ctx, format!("⚠️ Member `{raw_id}` not found in cache.")).await;
        },
    }
}

async fn handle_channel(ctx: &Context, msg: &Message, arg: &str) {
    let raw_id: u64 = match arg.parse() {
        Ok(v) if v > 0 => v,
        _ => {
            let _ = msg.reply(ctx, "Usage: `~channel <channel_id>`").await;
            return;
        },
    };
    let channel_id = ChannelId::new(raw_id);

    let data = ctx.data.read().await;
    let Some(bot_state) = data.get::<BotStateKey>() else { return };
    let state = bot_state.read().await;

    match state.state.get_channel(channel_id).await {
        Some(ch) => {
            let _ = msg
                .reply(
                    ctx,
                    format!(
                        "📺 **Channel (from cache)**\n\
                         • Name  : `{}`\n\
                         • ID    : `{}`\n\
                         • Guild : `{}`\n\
                         • Kind  : `{:?}`",
                        ch.name, ch.id, ch.guild_id, ch.kind,
                    ),
                )
                .await;
        },
        None => {
            let _ = msg.reply(ctx, format!("⚠️ Channel `{raw_id}` not found in cache.")).await;
        },
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "e22_distributed_cache_bot=info,oxcord=warn".parse().unwrap()),
        )
        .init();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN env var not set");

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Failed to create client");

    // Register shared BotState in TypeMap.
    {
        let mut data = client.data.write().await;
        data.insert::<BotStateKey>(Arc::new(RwLock::new(BotState::new())));
    }

    info!("Starting e22_distributed_cache_bot…");
    if let Err(e) = client.start().await {
        panic!("Client error: {e:?}");
    }
}
