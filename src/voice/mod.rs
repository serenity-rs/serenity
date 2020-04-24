//! A module for connecting to voice channels.

mod audio;
mod connection;
mod connection_info;
mod constants;
mod dca;
mod error;
mod events;
mod manager;
mod handler;
mod payload;
mod streamer;
mod threading;

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
        ReadSeek,
        TrackQueue,
        create_player,
    },
    dca::DcaMetadata,
    error::{DcaError, VoiceError},
    events::{Event, EventContext, EventData, EventStore, TrackEvent},
    handler::Handler,
    manager::Manager,
    streamer::{
        ChildContainer,
        CompressedSource,
        CompressedSourceBase,
        Input,
        MemorySource,
        Reader,
        RestartableSource,
        child_to_reader,
        // dca,
        ffmpeg,
        ffmpeg_optioned,
        // opus,
        ytdl,
        ytdl_search,
    }
};
pub use audiopus::Bitrate;

use self::connection_info::ConnectionInfo;

const CRYPTO_MODE: &str = "xsalsa20_poly1305";

pub(crate) enum Status {
    Connect(ConnectionInfo),
    Disconnect,
    SetReceiver(Option<Box<dyn AudioReceiver>>),
    SetSender(Option<Audio>),
    AddSender(Audio),
    SetBitrate(Bitrate),
    AddEvent(EventData),
}
