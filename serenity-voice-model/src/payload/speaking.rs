use crate::{
	id::UserId,
	speaking_state::SpeakingState,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Speaking {
    pub speaking: SpeakingState,
    pub ssrc: u32,
    pub user_id: UserId,
}
