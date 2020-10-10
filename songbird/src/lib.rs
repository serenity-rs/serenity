//! # Songbird
//! 
//! ![project logo][logo]
//! 
//! Songbird is a cross-library compatible voice system for Discord, written in Rust. The library offers:
//! * A (standalone) gateway frontend compatible with [serenity] and [twilight] using the `"gateway"` and `"[serenity/twilight]-[rustls/native]"` features.
//!  * An optionally standalone driver for voice calls.
//!  * And, by default, a fully featured voice system featuring events, RT(C)P packet handling, seeking on compatible streams, shared multithreaded audio stream caches, and direct Opus data passthrough from DCA files.
//! 
//! ## Attribution
//! 
//! Songbird's logo is based upon the copyright-free image ["Black-Capped Chickadee"] by George Gorgas White.
//!
//! [logo]: https://raw.githubusercontent.com/FelixMcFelix/serenity/voice-rework/songbird/songbird.png
//! [serenity]: https://github.com/serenity-rs/serenity
//! [twilight]: https://github.com/twilight-rs/twilight
//! ["Black-Capped Chickadee"]: https://www.oldbookillustrations.com/illustrations/black-capped-chickadee/

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
    handler::Call,
    input::{
        ffmpeg,
        ytdl,
    },
    manager::Songbird,
    tracks::create_player,
};
#[cfg(feature = "serenity")]
pub use crate::serenity::*;

pub use info::ConnectionInfo;

