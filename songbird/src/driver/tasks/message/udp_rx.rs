use super::Interconnect;
use crate::driver::Config;

pub(crate) enum UdpRxMessage {
    SetConfig(Config),
    ReplaceInterconnect(Interconnect),

    Poison,
}
