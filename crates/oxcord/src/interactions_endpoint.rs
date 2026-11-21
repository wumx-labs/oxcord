//! Contains tools related to Discord's Interactions Endpoint URL feature.
//!
//! "You can optionally configure an interactions endpoint to receive interactions via HTTP POSTs
//! rather than over Gateway with a bot user."
//!
//! <https://docs.discord.com/developers/tutorials/upgrading-to-application-commands#adding-an-interactions-endpoint-url>
//!
//! See [`Verifier`] and [`InteractionHandler`] for example usage.

use async_trait::async_trait;
use ed25519_dalek::Verifier as _;

#[cfg(feature = "builder")]
use crate::builder::{CreateAutocompleteResponse, CreateInteractionResponse};
use crate::model::application::{
    CommandInteraction,
    ComponentInteraction,
    Interaction,
    ModalInteraction,
    PingInteraction,
};

/// Fast hex parser for fixed-size byte buffers without allocations.
fn parse_hex<const N: usize>(s: &str) -> Option<[u8; N]> {
    if s.len() != N * 2 {
        return None;
    }

    let mut res = [0u8; N];
    let bytes = s.as_bytes();

    for (i, target) in res.iter_mut().enumerate() {
        let hi = decode_hex_nibble(bytes[2 * i])?;
        let lo = decode_hex_nibble(bytes[2 * i + 1])?;
        *target = (hi << 4) | lo;
    }
    Some(res)
}

#[inline]
const fn decode_hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// The byte array couldn't be parsed into a valid cryptographic public key.
#[derive(Debug)]
pub struct InvalidKey(ed25519_dalek::SignatureError);

impl std::fmt::Display for InvalidKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid bot public key: {}", self.0)
    }
}

impl std::error::Error for InvalidKey {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Errors that can occur during signature verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationError {
    /// The signature hex string is malformed or invalid length.
    InvalidSignatureHex,
    /// Cryptographic signature verification failed (forged or altered payload/timestamp).
    InvalidSignature,
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignatureHex => write!(f, "signature is not a valid 64-byte hex string"),
            Self::InvalidSignature => {
                write!(f, "cryptographic ed25519 signature verification failed")
            },
        }
    }
}

impl std::error::Error for VerificationError {}

/// Used to cryptographically verify incoming interactions HTTP requests for authenticity.
///
/// If incoming requests are not verified, Discord will reject the URL for security reasons.
///
/// Features stack-allocated buffer optimization for payloads under 4 KiB, eliminating
/// heap allocations on typical interaction and ping requests.
///
/// ```rust
/// use oxcord::interactions_endpoint::Verifier;
///
/// let verifier =
///     Verifier::new("67c6bd767ca099e79efac9fcce4d2022a63bf7dea780e7f3d813f694c1597089");
///
/// # let http_headers = std::collections::HashMap::from([("X-Signature-Ed25519", ""), ("X-Signature-Timestamp", "")]);
/// # let request_body = &[];
/// let signature = http_headers["X-Signature-Ed25519"];
/// let timestamp = http_headers["X-Signature-Timestamp"];
/// if verifier.verify(signature, timestamp, request_body).is_err() {
///     // Send HTTP 401 Unauthorized response
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Verifier {
    public_key: ed25519_dalek::VerifyingKey,
}

impl Verifier {
    /// Creates a new [`Verifier`] from the given public key hex string.
    ///
    /// # Panics
    /// Panics if the given key is invalid. For a low-level, non-panicking variant, see
    /// [`Self::try_new()`].
    #[must_use]
    pub fn new(public_key: &str) -> Self {
        Self::try_new(parse_hex(public_key).expect("public key must be a 64 digit hex string"))
            .expect("invalid public key")
    }

    /// Creates a new [`Verifier`] from the public key bytes.
    ///
    /// # Errors
    /// [`InvalidKey`] if the key isn't cryptographically valid.
    pub fn try_new(public_key: [u8; 32]) -> Result<Self, InvalidKey> {
        Ok(Self {
            public_key: ed25519_dalek::VerifyingKey::from_bytes(&public_key).map_err(InvalidKey)?,
        })
    }

    /// Verifies a Discord request for authenticity, given the `X-Signature-Ed25519` HTTP header,
    /// `X-Signature-Timestamp` HTTP header, and request body.
    ///
    /// Returns `Ok(())` on success, or `Err(())` on failure for compatibility.
    #[allow(clippy::result_unit_err, clippy::missing_errors_doc)]
    pub fn verify(&self, signature: &str, timestamp: &str, body: &[u8]) -> Result<(), ()> {
        self.verify_fast(signature, timestamp, body).map_err(|_| ())
    }

    /// Verifies a Discord request with detailed error reporting.
    ///
    /// For payloads where `timestamp.len() + body.len() <= 4096`, this executes with
    /// **zero heap allocations** using a stack-allocated buffer.
    ///
    /// # Errors
    /// Returns [`VerificationError`] if the signature hex is invalid or verification fails.
    pub fn verify_fast(
        &self,
        signature: &str,
        timestamp: &str,
        body: &[u8],
    ) -> Result<(), VerificationError> {
        const STACK_BUFFER_LIMIT: usize = 4096;

        let sig_bytes = parse_hex::<64>(signature).ok_or(VerificationError::InvalidSignatureHex)?;
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);

        let ts_bytes = timestamp.as_bytes();
        let total_len = ts_bytes.len() + body.len();

        if total_len <= STACK_BUFFER_LIMIT {
            let mut buf = [0u8; STACK_BUFFER_LIMIT];
            buf[..ts_bytes.len()].copy_from_slice(ts_bytes);
            buf[ts_bytes.len()..total_len].copy_from_slice(body);
            self.public_key
                .verify(&buf[..total_len], &sig)
                .map_err(|_| VerificationError::InvalidSignature)
        } else {
            let mut buf = Vec::with_capacity(total_len);
            buf.extend_from_slice(ts_bytes);
            buf.extend_from_slice(body);
            self.public_key.verify(&buf, &sig).map_err(|_| VerificationError::InvalidSignature)
        }
    }

    /// Verifies a Discord request using a caller-provided reusable scratch buffer for zero
    /// allocations regardless of payload size.
    ///
    /// # Errors
    /// Returns [`VerificationError`] if the signature hex is invalid or verification fails.
    pub fn verify_with_buffer(
        &self,
        signature: &str,
        timestamp: &str,
        body: &[u8],
        scratch: &mut Vec<u8>,
    ) -> Result<(), VerificationError> {
        let sig_bytes = parse_hex::<64>(signature).ok_or(VerificationError::InvalidSignatureHex)?;
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);

        scratch.clear();
        scratch.reserve(timestamp.len() + body.len());
        scratch.extend_from_slice(timestamp.as_bytes());
        scratch.extend_from_slice(body);

        self.public_key.verify(scratch, &sig).map_err(|_| VerificationError::InvalidSignature)
    }
}

/// Asynchronous handler interface for Discord Interactions received over HTTP.
///
/// Enables serverless or Gateway-less bots that handle Slash Commands, Autocomplete,
/// Message Components, and Modals with automatic Ping/Pong verification.
#[cfg(feature = "builder")]
#[async_trait]
pub trait InteractionHandler: Send + Sync {
    /// Handler for Ping interactions sent by Discord to validate the endpoint.
    ///
    /// Default implementation automatically responds with [`CreateInteractionResponse::Pong`].
    async fn on_ping(&self, _ping: &PingInteraction) -> Option<CreateInteractionResponse> {
        Some(CreateInteractionResponse::Pong)
    }

    /// Handler for Application (Slash) Commands.
    async fn on_command(&self, _command: &CommandInteraction) -> Option<CreateInteractionResponse> {
        None
    }

    /// Handler for Autocomplete suggestions.
    async fn on_autocomplete(
        &self,
        _autocomplete: &CommandInteraction,
    ) -> Option<CreateInteractionResponse> {
        Some(CreateInteractionResponse::Autocomplete(CreateAutocompleteResponse::default()))
    }

    /// Handler for Message Components (buttons, select menus).
    async fn on_component(
        &self,
        _component: &ComponentInteraction,
    ) -> Option<CreateInteractionResponse> {
        None
    }

    /// Handler for Modal Submissions.
    async fn on_modal(&self, _modal: &ModalInteraction) -> Option<CreateInteractionResponse> {
        None
    }

    /// Main dispatcher for incoming interactions.
    ///
    /// Delegates to [`Self::on_ping`], [`Self::on_command`], [`Self::on_autocomplete`],
    /// [`Self::on_component`], or [`Self::on_modal`] by default.
    async fn handle(&self, interaction: Interaction) -> Option<CreateInteractionResponse> {
        match interaction {
            Interaction::Ping(ping) => self.on_ping(&ping).await,
            Interaction::Command(command) => self.on_command(&command).await,
            Interaction::Autocomplete(autocomplete) => self.on_autocomplete(&autocomplete).await,
            Interaction::Component(component) => self.on_component(&component).await,
            Interaction::Modal(modal) => self.on_modal(&modal).await,
            _ => None,
        }
    }
}

/// Standardized HTTP response produced by processing an interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionResponseOutput {
    /// HTTP status code (e.g. 200, 204).
    pub status: u16,
    /// Content-Type header value (e.g. `"application/json"`).
    pub content_type: &'static str,
    /// Response body bytes.
    pub body: Vec<u8>,
}

impl InteractionResponseOutput {
    /// Constructs a JSON 200 OK response.
    #[must_use]
    pub fn json(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            content_type: "application/json",
            body,
        }
    }

    /// Constructs an empty 204 No Content response.
    #[must_use]
    pub fn no_content() -> Self {
        Self {
            status: 204,
            content_type: "text/plain",
            body: Vec::new(),
        }
    }
}

/// Errors that can occur when processing an incoming interaction request.
#[derive(Debug)]
pub enum InteractionProcessingError {
    /// Request signature verification failed. The HTTP server should reply with 401 Unauthorized.
    Verification(VerificationError),
    /// Request body could not be parsed as a valid Discord [`Interaction`]. The HTTP server should
    /// reply with 400 Bad Request.
    InvalidJson(serde_json::Error),
    /// Handler response could not be serialized to JSON.
    Serialization(serde_json::Error),
}

impl std::fmt::Display for InteractionProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verification(e) => write!(f, "signature verification error: {e}"),
            Self::InvalidJson(e) => write!(f, "invalid interaction json: {e}"),
            Self::Serialization(e) => write!(f, "failed to serialize response: {e}"),
        }
    }
}

impl std::error::Error for InteractionProcessingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Verification(e) => Some(e),
            Self::InvalidJson(e) | Self::Serialization(e) => Some(e),
        }
    }
}

/// Convenience function to verify, deserialize, handle, and serialize a Discord interaction
/// request over HTTP.
///
/// Compatible with any web framework (tiny_http, Axum, Actix-web, Tower, Warp, etc.).
///
/// # Errors
/// Returns [`InteractionProcessingError`] if signature verification fails (401),
/// JSON deserialization fails (400), or JSON serialization fails (500).
#[cfg(feature = "builder")]
pub async fn process_interaction<H: InteractionHandler>(
    verifier: &Verifier,
    handler: &H,
    signature: &str,
    timestamp: &str,
    body: &[u8],
) -> Result<InteractionResponseOutput, InteractionProcessingError> {
    verifier
        .verify_fast(signature, timestamp, body)
        .map_err(InteractionProcessingError::Verification)?;

    let interaction: Interaction =
        serde_json::from_slice(body).map_err(InteractionProcessingError::InvalidJson)?;

    let response = handler.handle(interaction).await;

    match response {
        Some(resp) => {
            let json_bytes =
                serde_json::to_vec(&resp).map_err(InteractionProcessingError::Serialization)?;
            Ok(InteractionResponseOutput::json(json_bytes))
        },
        None => Ok(InteractionResponseOutput::no_content()),
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::Signer as _;

    use super::*;

    fn test_keys() -> (ed25519_dalek::SigningKey, Verifier) {
        let seed = [42u8; 32];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        let verifier = Verifier::try_new(verifying_key.to_bytes()).unwrap();
        (signing_key, verifier)
    }

    fn sign_message(
        signing_key: &ed25519_dalek::SigningKey,
        timestamp: &str,
        body: &[u8],
    ) -> String {
        let mut msg = Vec::with_capacity(timestamp.len() + body.len());
        msg.extend_from_slice(timestamp.as_bytes());
        msg.extend_from_slice(body);
        let sig = signing_key.sign(&msg);
        let bytes = sig.to_bytes();
        let mut hex = String::with_capacity(128);
        for b in bytes {
            use std::fmt::Write;
            write!(&mut hex, "{b:02x}").unwrap();
        }
        hex
    }

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex::<4>("bf7dea78"), Some([0xBF, 0x7D, 0xEA, 0x78]));
        assert_eq!(parse_hex::<4>("BF7DEA78"), Some([0xBF, 0x7D, 0xEA, 0x78]));
        assert_eq!(parse_hex::<4>("bf7dea7"), None);
        assert_eq!(parse_hex::<4>("bf7dea789"), None);
        assert_eq!(parse_hex::<4>("bf7dea7x"), None);
        assert_eq!(parse_hex(""), Some([]));
        assert_eq!(
            parse_hex("67c6bd767ca099e79efac9fcce4d2022a63bf7dea780e7f3d813f694c1597089"),
            Some([
                0x67, 0xC6, 0xBD, 0x76, 0x7C, 0xA0, 0x99, 0xE7, 0x9E, 0xFA, 0xC9, 0xFC, 0xCE, 0x4D,
                0x20, 0x22, 0xA6, 0x3B, 0xF7, 0xDE, 0xA7, 0x80, 0xE7, 0xF3, 0xD8, 0x13, 0xF6, 0x94,
                0xC1, 0x59, 0x70, 0x89
            ])
        );
    }

    #[test]
    fn test_verifier_signature_verification() {
        let (signing_key, verifier) = test_keys();
        let timestamp = "1672531199";
        let body = b"{\"type\":1}";

        let signature = sign_message(&signing_key, timestamp, body);

        // Valid signature passes
        assert!(verifier.verify(&signature, timestamp, body).is_ok());
        assert!(verifier.verify_fast(&signature, timestamp, body).is_ok());

        // Altered body fails
        let altered_body = b"{\"type\":2}";
        assert!(verifier.verify(&signature, timestamp, altered_body).is_err());
        assert_eq!(
            verifier.verify_fast(&signature, timestamp, altered_body),
            Err(VerificationError::InvalidSignature)
        );

        // Altered timestamp fails
        let altered_ts = "1672531200";
        assert!(verifier.verify(&signature, altered_ts, body).is_err());

        // Corrupted signature hex fails
        assert_eq!(
            verifier.verify_fast("invalid_hex", timestamp, body),
            Err(VerificationError::InvalidSignatureHex)
        );
    }

    #[test]
    fn test_verifier_large_payload() {
        let (signing_key, verifier) = test_keys();
        let timestamp = "1672531199";
        // Create payload larger than stack buffer (5000 bytes > 4096)
        let body = vec![b'A'; 5000];

        let signature = sign_message(&signing_key, timestamp, &body);

        assert!(verifier.verify_fast(&signature, timestamp, &body).is_ok());

        // Test with scratch buffer
        let mut scratch = Vec::new();
        assert!(verifier.verify_with_buffer(&signature, timestamp, &body, &mut scratch).is_ok());
    }

    #[cfg(feature = "builder")]
    struct TestBotHandler;

    #[cfg(feature = "builder")]
    #[async_trait]
    impl InteractionHandler for TestBotHandler {
        async fn on_command(
            &self,
            _command: &CommandInteraction,
        ) -> Option<CreateInteractionResponse> {
            use crate::builder::CreateInteractionResponseMessage;
            Some(CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Hello from TestBot!"),
            ))
        }

        async fn on_autocomplete(
            &self,
            _autocomplete: &CommandInteraction,
        ) -> Option<CreateInteractionResponse> {
            Some(CreateInteractionResponse::Autocomplete(
                CreateAutocompleteResponse::default()
                    .set_choices(vec!["Option 1".into(), "Option 2".into()]),
            ))
        }
    }

    #[cfg(feature = "builder")]
    #[tokio::test]
    async fn test_process_interaction_ping_pong() {
        let (signing_key, verifier) = test_keys();
        let handler = TestBotHandler;

        let timestamp = "1672531199";
        let ping_body = serde_json::json!({
            "id": "1234567890",
            "type": 1,
            "version": 1,
            "token": "interaction_token",
            "application_id": "9876543210"
        })
        .to_string();

        let signature = sign_message(&signing_key, timestamp, ping_body.as_bytes());

        let result =
            process_interaction(&verifier, &handler, &signature, timestamp, ping_body.as_bytes())
                .await
                .unwrap();

        assert_eq!(result.status, 200);
        assert_eq!(result.content_type, "application/json");

        let resp_json: serde_json::Value = serde_json::from_slice(&result.body).unwrap();
        // Type 1 is Pong
        assert_eq!(resp_json.get("type").and_then(|v| v.as_u64()), Some(1));
    }

    #[cfg(feature = "builder")]
    #[tokio::test]
    async fn test_process_interaction_invalid_signature_rejected() {
        let (_, verifier) = test_keys();
        let handler = TestBotHandler;

        let timestamp = "1672531199";
        let body = b"{\"type\":1}";
        let bad_sig = "0".repeat(128);

        let err =
            process_interaction(&verifier, &handler, &bad_sig, timestamp, body).await.unwrap_err();

        match err {
            InteractionProcessingError::Verification(VerificationError::InvalidSignature) => {},
            other => panic!("expected InvalidSignature error, got {other:?}"),
        }
    }

    #[cfg(feature = "builder")]
    #[tokio::test]
    async fn test_process_interaction_command() {
        let (signing_key, verifier) = test_keys();
        let handler = TestBotHandler;

        let timestamp = "1672531199";
        let cmd_body = serde_json::json!({
            "id": "1234567890",
            "type": 2,
            "version": 1,
            "token": "interaction_token",
            "application_id": "9876543210",
            "channel_id": "111222333",
            "locale": "en-US",
            "entitlements": [],
            "attachment_size_limit": 25000000,
            "data": {
                "id": "555666777",
                "name": "ping",
                "type": 1,
                "options": [],
                "resolved": {}
            }
        })
        .to_string();

        let signature = sign_message(&signing_key, timestamp, cmd_body.as_bytes());

        let result =
            process_interaction(&verifier, &handler, &signature, timestamp, cmd_body.as_bytes())
                .await
                .unwrap();

        assert_eq!(result.status, 200);
        let resp_json: serde_json::Value = serde_json::from_slice(&result.body).unwrap();
        // Type 4 is ChannelMessageWithSource
        assert_eq!(resp_json.get("type").and_then(|v| v.as_u64()), Some(4));
        assert_eq!(resp_json["data"]["content"].as_str(), Some("Hello from TestBot!"));
    }

    #[cfg(feature = "builder")]
    #[tokio::test]
    async fn test_process_interaction_autocomplete() {
        let (signing_key, verifier) = test_keys();
        let handler = TestBotHandler;

        let timestamp = "1672531199";
        let auto_body = serde_json::json!({
            "id": "1234567890",
            "type": 4,
            "version": 1,
            "token": "interaction_token",
            "application_id": "9876543210",
            "channel_id": "111222333",
            "locale": "en-US",
            "entitlements": [],
            "attachment_size_limit": 25000000,
            "data": {
                "id": "555666777",
                "name": "search",
                "type": 1,
                "options": [
                    {
                        "name": "query",
                        "type": 3,
                        "value": "opt",
                        "focused": true
                    }
                ],
                "resolved": {}
            }
        })
        .to_string();

        let signature = sign_message(&signing_key, timestamp, auto_body.as_bytes());

        let result =
            process_interaction(&verifier, &handler, &signature, timestamp, auto_body.as_bytes())
                .await
                .unwrap();

        assert_eq!(result.status, 200);
        let resp_json: serde_json::Value = serde_json::from_slice(&result.body).unwrap();
        // Type 8 is ApplicationCommandAutocompleteResult
        assert_eq!(resp_json.get("type").and_then(|v| v.as_u64()), Some(8));
        assert_eq!(resp_json["data"]["choices"].as_array().unwrap().len(), 2);
    }
}
