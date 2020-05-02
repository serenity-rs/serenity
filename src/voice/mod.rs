//! A module for connecting to voice channels.

mod audio;
mod connection;
mod connection_info;
mod constants;
mod dca;
mod error;
mod events;
mod handler;
pub mod input;
mod manager;
mod payload;
mod threading;
pub mod tracks;

pub use audiopus::Bitrate;

pub use self::{
    audio::{
        Audio,
        AudioCommand,
        AudioFn,
        AudioHandle,
        AudioQueryResult,
        AudioReceiver,
        AudioResult,
        AudioState,
        AudioType,
        BlockingAudioQueryResult,
        LoopState,
        PlayMode,
        // ReadSeek,
        TrackQueue,
        create_player,
    },
    dca::DcaMetadata,
    error::{DcaError, VoiceError},
    events::{Event, EventContext, EventData, EventStore, TrackEvent},
    handler::Handler,
    input::{
        ffmpeg,
        ytdl,
    },
    manager::Manager,
};
pub use audiopus as opus;

use self::connection_info::ConnectionInfo;

const CRYPTO_MODE: &str = "xsalsa20_poly1305";

#[allow(clippy::large_enum_variant)]
pub(crate) enum Status {
    Connect(ConnectionInfo),
    Disconnect,
    SetReceiver(Option<Box<dyn AudioReceiver>>),
    SetSender(Option<Audio>),
    AddSender(Audio),
    SetBitrate(Bitrate),
    AddEvent(EventData),
}
