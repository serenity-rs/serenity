use serde::{Deserialize, Serialize};

/// Server's confirmation of a negotiated encryption scheme.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct SessionDescription {
    /// The negotiated encryption mode.
    pub mode: String,
    /// Key used for encryption of RTP payloads using the chosen mode.
    pub secret_key: Vec<u8>,
}
