use serde::{Deserialize, Serialize};

use crate::protocol_data::ProtocolData;

/// Used to select the voice protocol and encryption mechanism.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SelectProtocol {
    /// Client's response to encryption/connection negotiation.
    pub data: ProtocolData,
    /// Transport protocol.
    ///
    /// Currently, `"udp"` is the only known accepted value.
    pub protocol: String,
}
