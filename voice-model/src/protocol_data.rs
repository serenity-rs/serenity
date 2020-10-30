use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// The client's response to a connection offer.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProtocolData {
    /// IP address of the client as seen by the server (*e.g.*, after using [IP DIscovery]
    /// for NAT hole-punching).
    ///
    /// [IP Discovery]: https://docs.rs/discortp/discord/struct.IpDiscovery.html
    pub address: IpAddr,
    /// The client's chosen encryption mode (from those offered by the server).
    pub mode: String,
    /// UDP source port of the client as seen by the server, as above.
    pub port: u16,
}
