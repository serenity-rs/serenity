use crate::model::id::{GuildId, UserId};
use std::fmt;

#[derive(Clone)]
pub struct ConnectionInfo {
    pub endpoint: String,
    pub guild_id: GuildId,
    pub session_id: String,
    pub token: String,
    pub user_id: UserId,
}

impl fmt::Debug for ConnectionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionInfo")
            .field("endpoint", &self.endpoint)
            .field("guild_id", &self.guild_id)
            .field("session_id", &self.session_id)
            .field("user_id", &self.user_id)
            .finish()
    }
}
