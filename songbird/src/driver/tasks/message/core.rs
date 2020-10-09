use crate::{
    events::EventData,
    tracks::Track,
    Bitrate,
    ConnectionInfo,
};

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum CoreMessage {
    Connect(ConnectionInfo),
    Disconnect,
    SetTrack(Option<Track>),
    AddTrack(Track),
    SetBitrate(Bitrate),
    AddEvent(EventData),
    Mute(bool),
    Reconnect,
    RebuildInterconnect,
    Poison,
}
