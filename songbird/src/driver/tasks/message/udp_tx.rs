pub enum UdpTxMessage {
    Packet(Vec<u8>), // FIXME: do something cheaper.
    Poison,
}
