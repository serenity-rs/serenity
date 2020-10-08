use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SessionDescription {
    pub mode: String,
    pub secret_key: Vec<u8>,
}
