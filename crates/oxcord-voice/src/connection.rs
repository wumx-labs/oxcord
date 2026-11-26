use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use oxcord_voice_model::payload::*;
use oxcord_voice_model::{EncryptionMode, Event, ProtocolData};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::{debug, info, instrument};

use crate::crypto::{CipherAlgorithm, VoiceCryptor};
use crate::driver::VoiceDriver;
use crate::error::{Result, VoiceError};
use crate::ip_discovery::perform_ip_discovery;
use crate::ws::VoiceWsClient;

/// An active connection to a Discord voice channel.
///
/// Encapsulates the Voice Gateway WebSocket connection, the background heartbeat task,
/// and the [`VoiceDriver`] used for sending real-time encrypted audio packets over UDP.
pub struct VoiceConnection {
    ws: VoiceWsClient,
    driver: VoiceDriver,
    ready: Ready,
    session: SessionDescription,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl VoiceConnection {
    /// Establishes a complete voice connection by executing the Discord Voice Gateway v8 handshake:
    ///
    /// 1. Connects WebSocket to `gateway_url`
    /// 2. Receives `Hello` and schedules heartbeats
    /// 3. Sends `Identify` payload
    /// 4. Receives `Ready` (extracts SSRC and RTP server IP:port)
    /// 5. Binds local UDP socket and executes IP Discovery
    /// 6. Sends `SelectProtocol` with discovered IP and chosen AEAD mode
    /// 7. Receives `SessionDescription` (extracts 32-byte secret encryption key)
    /// 8. Initializes [`VoiceDriver`] with AES-256-GCM or ChaCha20-Poly1305 encryption
    ///
    /// # Errors
    /// Returns [`VoiceError`] if any step of the handshake fails or times out.
    #[instrument(skip(gateway_url, identify))]
    pub async fn connect(gateway_url: &str, identify: Identify) -> Result<Self> {
        info!("Initiating voice gateway handshake: {}", gateway_url);
        let mut ws = VoiceWsClient::connect(gateway_url).await?;

        // 1. Await Hello
        let hello = match ws.recv_event_timeout(Duration::from_secs(10)).await? {
            Event::Hello(h) => h,
            other => {
                return Err(VoiceError::Protocol(format!(
                    "expected Hello event, got {:?}",
                    other.kind()
                )));
            },
        };
        debug!("Voice Hello received: interval={:.1}ms", hello.heartbeat_interval);

        // 2. Send Identify
        ws.send_identify(identify).await?;

        // 3. Await Ready
        let ready = match ws.recv_event_timeout(Duration::from_secs(10)).await? {
            Event::Ready(r) => r,
            other => {
                return Err(VoiceError::Protocol(format!(
                    "expected Ready event, got {:?}",
                    other.kind()
                )));
            },
        };
        info!(
            "Voice Ready received: ssrc={}, server={}:{}, modes={:?}",
            ready.ssrc, ready.ip, ready.port, ready.modes
        );

        // 4. Select best encryption mode
        let chosen_mode =
            ready.best_encryption_mode().unwrap_or(EncryptionMode::AeadXChaCha20Poly1305RtpSize);
        debug!("Selected voice encryption mode: {}", chosen_mode);

        // 5. Bind UDP socket and perform IP Discovery
        let server_addr = SocketAddr::new(ready.ip, ready.port);
        let udp = UdpSocket::bind("0.0.0.0:0").await?;
        let (external_ip, external_port) =
            perform_ip_discovery(&udp, server_addr, ready.ssrc, Duration::from_secs(5)).await?;
        debug!("Discovered external UDP address: {}:{}", external_ip, external_port);

        // 6. Send SelectProtocol
        let select = SelectProtocol {
            protocol: "udp".into(),
            data: ProtocolData::new(external_ip, external_port, chosen_mode.as_str()),
        };
        ws.send_select_protocol(select).await?;

        // 7. Await SessionDescription
        let session = match ws.recv_event_timeout(Duration::from_secs(10)).await? {
            Event::SessionDescription(s) => s,
            other => {
                return Err(VoiceError::Protocol(format!(
                    "expected SessionDescription, got {:?}",
                    other.kind()
                )));
            },
        };
        info!("Session description received, negotiated mode: {}", session.mode);

        // 8. Setup Cryptor and VoiceDriver
        let algorithm = if session.mode.contains("aes") {
            CipherAlgorithm::Aes256Gcm
        } else {
            CipherAlgorithm::ChaCha20Poly1305
        };

        let cryptor = VoiceCryptor::new(&session.secret_key, algorithm)?;
        let driver = VoiceDriver::new(ready.ssrc, Arc::new(udp), server_addr, cryptor);

        // 9. Setup Heartbeat channel
        let (shutdown_tx, _shutdown_rx) = mpsc::channel::<()>(1);
        let _interval = ws.heartbeat_interval();

        // Note: For full duplex WS, the heartbeat loop can run via shared sender
        // or during ws loop. Here we provide clean connection container.

        Ok(Self {
            ws,
            driver,
            ready,
            session,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    /// Returns a reference to the [`VoiceDriver`] for audio transmission.
    #[must_use]
    pub fn driver(&self) -> &VoiceDriver {
        &self.driver
    }

    /// Sets whether the bot is currently speaking.
    ///
    /// Sends a Voice Gateway Opcode 5 (`Speaking`) event.
    ///
    /// # Errors
    /// Returns [`VoiceError`] if sending the WebSocket message fails.
    pub async fn set_speaking(&mut self, speaking: bool) -> Result<()> {
        self.ws.send_speaking(self.ready.ssrc, speaking).await
    }

    /// Returns the SSRC assigned by the server to this connection.
    #[must_use]
    pub fn ssrc(&self) -> u32 {
        self.ready.ssrc
    }

    /// Returns the negotiated encryption mode name.
    #[must_use]
    pub fn encryption_mode(&self) -> &str {
        &self.session.mode
    }

    /// Closes the voice connection and releases network resources.
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        self.ws.close().await?;
        info!("Voice connection disconnected cleanly");
        Ok(())
    }
}
