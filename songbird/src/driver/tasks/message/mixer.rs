use super::{Interconnect, UdpMessage};

use crate::{tracks::Track, Bitrate};
use flume::Sender;
use xsalsa20poly1305::XSalsa20Poly1305 as Cipher;

pub struct MixerConnection {
    pub cipher: Cipher,
    pub udp: Sender<UdpMessage>,
}

impl Drop for MixerConnection {
    fn drop(&mut self) {
        let _ = self.udp.send(UdpMessage::Poison);
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
    Poison,
}
