use crate::{
    driver::{connection::error::Error, Config},
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
    SetConfig(Config),
    Mute(bool),
    Reconnect,
    FullReconnect,
    RebuildInterconnect,
    Poison,
}
