use crate::protocol_data::ProtocolData;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SelectProtocol {
	pub data: ProtocolData,
    pub protocol: String,
}
