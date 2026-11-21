use ring::aead::{AES_256_GCM, Aad, CHACHA20_POLY1305, LessSafeKey, Nonce, UnboundKey};

use crate::error::{Result, VoiceError};
use crate::rtp::{RTP_HEADER_SIZE, RtpHeader};

/// Size of the trailing nonce counter in bytes for modern AEAD modes.
pub const NONCE_COUNTER_SIZE: usize = 4;

/// Size of the authentication tag in bytes (16 bytes for GCM and Poly1305).
pub const AEAD_TAG_SIZE: usize = 16;

/// Minimum size of an encrypted packet (12-byte header + 16-byte tag + 4-byte counter = 32 bytes).
pub const MIN_ENCRYPTED_PACKET_SIZE: usize = RTP_HEADER_SIZE + AEAD_TAG_SIZE + NONCE_COUNTER_SIZE;

/// Cipher algorithm used for voice encryption.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CipherAlgorithm {
    /// AES-256-GCM with RTP size handling (hardware-accelerated).
    Aes256Gcm,
    /// ChaCha20-Poly1305 with RTP size handling.
    ChaCha20Poly1305,
}

/// Encryption and decryption engine for Discord voice RTP packets.
pub struct VoiceCryptor {
    key: LessSafeKey,
    algorithm: CipherAlgorithm,
    nonce_counter: u32,
}

impl VoiceCryptor {
    /// Creates a new `VoiceCryptor` with the specified secret key and cipher algorithm.
    ///
    /// # Errors
    /// Returns [`VoiceError::Crypto`] if the key length is invalid (must be 32 bytes).
    pub fn new(secret_key: &[u8], algorithm: CipherAlgorithm) -> Result<Self> {
        if secret_key.len() != 32 {
            return Err(VoiceError::Crypto("secret key must be exactly 32 bytes"));
        }

        let ring_algo = match algorithm {
            CipherAlgorithm::Aes256Gcm => &AES_256_GCM,
            CipherAlgorithm::ChaCha20Poly1305 => &CHACHA20_POLY1305,
        };

        let unbound_key = UnboundKey::new(ring_algo, secret_key)
            .map_err(|_| VoiceError::Crypto("failed to create unbound key"))?;
        let key = LessSafeKey::new(unbound_key);

        Ok(Self {
            key,
            algorithm,
            nonce_counter: 0,
        })
    }

    /// Creates an AES-256-GCM cryptor (the preferred, hardware-accelerated mode for Discord).
    pub fn aes256_gcm(secret_key: &[u8]) -> Result<Self> {
        Self::new(secret_key, CipherAlgorithm::Aes256Gcm)
    }

    /// Creates a ChaCha20-Poly1305 cryptor.
    pub fn chacha20_poly1305(secret_key: &[u8]) -> Result<Self> {
        Self::new(secret_key, CipherAlgorithm::ChaCha20Poly1305)
    }

    /// Constructs the 12-byte AEAD nonce from a 32-bit counter.
    ///
    /// Nonce layout: `[counter (4 bytes big-endian), 0, 0, 0, 0, 0, 0, 0, 0]`.
    #[must_use]
    fn construct_nonce(counter: u32) -> Nonce {
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(&counter.to_be_bytes());
        Nonce::assume_unique_for_key(nonce_bytes)
    }

    /// Encrypts an Opus audio payload with the given RTP header into a complete UDP packet.
    ///
    /// # Packet Layout
    /// `[RTP Header (12 bytes)] + [Encrypted Opus Payload + Tag (16 bytes)] + [Nonce Counter (4
    /// bytes)]`
    ///
    /// # Errors
    /// Returns [`VoiceError::Crypto`] if encryption fails.
    pub fn encrypt_rtp_packet(
        &mut self,
        header: &RtpHeader,
        opus_payload: &[u8],
    ) -> Result<Vec<u8>> {
        let counter = self.nonce_counter;
        self.nonce_counter = self.nonce_counter.wrapping_add(1);

        let nonce = Self::construct_nonce(counter);

        // Preallocate exact capacity
        let total_size = RTP_HEADER_SIZE + opus_payload.len() + AEAD_TAG_SIZE + NONCE_COUNTER_SIZE;
        let mut packet = Vec::with_capacity(total_size);

        // 1. Write unencrypted RTP Header (12 bytes)
        let header_bytes = header.to_bytes();
        packet.extend_from_slice(&header_bytes);

        // 2. Encrypt Opus payload with AAD = header
        let mut encrypted_payload = opus_payload.to_vec();
        let aad = Aad::from(&header_bytes);
        self.key
            .seal_in_place_append_tag(nonce, aad, &mut encrypted_payload)
            .map_err(|_| VoiceError::Crypto("failed to encrypt audio packet"))?;

        // 3. Append encrypted payload + authentication tag
        packet.extend_from_slice(&encrypted_payload);

        // 4. Append the 4-byte nonce counter (big-endian) to the end
        packet.extend_from_slice(&counter.to_be_bytes());

        Ok(packet)
    }

    /// Decrypts a received UDP packet, returning the [`RtpHeader`] and the plaintext Opus payload.
    ///
    /// # Errors
    /// Returns [`VoiceError`] if the packet is too short or authentication tag verification fails.
    pub fn decrypt_rtp_packet(&self, packet: &[u8]) -> Result<(RtpHeader, Vec<u8>)> {
        if packet.len() < MIN_ENCRYPTED_PACKET_SIZE {
            return Err(VoiceError::Protocol(format!(
                "packet too short for AEAD: expected >= {} bytes, got {}",
                MIN_ENCRYPTED_PACKET_SIZE,
                packet.len()
            )));
        }

        // 1. Parse RTP Header
        let header = RtpHeader::parse(&packet[..RTP_HEADER_SIZE])?;
        let header_bytes = &packet[..RTP_HEADER_SIZE];

        // 2. Read trailing 4-byte nonce counter
        let counter_start = packet.len() - NONCE_COUNTER_SIZE;
        let counter = u32::from_be_bytes([
            packet[counter_start],
            packet[counter_start + 1],
            packet[counter_start + 2],
            packet[counter_start + 3],
        ]);

        let nonce = Self::construct_nonce(counter);

        // 3. Extract encrypted payload + tag
        let mut in_out = packet[RTP_HEADER_SIZE..counter_start].to_vec();

        // 4. Open in place with AAD = header
        let aad = Aad::from(header_bytes);
        let plaintext = self
            .key
            .open_in_place(nonce, aad, &mut in_out)
            .map_err(|_| VoiceError::Crypto("failed to decrypt or authenticate packet"))?;

        Ok((header, plaintext.to_vec()))
    }

    /// Returns the algorithm in use.
    #[must_use]
    pub fn algorithm(&self) -> CipherAlgorithm {
        self.algorithm
    }

    /// Returns the next nonce counter value.
    #[must_use]
    pub fn nonce_counter(&self) -> u32 {
        self.nonce_counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes256_gcm_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let mut cryptor = VoiceCryptor::aes256_gcm(&key).unwrap();

        let header = RtpHeader::new(100, 9600, 0x1234_5678);
        let opus_data = b"discord opus audio payload test data 1234567890";

        let encrypted = cryptor.encrypt_rtp_packet(&header, opus_data).unwrap();
        assert_eq!(
            encrypted.len(),
            RTP_HEADER_SIZE + opus_data.len() + AEAD_TAG_SIZE + NONCE_COUNTER_SIZE
        );

        let (decrypted_header, decrypted_payload) = cryptor.decrypt_rtp_packet(&encrypted).unwrap();
        assert_eq!(header, decrypted_header);
        assert_eq!(opus_data, decrypted_payload.as_slice());
    }

    #[test]
    fn test_chacha20_poly1305_encrypt_decrypt_roundtrip() {
        let key = [0x77u8; 32];
        let mut cryptor = VoiceCryptor::chacha20_poly1305(&key).unwrap();

        let header = RtpHeader::new(1, 960, 0xABCD_EF01);
        let opus_data = b"opus sample bytes";

        let encrypted = cryptor.encrypt_rtp_packet(&header, opus_data).unwrap();
        let (decrypted_header, decrypted_payload) = cryptor.decrypt_rtp_packet(&encrypted).unwrap();
        assert_eq!(header, decrypted_header);
        assert_eq!(opus_data, decrypted_payload.as_slice());
    }

    #[test]
    fn test_tampered_packet_fails_decryption() {
        let key = [0x11u8; 32];
        let mut cryptor = VoiceCryptor::aes256_gcm(&key).unwrap();

        let header = RtpHeader::new(5, 4800, 0x9999);
        let opus_data = b"secure voice payload";

        let mut encrypted = cryptor.encrypt_rtp_packet(&header, opus_data).unwrap();

        // Tamper with one ciphertext byte
        encrypted[15] ^= 0xFF;

        assert!(cryptor.decrypt_rtp_packet(&encrypted).is_err());
    }
}
