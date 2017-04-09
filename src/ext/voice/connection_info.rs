use ::model::{GuildId, UserId};

#[derive(Clone, Debug)]
pub struct ConnectionInfo {
    pub endpoint: String,
    pub guild_id: GuildId,
    pub session_id: String,
    pub token: String,
    pub user_id: UserId,
}
