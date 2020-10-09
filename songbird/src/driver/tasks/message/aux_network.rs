use crate::ws::WsStream;
use tokio::net::udp::RecvHalf;
use xsalsa20poly1305::XSalsa20Poly1305 as Cipher;

pub enum AuxPacketMessage {
    Udp(RecvHalf),
    UdpCipher(Cipher),
    Ws(Box<WsStream>),

    SetSsrc(u32),
    SetKeepalive(f64),
    Speaking(bool),

    Poison,
}
