use crate::id::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Identify {
    pub server_id: GuildId,
    pub session_id: String,
    pub token: String,
    pub user_id: UserId,
}
