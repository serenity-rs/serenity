use super::Interconnect;

pub(crate) enum UdpRxMessage {
    ReplaceInterconnect(Interconnect),

    Poison,
}
