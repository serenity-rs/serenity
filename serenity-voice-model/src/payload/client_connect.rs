use crate::id::UserId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ClientConnect {
    pub audio_ssrc: u32,
    pub user_id: UserId,
    pub video_ssrc: u32,
}
