use super::{Interconnect, UdpRxMessage, UdpTxMessage, WsMessage};

use crate::{tracks::Track, Bitrate};
use flume::Sender;
use xsalsa20poly1305::XSalsa20Poly1305 as Cipher;

pub(crate) struct MixerConnection {
    pub cipher: Cipher,
    pub udp_rx: Sender<UdpRxMessage>,
    pub udp_tx: Sender<UdpTxMessage>,
}

impl Drop for MixerConnection {
    fn drop(&mut self) {
        let _ = self.udp_rx.send(UdpRxMessage::Poison);
        let _ = self.udp_tx.send(UdpTxMessage::Poison);
    }
}

pub(crate) enum MixerMessage {
    AddTrack(Track),
    SetTrack(Option<Track>),
    SetBitrate(Bitrate),
    SetMute(bool),
    SetConn(MixerConnection, u32),
    DropConn,
    ReplaceInterconnect(Interconnect),
    RebuildEncoder,

    Ws(Option<Sender<WsMessage>>),
    Poison,
}
