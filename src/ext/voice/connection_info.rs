use ::model::UserId;

#[derive(Clone, Debug)]
pub struct ConnectionInfo {
    pub endpoint: String,
    pub session_id: String,
    pub target_id: u64,
    pub token: String,
    pub user_id: UserId,
}
