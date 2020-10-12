use super::Interconnect;
use crate::ws::WsStream;

pub(crate) enum WsMessage {
    Ws(Box<WsStream>),
    ReplaceInterconnect(Interconnect),
    SetKeepalive(f64),
    Speaking(bool),

    Poison,
}
