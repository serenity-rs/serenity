use serde_repr::{Serialize_repr, Deserialize_repr};

/// Enum to map voice opcodes.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize_repr)]
#[non_exhaustive]
#[repr(u8)]
pub enum OpCode {
    /// Used to begin a voice websocket connection.
    Identify = 0,
    /// Used to select the voice protocol.
    SelectProtocol = 1,
    /// Used to complete the websocket handshake.
    Ready = 2,
    /// Used to keep the websocket connection alive.
    Heartbeat = 3,
    /// Used to describe the session.
    SessionDescription = 4,
    /// Used to indicate which users are speaking.
    Speaking = 5,
    /// Heartbeat ACK, received by the client to show the server's receipt of a heartbeat.
    HeartbeatAck = 6,
    /// Sent after a disconnect to attempt to resume a session.
    Resume = 7,
    /// Used to determine how often the client must send a heartbeat.
    Hello = 8,
    /// Sent by the server if a session coulkd successfully be resumed.
    Resumed = 9,
    /// Message indicating that another user has connected to the voice channel.
    ClientConnect = 12,
    /// Message indicating that another user has disconnected from the voice channel.
    ClientDisconnect = 13,
}
