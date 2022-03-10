use crate::id::*;
use serde::{Deserialize, Serialize};

/// Used to begin a voice websocket connection.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Identify {
    /// GuildId which the target voice channel belongs to.
    pub server_id: GuildId,
    /// Authentication session received from Discord's main gateway as part of a
    /// `"VOICE_STATE_UPDATE"` message.
    pub session_id: String,
    /// Authentication token received from Discord's main gateway as part of a
    /// `"VOICE_SERVER_UPDATE"` message.
    pub token: String,
    /// UserId of the client who is connecting.
    pub user_id: UserId,
    /// if we are transmitting video
    #[serde(default)]
    pub video: bool,
    /// which streams we are transmitting
    pub streams: Vec<StreamType>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StreamType {
    pub quality: u32, // usually 100
    pub rid: String,  // usually "100"
    #[serde(rename = "type")]
    pub kind: String, // "screen" for screenshares
}
