use std::net::IpAddr;

use serde::{Deserialize, Serialize};

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
}
