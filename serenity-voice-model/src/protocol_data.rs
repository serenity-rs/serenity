use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProtocolData {
    pub address: IpAddr,
    pub mode: String,
    pub port: u16,
}
