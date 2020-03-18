//! A module for connecting to voice channels.

mod audio;
mod connection;
mod connection_info;
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
        create_player,
        Audio,
        AudioFn,
        AudioHandle,
        AudioQueryResult,
        AudioReceiver,
        AudioResult,
        AudioSource,
        AudioState,
        AudioType,
        BlockingAudioQueryResult,
        TrackQueue,
    },
    dca::DcaMetadata,
    error::{DcaError, VoiceError},
    events::{Event, EventContext, EventData, EventFn, TrackEvent},
    handler::Handler,
    manager::Manager,
    streamer::{
        dca,
        ffmpeg,
        ffmpeg_optioned,
        opus,
        pcm,
        ytdl,
        ytdl_search
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
}
