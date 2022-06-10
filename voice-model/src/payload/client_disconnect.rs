use serde::{Deserialize, Serialize};

use crate::id::UserId;

/// Message indicating that another user has disconnected from the voice channel.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ClientDisconnect {
    /// Id of the disconnected user.
    pub user_id: UserId,
}
