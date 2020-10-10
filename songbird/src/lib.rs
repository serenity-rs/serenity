//! A module for connecting to voice channels.

pub mod constants;
#[cfg(feature = "driver")]
pub mod driver;
#[cfg(feature = "driver")]
pub mod events;
pub mod error;
#[cfg(feature = "gateway")]
mod handler;
#[cfg(feature = "gateway")]
pub mod id;
pub(crate) mod info;
#[cfg(feature = "driver")]
pub mod input;
#[cfg(feature = "gateway")]
mod manager;
#[cfg(feature = "serenity")]
pub mod serenity;
#[cfg(feature = "gateway")]
pub mod shards;
#[cfg(feature = "driver")]
mod timer;
#[cfg(feature = "driver")]
pub mod tracks;
#[cfg(feature = "driver")]
mod ws;

#[cfg(feature = "driver")]
pub use audiopus::{self as opus, Bitrate};
#[cfg(feature = "driver")]
pub use discortp as packet;
#[cfg(feature = "driver")]
pub use serenity_voice_model as model;

pub use crate::{
    events::{CoreEvent, Event, EventContext, EventHandler, TrackEvent},
    handler::Handler,
    input::{
        ffmpeg,
        ytdl,
    },
    manager::Manager,
    tracks::create_player,
};
#[cfg(feature = "serenity")]
pub use crate::serenity::*;

pub use info::ConnectionInfo;

