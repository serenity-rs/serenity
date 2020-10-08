use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Ready {
	pub ip: IpAddr,
    pub modes: Vec<String>,
    pub port: u16,
    pub ssrc: u32,
}
