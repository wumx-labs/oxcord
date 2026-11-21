use crate::error::{Result, VoiceError};

/// Standard sample rate for Discord Opus audio (48 kHz).
pub const SAMPLE_RATE: u32 = 48_000;

/// Standard number of audio channels (stereo).
pub const CHANNELS: u8 = 2;

/// Standard frame duration in milliseconds.
pub const FRAME_DURATION_MS: u32 = 20;

/// Number of samples per 20ms frame at 48 kHz (960 samples).
pub const SAMPLES_PER_FRAME: u32 = (SAMPLE_RATE / 1000) * FRAME_DURATION_MS;

/// Standard Opus silence frame (3 bytes per RFC 6716).
///
/// Discord requires at least 5 frames of silence to be sent when beginning
/// or stopping audio transmission to prevent audio popping.
pub const OPUS_SILENCE_FRAME: [u8; 3] = [0xF8, 0xFF, 0xFE];

/// Minimum size of an RTP header in bytes.
pub const RTP_HEADER_SIZE: usize = 12;

/// Payload type identifier for Discord Opus audio.
pub const DISCORD_OPUS_PAYLOAD_TYPE: u8 = 120;

/// An RTP packet header according to RFC 3550.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RtpHeader {
    /// RTP version (always 2 for Discord).
    pub version: u8,
    /// Whether padding is present.
    pub padding: bool,
    /// Whether an extension header is present.
    pub extension: bool,
    /// Contributing source count (CSRC).
    pub csrc_count: u8,
    /// Marker bit.
    pub marker: bool,
    /// Payload type (120 for Discord Opus).
    pub payload_type: u8,
    /// Sequence number (increments by 1 for each packet sent).
    pub sequence: u16,
    /// Timestamp (increments by 960 for each 20ms frame).
    pub timestamp: u32,
    /// Synchronization source identifier (SSRC).
    pub ssrc: u32,
}

impl Default for RtpHeader {
    fn default() -> Self {
        Self {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: DISCORD_OPUS_PAYLOAD_TYPE,
            sequence: 0,
            timestamp: 0,
            ssrc: 0,
        }
    }
}

impl RtpHeader {
    /// Creates a new RTP header with the given sequence, timestamp, and SSRC.
    #[must_use]
    pub fn new(sequence: u16, timestamp: u32, ssrc: u32) -> Self {
        Self {
            sequence,
            timestamp,
            ssrc,
            ..Default::default()
        }
    }

    /// Serializes this header into a 12-byte array.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; RTP_HEADER_SIZE] {
        let mut buf = [0u8; RTP_HEADER_SIZE];
        self.write_into(&mut buf);
        buf
    }

    /// Serializes this header into the beginning of `buf`.
    ///
    /// # Panics
    /// Panics if `buf.len() < RTP_HEADER_SIZE`.
    pub fn write_into(&self, buf: &mut [u8]) {
        assert!(buf.len() >= RTP_HEADER_SIZE, "buffer too small for RTP header");

        let b0 = (self.version << 6)
            | ((self.padding as u8) << 5)
            | ((self.extension as u8) << 4)
            | (self.csrc_count & 0x0F);
        let b1 = ((self.marker as u8) << 7) | (self.payload_type & 0x7F);

        buf[0] = b0;
        buf[1] = b1;
        buf[2..4].copy_from_slice(&self.sequence.to_be_bytes());
        buf[4..8].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[8..12].copy_from_slice(&self.ssrc.to_be_bytes());
    }

    /// Parses an RTP header from a byte slice.
    ///
    /// # Errors
    /// Returns [`VoiceError::Protocol`] if the buffer is smaller than 12 bytes.
    pub fn parse(buf: &[u8]) -> Result<Self> {
        if buf.len() < RTP_HEADER_SIZE {
            return Err(VoiceError::Protocol(format!(
                "RTP header too short: expected at least 12 bytes, got {}",
                buf.len()
            )));
        }

        let b0 = buf[0];
        let b1 = buf[1];

        let version = b0 >> 6;
        let padding = (b0 & 0x20) != 0;
        let extension = (b0 & 0x10) != 0;
        let csrc_count = b0 & 0x0F;

        let marker = (b1 & 0x80) != 0;
        let payload_type = b1 & 0x7F;

        let sequence = u16::from_be_bytes([buf[2], buf[3]]);
        let timestamp = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let ssrc = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);

        Ok(Self {
            version,
            padding,
            extension,
            csrc_count,
            marker,
            payload_type,
            sequence,
            timestamp,
            ssrc,
        })
    }
}

/// Helper state machine for sequencing and timestamping outgoing RTP packets.
#[derive(Clone, Debug)]
pub struct RtpPacketizer {
    ssrc: u32,
    sequence: u16,
    timestamp: u32,
}

impl RtpPacketizer {
    /// Creates a new packetizer with the given SSRC. Sequence and timestamp start at 0.
    #[must_use]
    pub fn new(ssrc: u32) -> Self {
        Self {
            ssrc,
            sequence: 0,
            timestamp: 0,
        }
    }

    /// Produces the next [`RtpHeader`] and advances sequence and timestamp counters.
    pub fn next_header(&mut self) -> RtpHeader {
        let header = RtpHeader::new(self.sequence, self.timestamp, self.ssrc);
        self.sequence = self.sequence.wrapping_add(1);
        self.timestamp = self.timestamp.wrapping_add(SAMPLES_PER_FRAME);
        header
    }

    /// Resets the sequence and timestamp counters.
    pub fn reset(&mut self) {
        self.sequence = 0;
        self.timestamp = 0;
    }

    /// Returns the current SSRC.
    #[must_use]
    pub fn ssrc(&self) -> u32 {
        self.ssrc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_header_serialization_roundtrip() {
        let header = RtpHeader::new(42, 9600, 0xCAFE_BABE);
        let bytes = header.to_bytes();

        assert_eq!(bytes[0], 0x80); // V=2
        assert_eq!(bytes[1], 120); // Opus PT
        assert_eq!(&bytes[2..4], &42u16.to_be_bytes());
        assert_eq!(&bytes[4..8], &9600u32.to_be_bytes());
        assert_eq!(&bytes[8..12], &0xCAFE_BABEu32.to_be_bytes());

        let parsed = RtpHeader::parse(&bytes).unwrap();
        assert_eq!(header, parsed);
    }

    #[test]
    fn test_rtp_packetizer_progression() {
        let mut packetizer = RtpPacketizer::new(12345);

        let h0 = packetizer.next_header();
        assert_eq!(h0.sequence, 0);
        assert_eq!(h0.timestamp, 0);

        let h1 = packetizer.next_header();
        assert_eq!(h1.sequence, 1);
        assert_eq!(h1.timestamp, 960);

        let h2 = packetizer.next_header();
        assert_eq!(h2.sequence, 2);
        assert_eq!(h2.timestamp, 1920);
    }
}
