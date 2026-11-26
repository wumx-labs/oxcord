//! Example demonstrating OXCORD's native Voice Gateway (v8), Voice State Tracking,
//! and RTP audio transmission pipeline.

use std::env;
use std::sync::Arc;

use oxcord::gateway::VoiceStateTracker;
use oxcord::model::channel::Message;
use oxcord::model::id::{ChannelId, GuildId, UserId};
use oxcord::ops::MessageExt;
use oxcord::prelude::*;

struct Handler {
    voice_tracker: Arc<VoiceStateTracker>,
}

#[oxcord::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let Some(guild_id) = msg.guild_id else {
            return;
        };

        let content = msg.content.trim();

        if content == "~join" {
            // Find the voice channel the message author is currently in
            let channel_opt = find_author_voice_channel(&ctx, guild_id, msg.author.id).await;

            match channel_opt {
                Some(channel_id) => {
                    ctx.join_voice_channel(guild_id, channel_id, false, false);
                    let _ = msg
                        .reply(
                            &ctx,
                            format!("Joining voice channel <#{channel_id}> and initializing Voice Gateway v8 handshake..."),
                        )
                        .await;
                },
                None => {
                    let _ = msg.reply(&ctx, "You must be in a voice channel first!").await;
                },
            }
        } else if content == "~leave" {
            ctx.leave_voice_channel(guild_id);
            let _ = msg.reply(&ctx, "Left the voice channel.").await;
        } else if content == "~status" {
            let state = self.voice_tracker.get_state(guild_id).await;
            match state {
                Some(st) => {
                    let ready = st.is_ready_for_handshake();
                    let url = st.gateway_url(8).unwrap_or_else(|| "N/A".into());
                    let endpoint = st.endpoint.unwrap_or_else(|| "N/A".into());
                    let has_session = st.session_id.is_some();
                    let has_token = st.token.is_some();

                    let text = format!(
                        "**Voice Connection Status (v8)**\n\
                        - Ready for handshake: `{ready}`\n\
                        - Endpoint: `{endpoint}`\n\
                        - Gateway URL: `{url}`\n\
                        - Session ID present: `{has_session}`\n\
                        - Token present: `{has_token}`\n\
                        - Self Mute: `{}` | Self Deaf: `{}`",
                        st.self_mute, st.self_deaf
                    );
                    let _ = msg.reply(&ctx, text).await;
                },
                None => {
                    let _ = msg.reply(&ctx, "No voice state tracked for this guild.").await;
                },
            }
        }
    }
}

async fn find_author_voice_channel(
    ctx: &Context,
    guild_id: GuildId,
    user_id: UserId,
) -> Option<ChannelId> {
    ctx.cache
        .guild(guild_id)
        .and_then(|guild| guild.voice_states.get(&user_id).and_then(|vs| vs.channel_id))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
        eprintln!("DISCORD_TOKEN not set. Exiting.");
        std::process::exit(0);
    });

    let voice_tracker = Arc::new(VoiceStateTracker::new());

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let handler = Handler {
        voice_tracker: Arc::clone(&voice_tracker),
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .voice_manager_arc(voice_tracker)
        .await?;

    println!("Voice bot started! Send ~join, ~leave, or ~status in a server.");
    client.start().await?;

    Ok(())
}
