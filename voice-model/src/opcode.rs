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
    /// Video stream information.
    Video = 10,
    /// List of user IDs in channel.
    ClientsConnect = 11,
    /// Message indicating that another user has connected to the voice channel.
    ClientConnect = 12,
    /// Message indicating that another user has disconnected from the voice channel.
    ClientDisconnect = 13,
    /// Media Sink wants (internal).
    MediaSinkWants = 14,
    /// Voice backend version (internal).
    VoiceBackendVersion = 15,
    /// Channel options update (internal).
    ChannelOptionsUpdate = 16,
    /// Flags update (muted, deafened, etc.).
    Flags = 18,
    /// Platform information.
    Platform = 20,
    /// DAVE: Prepares for a protocol transition.
    DavePrepareTransition = 21,
    /// DAVE: Executes a prepared protocol transition.
    DaveExecuteTransition = 22,
    /// DAVE: Signals the bot is ready for group operations.
    DaveTransitionReady = 23,
    /// DAVE: Notifies of an upcoming epoch change.
    DavePrepareEpoch = 24,
    /// DAVE: Provides the external sender package for MLS group initialization.
    DaveMlsExternalSender = 25,
    /// DAVE: Sends the key package for MLS group participation.
    DaveMlsKeyPackage = 26,
    /// DAVE: Provides proposals for group member changes (add/remove).
    DaveMlsProposals = 27,
    /// DAVE: Provides a commit with optional welcome for group transitions.
    DaveMlsCommitWelcome = 28,
    /// DAVE: Announces a commit for group transition.
    DaveMlsAnnounceCommitTransition = 29,
    /// DAVE: Provides the welcome message for new members.
    DaveMlsWelcome = 30,
    /// DAVE: Reports an invalid commit or welcome message.
    DaveMlsInvalidCommitWelcome = 31,
}
