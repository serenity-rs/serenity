use crate::id::UserId;
use serde::{Deserialize, Serialize};

/// Message indicating that another user has connected to the voice channel.
///
/// Acts as a source of UserId+SSRC identification.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ClientConnect {
    /// SSRC of any audio packets sent by this newly joined user.
    pub audio_ssrc: u32,
    /// ID of the connecting user.
    ///
    /// Users will not see any packets with this field.
    #[serde(default, skip_serializing)]
    pub user_id: UserId,
    /// SSRC of any audio packets sent by this newly joined user.
    ///
    /// Bots should not see any packets with this SSRC.
    pub video_ssrc: u32,
    /// RTX SSRC - optional
    ///
    /// Bots should not see any packets with this SSRC.
    pub rtx_ssrc: Option<u32>,
    /// stream information - optional
    ///
    /// Bots should not see any packets with field.
    #[serde(default)]
    pub streams: Vec<Stream>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Stream {
    #[serde(rename = "type")]
    kind: String, // "video", usually
    rid: String, // "100", usually
    ssrc: u32,   // 0 sometimes
    active: bool,
    quality: u32,       // 100, usually
    rtx_ssrc: u32,      // 0 sometimes
    max_bitrate: u32,   // 4000000, usually
    max_framerate: u32, // 30, sometimes
    max_resolution: Resolution,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Resolution {
    #[serde(rename = "type")]
    kind: String, // "fixed", usually
    width: u32,  // e.g. 1280
    height: u32, // e.g. 720
}
