use serde::{Deserialize, Serialize};

/// Server's confirmation of a negotiated encryption scheme.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum SessionDescription {
    UDP {
        /// The negotiated encryption mode.
        mode: String,
        /// Key used for encryption of RTP payloads using the chosen mode.
        secret_key: Vec<u8>,
    },
    WebRTC {
        audio_codec: String, // "opus", probably
        video_codec: String, // "vp9" /"vp8", maybe
        media_session_id: String,
        sdp: String, // those sdp options
    },
}
