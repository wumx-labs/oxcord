use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, instrument, trace};

use crate::crypto::VoiceCryptor;
use crate::error::Result;
use crate::rtp::{FRAME_DURATION_MS, OPUS_SILENCE_FRAME, RtpPacketizer};

/// Real-time audio driver for transmitting encrypted Opus frames over UDP.
#[derive(Clone)]
pub struct VoiceDriver {
    ssrc: u32,
    udp_socket: Arc<UdpSocket>,
    server_addr: SocketAddr,
    cryptor: Arc<Mutex<VoiceCryptor>>,
    packetizer: Arc<Mutex<RtpPacketizer>>,
}

impl VoiceDriver {
    /// Creates a new `VoiceDriver`.
    pub(crate) fn new(
        ssrc: u32,
        udp_socket: Arc<UdpSocket>,
        server_addr: SocketAddr,
        cryptor: VoiceCryptor,
    ) -> Self {
        Self {
            ssrc,
            udp_socket,
            server_addr,
            cryptor: Arc::new(Mutex::new(cryptor)),
            packetizer: Arc::new(Mutex::new(RtpPacketizer::new(ssrc))),
        }
    }

    /// Encrypts and transmits a single Opus audio frame over UDP.
    ///
    /// The frame should contain valid Opus-encoded audio (typically representing 20ms of 48kHz
    /// audio).
    ///
    /// # Errors
    /// Returns [`VoiceError`] if encryption or UDP socket transmission fails.
    #[instrument(skip(self, opus_frame), level = "trace")]
    pub async fn send_audio_frame(&self, opus_frame: &[u8]) -> Result<()> {
        let header = {
            let mut packetizer = self.packetizer.lock().await;
            packetizer.next_header()
        };

        let packet = {
            let mut cryptor = self.cryptor.lock().await;
            cryptor.encrypt_rtp_packet(&header, opus_frame)?
        };

        self.udp_socket.send_to(&packet, self.server_addr).await?;
        trace!("Sent voice frame seq={}: {} bytes", header.sequence, packet.len());
        Ok(())
    }

    /// Sends a sequence of Opus silence frames (`[0xF8, 0xFF, 0xFE]`) to initialize or terminate
    /// transmission.
    ///
    /// Discord requires at least 5 silence frames to avoid audio popping or buffer underrun.
    ///
    /// # Errors
    /// Returns [`VoiceError`] if sending any frame fails.
    pub async fn send_silence_frames(&self, count: usize) -> Result<()> {
        debug!("Sending {} silence frames", count);
        for _ in 0..count {
            self.send_audio_frame(&OPUS_SILENCE_FRAME).await?;
            sleep(Duration::from_millis(FRAME_DURATION_MS as u64)).await;
        }
        Ok(())
    }

    /// Returns the SSRC assigned to this voice stream.
    #[must_use]
    pub fn ssrc(&self) -> u32 {
        self.ssrc
    }

    /// Returns the server destination address.
    #[must_use]
    pub fn server_addr(&self) -> SocketAddr {
        self.server_addr
    }
}
