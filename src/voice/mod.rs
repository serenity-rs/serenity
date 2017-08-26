//! A module for connecting to voice channels.

mod audio;
mod connection;
mod connection_info;
mod dca;
mod error;
mod manager;
mod handler;
mod payload;
mod streamer;
mod threading;

pub use self::audio::{AudioReceiver, AudioSource, AudioType};
pub use self::dca::DcaMetadata;
pub use self::error::{DcaError, VoiceError};
pub use self::handler::Handler;
pub use self::manager::Manager;
pub use self::streamer::{dca, ffmpeg, opus, pcm, ytdl};

use self::connection_info::ConnectionInfo;

const CRYPTO_MODE: &'static str = "xsalsa20_poly1305";

pub(crate) enum Status {
    Connect(ConnectionInfo),
    #[allow(dead_code)]
    Disconnect,
    SetReceiver(Option<Box<AudioReceiver>>),
    SetSender(Option<Box<AudioSource>>),
}
