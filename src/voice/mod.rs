//! A module for connecting to voice channels.

mod audio;
mod connection;
mod connection_info;
mod constants;
mod dca;
mod error;
pub mod events;
mod handler;
pub mod input;
mod manager;
mod payload;
mod threading;
pub mod tracks;

pub use audiopus::{self as opus, Bitrate};
pub use discortp as packet;
pub use self::{
    audio::AudioReceiver,
    dca::DcaMetadata,
    error::{DcaError, VoiceError},
    events::{Event, EventContext, TrackEvent},
    handler::Handler,
    input::{
        ffmpeg,
        ytdl,
    },
    manager::Manager,
    tracks::create_player,
};

use connection_info::ConnectionInfo;
use events::EventData;
use tracks::Track;

const CRYPTO_MODE: &str = "xsalsa20_poly1305";

#[allow(clippy::large_enum_variant)]
pub(crate) enum Status {
    Connect(ConnectionInfo),
    Disconnect,
    SetReceiver(Option<Box<dyn AudioReceiver>>),
    SetTrack(Option<Track>),
    AddTrack(Track),
    SetBitrate(Bitrate),
    AddEvent(EventData),
}
