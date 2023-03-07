use serde::{Deserialize, Serialize};

use crate::id::GuildId;

/// Sent by the client after a disconnect to attempt to resume a session.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Resume {
    /// GuildId which the target voice channel belongs to.
    pub server_id: GuildId,
    /// Authentication session received from Discord's main gateway as part of a
    /// `"VOICE_STATE_UPDATE"` message.
    pub session_id: String,
    /// Authentication token received from Discord's main gateway as part of a
    /// `"VOICE_SERVER_UPDATE"` message.
    pub token: String,
}
