use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Resume {
    pub server_id: String,
    pub session_id: String,
    pub token: String,
}
