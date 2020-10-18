use crate::{
    driver::connection::error::Error,
    events::EventData,
    tracks::Track,
    Bitrate,
    ConnectionInfo,
};
use flume::Sender;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum CoreMessage {
    ConnectWithResult(ConnectionInfo, Sender<Result<(), Error>>),
    Disconnect,
    SetTrack(Option<Track>),
    AddTrack(Track),
    SetBitrate(Bitrate),
    AddEvent(EventData),
    Mute(bool),
    Reconnect,
    FullReconnect,
    RebuildInterconnect,
    Poison,
}
