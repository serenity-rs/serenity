pub enum UdpMessage {
    Packet(Vec<u8>), // FIXME: do something cheaper.
    Poison,
}
