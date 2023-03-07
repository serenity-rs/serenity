use serde::{Deserialize, Serialize};

use crate::id::UserId;

/// Message indicating that another user has connected to the voice channel.
///
/// Acts as a source of UserId+SSRC identification.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ClientConnect {
    /// SSRC of any audio packets sent by this newly joined user.
    pub audio_ssrc: u32,
    /// ID of the connecting user.
    pub user_id: UserId,
    /// SSRC of any audio packets sent by this newly joined user.
    ///
    /// Bots should not see any packets with this SSRC.
    pub video_ssrc: u32,
}
