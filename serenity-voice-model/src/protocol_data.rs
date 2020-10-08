use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProtocolData {
    pub address: String,
    pub mode: String,
    pub port: u16,
}
