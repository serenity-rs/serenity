use crate::util::json_safe_u64;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct HeartbeatAck {
    #[serde(with = "json_safe_u64")] 
    pub nonce: u64,
}
