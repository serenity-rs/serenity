use serde::{Deserialize, Serialize};

use crate::id::UserId;
use crate::speaking_state::SpeakingState;

/// Used to indicate which users are speaking, or to inform Discord that the client is now speaking.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Speaking {
    /// Function currently unknown.
    ///
    /// Docs suggest setting to `Some(0)` when sending this message as a client.
    pub delay: Option<u32>,
    /// How/whether a user has started/stopped speaking.
    pub speaking: SpeakingState,
    /// RTP synchronisation source of the speaker.
    pub ssrc: u32,
    /// User ID of the speaker, included in messages *received from* the server.
    ///
    /// Used alongside the SSRC to map individual packets to their sender.
    pub user_id: Option<UserId>,
}
