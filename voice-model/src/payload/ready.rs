use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// RTP server's connection offer and supported encryption modes.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Ready {
    /// IP address of the call's allocated RTP server.
    pub ip: IpAddr,
    /// Set of voice encryption modes offered by the server.
    pub modes: Vec<String>,
    /// Destination port on the call's allocated RTP server.
    pub port: u16,
    /// RTP synchronisation source assigned by the server to the client.
    pub ssrc: u32,
    /// Streams which are scheduled or active
    pub streams: Vec<StreamItem>,
    /// Experiments, who the fuck needs experiments...
    pub experiments: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StreamItem {
    #[serde(rename = "type")]
    pub kind: String,
    pub ssrc: u32,
    pub rtx_ssrc: u32,
    pub rid: String,
    pub quality: u32, // 100, usually
    pub active: bool,
}
