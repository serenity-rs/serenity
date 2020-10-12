use serde::{Deserialize, Serialize};

/// Used to determine how often the client must send a heartbeat.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Hello {
    /// Number of milliseconds to wait between sending heartbeat messages.
    pub heartbeat_interval: f64,
}
