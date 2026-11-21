use serde_repr::{Deserialize_repr, Serialize_repr};

/// An enum representing the [voice opcodes].
///
/// [voice opcodes]: https://docs.discord.com/developers/topics/opcodes-and-status-codes#voice
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize_repr, Serialize_repr,
)]
#[non_exhaustive]
#[repr(u8)]
pub enum Opcode {
    /// Used to begin a voice websocket connection.
    Identify = 0,
    /// Used to select the voice protocol.
    SelectProtocol = 1,
    /// Used to complete the websocket handshake.
    Ready = 2,
    /// Used to keep the websocket connection alive.
    Heartbeat = 3,
    /// Server's confirmation of a negotiated encryption scheme.
    SessionDescription = 4,
    /// Used to indicate which users are speaking, or to inform Discord that the client is now speaking.
    Speaking = 5,
    /// Heartbeat ACK, received by the client to show the server's receipt of a heartbeat.
    HeartbeatAck = 6,
    /// Sent after a disconnect to attempt to resume a session.
    Resume = 7,
    /// Used to determine how often the client must send a heartbeat.
    Hello = 8,
    /// Sent by the server if a session could successfully be resumed.
    Resumed = 9,
    /// Message indicating that another user has connected to the voice channel.
    ClientConnect = 12,
    /// Message indicating that another user has disconnected from the voice channel.
    ClientDisconnect = 13,
    /// Warns clients of an upcoming downgrade/transition away from the DAVE protocol.
    DavePrepareTransition = 21,
    /// Executes a previously announced protocol transition.
    DaveExecuteTransition = 22,
    /// Client acknowledges it is ready for the announced transition.
    DaveTransitionReady = 23,
    /// Signals an upcoming DAVE protocol version change or MLS group adjustment.
    DavePrepareEpoch = 24,
    /// Provides credentials and the public key for the MLS external sender.
    DaveMlsExternalSender = 25,
    /// Sends the client's MLS Key Package for a pending group member.
    DaveMlsKeyPackage = 26,
    /// Transmits MLS Proposals to be appended or revoked.
    DaveMlsProposals = 27,
    /// Sends an MLS Commit (with optional MLS Welcome messages) to the gateway.
    DaveMlsCommitWelcome = 28,
    /// Broadcasts the "winning" MLS Commit to be processed for an upcoming transition.
    DaveMlsAnnounceCommitTransition = 29,
    /// Sends an MLS Welcome message to join the group for an upcoming transition.
    DaveMlsWelcome = 30,
    /// Flags an invalid commit/welcome message and requests a re-add.
    DaveMlsInvalidCommitWelcome = 31,
}
