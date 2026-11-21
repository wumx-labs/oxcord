use std::env::consts;
use std::io::Read;
use std::time::SystemTime;

use flate2::read::ZlibDecoder;
use futures::{SinkExt, StreamExt};
use oxcord_model::constants::{self, Opcode};
use oxcord_model::event::GatewayEvent;
use oxcord_model::gateway::{GatewayIntents, ShardInfo};
use oxcord_model::id::{ChannelId, GuildId, UserId};
use oxcord_model::json::{from_slice, from_str, to_string};
use serde::Serialize;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, WebSocketConfig};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async_with_config};
use tracing::{debug, instrument, trace, warn};
use url::Url;

use super::{ActivityData, ChunkGuildFilter, PresenceData};
use crate::Result;
use crate::error::GatewayError;

#[derive(Serialize)]
struct IdentifyProperties {
    browser: &'static str,
    device: &'static str,
    os: &'static str,
}

#[derive(Serialize)]
struct ChunkGuildMessage<'a> {
    guild_id: GuildId,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<&'a str>,
    limit: u16,
    presences: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_ids: Option<Vec<UserId>>,
    nonce: &'a str,
}

#[derive(Serialize)]
struct PresenceUpdateMessage<'a> {
    afk: bool,
    status: &'a str,
    since: SystemTime,
    activities: &'a [&'a ActivityData],
}

#[derive(Serialize)]
#[serde(untagged)]
enum WebSocketMessageData<'a> {
    Heartbeat(Option<u64>),
    ChunkGuild(ChunkGuildMessage<'a>),
    SoundboardSounds {
        guild_ids: &'a [GuildId],
    },
    Identify {
        compress: bool,
        token: &'a str,
        large_threshold: u8,
        shard: &'a ShardInfo,
        intents: GatewayIntents,
        properties: IdentifyProperties,
        presence: PresenceUpdateMessage<'a>,
    },
    PresenceUpdate(PresenceUpdateMessage<'a>),
    Resume {
        session_id: &'a str,
        token: &'a str,
        seq: u64,
    },
    VoiceStateUpdate {
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        self_mute: bool,
        self_deaf: bool,
    },
}

#[derive(Serialize)]
struct WebSocketMessage<'a> {
    op: Opcode,
    d: WebSocketMessageData<'a>,
}

pub struct WsClient(WebSocketStream<MaybeTlsStream<TcpStream>>);

const TIMEOUT: Duration = Duration::from_millis(500);
const DECOMPRESSION_MULTIPLIER: usize = 3;
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

/// Decompress a binary gateway payload into raw bytes ready for JSON parsing.
///
/// Returns a `Vec<u8>` rather than `String` to avoid an extra UTF-8 validation
/// pass and string allocation — the caller feeds the bytes directly into
/// [`from_slice`] which handles encoding internally.
fn decompress_payload(bytes: &[u8]) -> Result<Vec<u8>> {
    if bytes.len() >= 4 && bytes[0..4] == ZSTD_MAGIC {
        #[cfg(feature = "zstd")]
        {
            zstd::decode_all(bytes).map_err(|why| {
                warn!("Err decompressing zstd bytes: {why:?}");
                debug!("Failing bytes: {bytes:?}");
                why.into()
            })
        }
        #[cfg(not(feature = "zstd"))]
        {
            warn!("Received zstd-compressed payload, but the `zstd` feature is not enabled.");
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "zstd payload received without `zstd` feature enabled",
            )
            .into())
        }
    } else {
        let mut decompressed = Vec::with_capacity(bytes.len() * DECOMPRESSION_MULTIPLIER);
        ZlibDecoder::new(bytes).read_to_end(&mut decompressed).map_err(|why| {
            warn!("Err decompressing zlib bytes: {why:?}");
            debug!("Failing bytes: {bytes:?}");
            why
        })?;
        Ok(decompressed)
    }
}

impl WsClient {
    pub async fn connect(url: Url) -> Result<Self> {
        let config = WebSocketConfig {
            max_message_size: None,
            max_frame_size: None,
            ..Default::default()
        };
        let (stream, _) = connect_async_with_config(url, Some(config), false).await?;

        Ok(Self(stream))
    }

    pub async fn recv_json(&mut self) -> Result<Option<GatewayEvent>> {
        let message = match timeout(TIMEOUT, self.0.next()).await {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(e))) => return Err(e.into()),
            Ok(None) | Err(_) => return Ok(None),
        };

        let value = match message {
            Message::Binary(bytes) => {
                let decompressed = decompress_payload(&bytes)?;

                from_slice(&decompressed).map_err(|why| {
                    warn!("Err deserializing bytes: {why:?}");
                    debug!("Failing bytes: {bytes:?}");
                    why
                })?
            },
            Message::Text(payload) => from_str(&payload).map_err(|why| {
                warn!("Err deserializing text: {why:?}; text: {payload}");
                why
            })?,
            Message::Close(Some(frame)) => {
                return Err(GatewayError::Closed(Some(frame)));
            },
            _ => return Ok(None),
        };

        Ok(Some(value))
    }

    pub async fn send_json(&mut self, value: &impl serde::Serialize) -> Result<()> {
        let message = to_string(value).map(Message::Text)?;

        self.0.send(message).await?;
        Ok(())
    }

    /// Delegate to `StreamExt::next`
    pub async fn next(&mut self) -> Option<std::result::Result<Message, WsError>> {
        self.0.next().await
    }

    /// Delegate to `SinkExt::send`
    pub async fn send(&mut self, message: Message) -> Result<()> {
        self.0.send(message).await?;
        Ok(())
    }

    /// Delegate to `WebSocketStream::close`
    pub async fn close(&mut self, msg: Option<CloseFrame<'_>>) -> Result<()> {
        self.0.close(msg).await?;
        Ok(())
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    pub async fn send_chunk_guild(
        &mut self,
        guild_id: GuildId,
        shard_info: &ShardInfo,
        limit: Option<u16>,
        presences: bool,
        filter: ChunkGuildFilter,
        nonce: Option<&str>,
    ) -> Result<()> {
        debug!("[{:?}] Requesting member chunks", shard_info);

        let (query, user_ids) = match filter {
            ChunkGuildFilter::None => (Some(String::new()), None),
            ChunkGuildFilter::Query(query) => (Some(query), None),
            ChunkGuildFilter::UserIds(user_ids) => (None, Some(user_ids)),
        };

        self.send_json(&WebSocketMessage {
            op: Opcode::RequestGuildMembers,
            d: WebSocketMessageData::ChunkGuild(ChunkGuildMessage {
                guild_id,
                query: query.as_deref(),
                limit: limit.unwrap_or(0),
                presences,
                user_ids,
                nonce: nonce.unwrap_or(""),
            }),
        })
        .await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    pub async fn request_soundboard_sounds(
        &mut self,
        guild_ids: &[GuildId],
        shard_info: &ShardInfo,
    ) -> Result<()> {
        debug!("[{:?}] Requesting soundboard sounds", shard_info);

        self.send_json(&WebSocketMessage {
            op: Opcode::ReqeustSoundboardSounds,
            d: WebSocketMessageData::SoundboardSounds {
                guild_ids,
            },
        })
        .await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[instrument(skip(self))]
    pub async fn send_heartbeat(&mut self, shard_info: &ShardInfo, seq: Option<u64>) -> Result<()> {
        trace!("[{:?}] Sending heartbeat d: {:?}", shard_info, seq);

        self.send_json(&WebSocketMessage {
            op: Opcode::Heartbeat,
            d: WebSocketMessageData::Heartbeat(seq),
        })
        .await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[instrument(skip(self, token))]
    pub async fn send_identify(
        &mut self,
        shard: &ShardInfo,
        token: &str,
        intents: GatewayIntents,
        presence: &PresenceData,
    ) -> Result<()> {
        let activities: Vec<_> = presence.activity.iter().collect();
        let now = SystemTime::now();

        debug!("[{:?}] Identifying", shard);

        let msg = WebSocketMessage {
            op: Opcode::Identify,
            d: WebSocketMessageData::Identify {
                token,
                shard,
                intents,
                compress: true,
                large_threshold: constants::LARGE_THRESHOLD,
                properties: IdentifyProperties {
                    browser: "oxcord",
                    device: "oxcord",
                    os: consts::OS,
                },
                presence: PresenceUpdateMessage {
                    afk: false,
                    since: now,
                    status: presence.status.name(),
                    activities: &activities,
                },
            },
        };

        self.send_json(&msg).await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[instrument(skip(self))]
    pub async fn send_presence_update(
        &mut self,
        shard_info: &ShardInfo,
        presence: &PresenceData,
    ) -> Result<()> {
        let activities: Vec<_> = presence.activity.iter().collect();
        let now = SystemTime::now();

        debug!("[{:?}] Sending presence update", shard_info);

        self.send_json(&WebSocketMessage {
            op: Opcode::PresenceUpdate,
            d: WebSocketMessageData::PresenceUpdate(PresenceUpdateMessage {
                afk: false,
                since: now,
                status: presence.status.name(),
                activities: &activities,
            }),
        })
        .await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[instrument(skip(self, token))]
    pub async fn send_resume(
        &mut self,
        shard_info: &ShardInfo,
        session_id: &str,
        seq: u64,
        token: &str,
    ) -> Result<()> {
        debug!("[{:?}] Sending resume; seq: {}", shard_info, seq);

        self.send_json(&WebSocketMessage {
            op: Opcode::Resume,
            d: WebSocketMessageData::Resume {
                session_id,
                token,
                seq,
            },
        })
        .await
    }

    /// Sends a voice state update over the gateway WebSocket (Opcode 4).
    ///
    /// Used to connect to, move between, or disconnect from voice channels,
    /// as well as to update self-mute and self-deaf status.
    ///
    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[instrument(skip(self))]
    pub async fn send_voice_state_update(
        &mut self,
        shard_info: &ShardInfo,
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        self_mute: bool,
        self_deaf: bool,
    ) -> Result<()> {
        debug!(
            "[{:?}] Sending voice state update for guild {}: channel={:?}, mute={}, deaf={}",
            shard_info, guild_id, channel_id, self_mute, self_deaf
        );

        self.send_json(&WebSocketMessage {
            op: Opcode::VoiceStateUpdate,
            d: WebSocketMessageData::VoiceStateUpdate {
                guild_id,
                channel_id,
                self_mute,
                self_deaf,
            },
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::ZlibEncoder;

    use super::*;

    #[test]
    fn test_zlib_decompression() {
        let text = r#"{"op":11}"#;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(text.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        let decompressed = decompress_payload(&compressed).unwrap();
        assert_eq!(decompressed, text.as_bytes());
    }

    #[cfg(feature = "zstd")]
    #[test]
    fn test_zstd_decompression() {
        let text = r#"{"op":11,"d":null}"#;
        let compressed = zstd::encode_all(text.as_bytes(), 3).unwrap();
        assert_eq!(&compressed[0..4], &ZSTD_MAGIC);

        let decompressed = decompress_payload(&compressed).unwrap();
        assert_eq!(decompressed, text.as_bytes());
    }

    #[test]
    fn test_voice_state_update_message_serialization() {
        let msg = WebSocketMessage {
            op: Opcode::VoiceStateUpdate,
            d: WebSocketMessageData::VoiceStateUpdate {
                guild_id: GuildId::new(12345),
                channel_id: Some(ChannelId::new(67890)),
                self_mute: false,
                self_deaf: true,
            },
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""op":4"#));
        assert!(json.contains(r#""guild_id":"12345""#));
        assert!(json.contains(r#""channel_id":"67890""#));
        assert!(json.contains(r#""self_mute":false"#));
        assert!(json.contains(r#""self_deaf":true"#));
    }
}
