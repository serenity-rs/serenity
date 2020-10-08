use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Ready {
	pub ip: IpAddr,
    pub modes: Vec<String>,
    pub port: u16,
    pub ssrc: u32,
}
