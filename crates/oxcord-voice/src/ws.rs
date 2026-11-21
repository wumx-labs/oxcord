use std::time::Duration;

use futures::{SinkExt, StreamExt};
use oxcord_voice_model::payload::*;
use oxcord_voice_model::{Event, SpeakingState};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{debug, instrument};
use url::Url;

use crate::error::{Result, VoiceError};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// WebSocket client for the Discord Voice Gateway (v4 / v8).
pub struct VoiceWsClient {
    ws: WsStream,
    heartbeat_interval: Duration,
    last_seq_ack: Option<u64>,
}

impl VoiceWsClient {
    /// Connects to the Voice Gateway at the specified endpoint URL.
    ///
    /// # Errors
    /// Returns [`VoiceError`] if WebSocket connection fails or URL is invalid.
    #[instrument(skip(gateway_url))]
    pub async fn connect(gateway_url: &str) -> Result<Self> {
        let url = Url::parse(gateway_url).map_err(|e| {
            VoiceError::Protocol(format!("invalid voice gateway URL '{gateway_url}': {e}"))
        })?;

        debug!("Connecting to Voice Gateway: {}", url);
        let (ws, _response) = connect_async(url).await?;

        Ok(Self {
            ws,
            heartbeat_interval: Duration::from_millis(41250), // fallback default
            last_seq_ack: None,
        })
    }

    /// Receives the next voice gateway [`Event`] from the server.
    ///
    /// # Errors
    /// Returns [`VoiceError`] on WebSocket disconnection or deserialization error.
    pub async fn recv_event(&mut self) -> Result<Event> {
        loop {
            let msg = self.ws.next().await.ok_or_else(|| {
                VoiceError::ConnectionClosed("Voice Gateway connection closed".into())
            })??;

            match msg {
                Message::Text(text) => {
                    let event = serde_json::from_str::<Event>(&text)?;
                    if let Event::Hello(hello) = &event {
                        self.heartbeat_interval =
                            Duration::from_secs_f64(hello.heartbeat_interval / 1000.0);
                        debug!(
                            "Voice Gateway Hello received; interval: {:?}",
                            self.heartbeat_interval
                        );
                    }
                    return Ok(event);
                },
                Message::Binary(bin) => {
                    let event = serde_json::from_slice::<Event>(&bin)?;
                    return Ok(event);
                },
                Message::Ping(data) => {
                    self.ws.send(Message::Pong(data)).await?;
                },
                Message::Close(frame) => {
                    let reason =
                        frame.map_or("unknown".into(), |f| format!("{}: {}", f.code, f.reason));
                    return Err(VoiceError::ConnectionClosed(reason));
                },
                _ => {},
            }
        }
    }

    /// Receives the next voice gateway [`Event`] with a timeout.
    pub async fn recv_event_timeout(&mut self, dur: Duration) -> Result<Event> {
        timeout(dur, self.recv_event())
            .await
            .map_err(|_| VoiceError::Protocol("timed out waiting for Voice Gateway event".into()))?
    }

    /// Sends a voice gateway event.
    pub async fn send_event(&mut self, event: &Event) -> Result<()> {
        let json = serde_json::to_string(event)?;
        self.ws.send(Message::Text(json)).await?;
        Ok(())
    }

    /// Sends an [`Identify`] event to begin the handshake.
    pub async fn send_identify(&mut self, identify: Identify) -> Result<()> {
        self.send_event(&Event::Identify(identify)).await
    }

    /// Sends a [`SelectProtocol`] event to finalize connection parameters.
    pub async fn send_select_protocol(&mut self, select: SelectProtocol) -> Result<()> {
        self.send_event(&Event::SelectProtocol(select)).await
    }

    /// Sends a [`Speaking`] event to notify the server of the bot's speaking status.
    pub async fn send_speaking(&mut self, ssrc: u32, speaking: bool) -> Result<()> {
        let state = if speaking { SpeakingState::MICROPHONE } else { SpeakingState::empty() };

        self.send_event(&Event::Speaking(Speaking {
            delay: Some(0),
            speaking: state,
            ssrc,
            user_id: None,
        }))
        .await
    }

    /// Sends a [`Heartbeat`] message with the given nonce.
    pub async fn send_heartbeat(&mut self, nonce: u64) -> Result<()> {
        self.send_event(&Event::Heartbeat(Heartbeat {
            nonce,
        }))
        .await
    }

    /// Returns the negotiated heartbeat interval.
    #[must_use]
    pub fn heartbeat_interval(&self) -> Duration {
        self.heartbeat_interval
    }

    /// Returns the last sequence number acknowledged by the server (Voice Gateway v8).
    #[must_use]
    pub fn last_seq_ack(&self) -> Option<u64> {
        self.last_seq_ack
    }

    /// Updates the last sequence number acknowledged by the server.
    pub fn set_last_seq_ack(&mut self, seq: u64) {
        self.last_seq_ack = Some(seq);
    }

    /// Closes the WebSocket connection cleanly.
    pub async fn close(&mut self) -> Result<()> {
        self.ws.close(None).await?;
        Ok(())
    }
}
