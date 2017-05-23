//! A module for connecting to voice channels.

mod audio;
mod connection;
mod connection_info;
mod error;
mod manager;
mod handler;
mod payload;
mod streamer;
mod threading;

pub use self::audio::{AudioReceiver, AudioSource};
pub use self::error::VoiceError;
pub use self::handler::Handler;
pub use self::manager::Manager;
pub use self::streamer::{ffmpeg, pcm, ytdl};

use self::connection_info::ConnectionInfo;

const CRYPTO_MODE: &'static str = "xsalsa20_poly1305";

#[doc(hidden)]
pub enum Status {
    Connect(ConnectionInfo),
    Disconnect,
    SetReceiver(Option<Box<AudioReceiver>>),
    SetSender(Option<Box<AudioSource>>),
}
