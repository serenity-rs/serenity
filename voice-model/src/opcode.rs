use serde_repr::{Deserialize_repr, Serialize_repr};

/// An enum representing the [voice opcodes].
///
/// [voice opcodes]: https://discord.com/developers/docs/topics/opcodes-and-status-codes#voice
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
}
