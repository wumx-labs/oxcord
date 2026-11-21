use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use async_trait::async_trait;
use futures::channel::mpsc::UnboundedSender as Sender;
use tokio::sync::RwLock;

use crate::gateway::ShardRunnerMessage;
use crate::model::id::{ChannelId, GuildId, UserId};
use crate::model::voice::VoiceState;

/// Interface for any compatible voice plugin.
///
/// This interface covers several oxcord-specific hooks, as well as packet handlers for
/// voice-specific gateway messages.
#[async_trait]
pub trait VoiceGatewayManager: Send + Sync {
    /// Performs initial setup at the start of a connection to Discord.
    ///
    /// This will only occur once, and provides the bot's ID and shard count.
    async fn initialise(&self, shard_count: u32, user_id: UserId);

    /// Handler fired in response to a [`Ready`] event.
    ///
    /// This provides the voice plugin with a channel to send gateway messages to Discord, once per
    /// active shard.
    ///
    /// [`Ready`]: crate::model::event::Event
    async fn register_shard(&self, shard_id: u32, sender: Sender<ShardRunnerMessage>);

    /// Handler fired in response to a disconnect, reconnection, or rebalance.
    ///
    /// This event invalidates the last sender associated with `shard_id`. Unless the bot is fully
    /// disconnecting, this is often followed by a call to [`Self::register_shard`]. Users may wish
    /// to buffer manually any gateway messages sent between these calls.
    async fn deregister_shard(&self, shard_id: u32);

    /// Handler for VOICE_SERVER_UPDATE messages.
    ///
    /// These contain the endpoint and token needed to form a voice connection session.
    async fn server_update(&self, guild_id: GuildId, endpoint: &Option<String>, token: &str);

    /// Handler for VOICE_STATE_UPDATE messages.
    ///
    /// These contain the session ID needed to form a voice connection session.
    async fn state_update(&self, guild_id: GuildId, voice_state: &VoiceState);
}

/// Voice connection state for a guild.
///
/// Tracks the guild's current voice channel, session credentials, endpoint,
/// and mute/deafen settings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VoiceConnectionState {
    /// The guild ID.
    pub guild_id: GuildId,
    /// The connected voice channel ID, or `None` if disconnected.
    pub channel_id: Option<ChannelId>,
    /// The session ID received from `VOICE_STATE_UPDATE`.
    pub session_id: Option<String>,
    /// The endpoint host received from `VOICE_SERVER_UPDATE`.
    pub endpoint: Option<String>,
    /// The authentication token received from `VOICE_SERVER_UPDATE`.
    pub token: Option<String>,
    /// Whether the bot is self-muted.
    pub self_mute: bool,
    /// Whether the bot is self-deafened.
    pub self_deaf: bool,
}

impl VoiceConnectionState {
    /// Returns `true` if all credentials required to connect to the Voice Gateway
    /// (`session_id`, `token`, and `endpoint`) are present.
    #[must_use]
    pub fn is_ready_for_handshake(&self) -> bool {
        self.session_id.is_some()
            && self.token.is_some()
            && self.endpoint.as_ref().is_some_and(|e| !e.is_empty())
    }

    /// Formats the Voice Gateway WebSocket endpoint URL (e.g. `wss://{endpoint}/?v={version}`).
    #[must_use]
    pub fn gateway_url(&self, version: u8) -> Option<String> {
        let endpoint = self.endpoint.as_ref()?;
        let host = endpoint.trim_end_matches(":443");
        Some(format!("wss://{host}/?v={version}"))
    }
}

/// A built-in thread-safe tracker and manager for Discord voice connections.
///
/// Implements [`VoiceGatewayManager`] to receive voice events dispatched from the
/// shard runner, tracks credentials per guild, and provides methods to join,
/// leave, mute, and construct voice gateway handshake payloads.
#[derive(Debug, Default)]
pub struct VoiceStateTracker {
    user_id: Arc<RwLock<Option<UserId>>>,
    shard_count: Arc<AtomicU32>,
    states: Arc<RwLock<HashMap<GuildId, VoiceConnectionState>>>,
    shard_senders: Arc<RwLock<HashMap<u32, Sender<ShardRunnerMessage>>>>,
}

impl VoiceStateTracker {
    /// Creates a new, empty `VoiceStateTracker`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves a snapshot of the current [`VoiceConnectionState`] for a guild, if tracked.
    pub async fn get_state(&self, guild_id: GuildId) -> Option<VoiceConnectionState> {
        self.states.read().await.get(&guild_id).cloned()
    }

    /// Returns `true` if the bot is currently in a voice channel in the specified guild.
    pub async fn is_connected(&self, guild_id: GuildId) -> bool {
        self.states.read().await.get(&guild_id).is_some_and(|s| s.channel_id.is_some())
    }

    /// Returns `true` if all credentials (`session_id`, `token`, `endpoint`) have arrived
    /// for the given guild.
    pub async fn is_ready_for_handshake(&self, guild_id: GuildId) -> bool {
        self.states
            .read()
            .await
            .get(&guild_id)
            .is_some_and(VoiceConnectionState::is_ready_for_handshake)
    }

    /// Returns the bot's user ID, if initialized.
    pub async fn bot_user_id(&self) -> Option<UserId> {
        *self.user_id.read().await
    }

    /// Returns the total number of shards, if initialized.
    #[must_use]
    pub fn shard_count(&self) -> u32 {
        self.shard_count.load(Ordering::Acquire)
    }

    /// Sends a request over the specified shard to join a voice channel.
    ///
    /// # Errors
    /// Returns `Err` if the shard ID is not registered or the channel is closed.
    pub async fn join(
        &self,
        shard_id: u32,
        guild_id: GuildId,
        channel_id: ChannelId,
        self_mute: bool,
        self_deaf: bool,
    ) -> Result<(), &'static str> {
        let senders = self.shard_senders.read().await;
        let sender = senders.get(&shard_id).ok_or("shard sender not registered")?;

        sender
            .unbounded_send(ShardRunnerMessage::UpdateVoiceState {
                guild_id,
                channel_id: Some(channel_id),
                self_mute,
                self_deaf,
            })
            .map_err(|_| "failed to send UpdateVoiceState to shard runner")
    }

    /// Sends a request over the specified shard to leave a voice channel.
    ///
    /// # Errors
    /// Returns `Err` if the shard ID is not registered or the channel is closed.
    pub async fn leave(&self, shard_id: u32, guild_id: GuildId) -> Result<(), &'static str> {
        let senders = self.shard_senders.read().await;
        let sender = senders.get(&shard_id).ok_or("shard sender not registered")?;

        sender
            .unbounded_send(ShardRunnerMessage::UpdateVoiceState {
                guild_id,
                channel_id: None,
                self_mute: false,
                self_deaf: false,
            })
            .map_err(|_| "failed to send UpdateVoiceState to shard runner")
    }

    /// Returns the Voice Gateway WebSocket endpoint URL for a guild.
    pub async fn gateway_url(&self, guild_id: GuildId, version: u8) -> Option<String> {
        let states = self.states.read().await;
        states.get(&guild_id).and_then(|s| s.gateway_url(version))
    }

    /// Constructs an [`Identify`] payload for the Voice Gateway WebSocket connection
    /// if all credentials are ready.
    #[cfg(feature = "voice_model")]
    pub async fn create_identify_payload(
        &self,
        guild_id: GuildId,
    ) -> Option<oxcord_voice_model::payload::Identify> {
        let user_id = (*self.user_id.read().await)?;
        let states = self.states.read().await;
        let state = states.get(&guild_id)?;

        let session_id = state.session_id.as_ref()?.clone();
        let token = state.token.as_ref()?.clone();

        Some(oxcord_voice_model::payload::Identify {
            server_id: oxcord_voice_model::id::GuildId(guild_id.get()),
            user_id: oxcord_voice_model::id::UserId(user_id.get()),
            session_id,
            token,
        })
    }
}

#[async_trait]
impl VoiceGatewayManager for VoiceStateTracker {
    async fn initialise(&self, shard_count: u32, user_id: UserId) {
        self.shard_count.store(shard_count, Ordering::Release);
        *self.user_id.write().await = Some(user_id);
    }

    async fn register_shard(&self, shard_id: u32, sender: Sender<ShardRunnerMessage>) {
        self.shard_senders.write().await.insert(shard_id, sender);
    }

    async fn deregister_shard(&self, shard_id: u32) {
        self.shard_senders.write().await.remove(&shard_id);
    }

    async fn server_update(&self, guild_id: GuildId, endpoint: &Option<String>, token: &str) {
        let mut states = self.states.write().await;
        let entry = states.entry(guild_id).or_insert_with(|| VoiceConnectionState {
            guild_id,
            ..Default::default()
        });

        entry.endpoint.clone_from(endpoint);
        entry.token = Some(token.to_string());
    }

    async fn state_update(&self, guild_id: GuildId, voice_state: &VoiceState) {
        // Only track voice states belonging to this bot
        let bot_user = *self.user_id.read().await;
        if let Some(bot_id) = bot_user {
            if voice_state.user_id != bot_id {
                return;
            }
        }

        let mut states = self.states.write().await;
        let entry = states.entry(guild_id).or_insert_with(|| VoiceConnectionState {
            guild_id,
            ..Default::default()
        });

        entry.channel_id = voice_state.channel_id;
        entry.session_id = Some(voice_state.session_id.clone());
        entry.self_mute = voice_state.self_mute;
        entry.self_deaf = voice_state.self_deaf;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_voice_connection_state_lifecycle() {
        let tracker = VoiceStateTracker::new();
        let bot_id = UserId::new(100);
        let guild_id = GuildId::new(200);
        let channel_id = ChannelId::new(300);

        tracker.initialise(1, bot_id).await;
        assert_eq!(tracker.bot_user_id().await, Some(bot_id));
        assert_eq!(tracker.shard_count(), 1);

        // State update for bot
        let vs_json = serde_json::json!({
            "user_id": bot_id,
            "channel_id": channel_id,
            "session_id": "test_session_id",
            "guild_id": guild_id,
            "deaf": false,
            "mute": false,
            "self_deaf": false,
            "self_mute": false,
            "self_video": false,
            "suppress": false,
        });
        let vs: VoiceState = serde_json::from_value(vs_json).unwrap();

        tracker.state_update(guild_id, &vs).await;
        assert!(!tracker.is_ready_for_handshake(guild_id).await);
        assert!(tracker.is_connected(guild_id).await);

        // Server update
        tracker
            .server_update(guild_id, &Some("voice-eu.discord.media:443".into()), "secret_token")
            .await;

        assert!(tracker.is_ready_for_handshake(guild_id).await);
        assert_eq!(
            tracker.gateway_url(guild_id, 8).await,
            Some("wss://voice-eu.discord.media/?v=8".into())
        );

        #[cfg(feature = "voice_model")]
        {
            let identify = tracker.create_identify_payload(guild_id).await.unwrap();
            assert_eq!(identify.server_id.0, guild_id.get());
            assert_eq!(identify.user_id.0, bot_id.get());
            assert_eq!(identify.session_id, "test_session_id");
            assert_eq!(identify.token, "secret_token");
        }
    }
}
