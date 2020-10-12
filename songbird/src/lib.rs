#![doc(
    html_logo_url = "https://raw.githubusercontent.com/FelixMcFelix/serenity/voice-rework/songbird/songbird.png",
    html_favicon_url = "https://raw.githubusercontent.com/FelixMcFelix/serenity/voice-rework/songbird/songbird-ico.png"
)]
// #![deny(missing_docs)]
//! # Songbird
//!
//! ![project logo][logo]
//!
//! Songbird is an async, cross-library compatible voice system for Discord, written in Rust. The library offers:
//!  * A (standalone) gateway frontend compatible with [serenity] and [twilight] using the `"gateway"` and `"[serenity/twilight]-[rustls/native]"` features.
//!  * An optionally standalone driver for voice calls, via the `"driver"` feature.
//!  * And, by default, a fully featured voice system featuring events, RT(C)P packet handling, seeking on compatible streams, shared multithreaded audio stream caches, and direct Opus data passthrough from DCA files.
//!
//! ## Attribution
//!
//! Songbird's logo is based upon the copyright-free image ["Black-Capped Chickadee"] by George Gorgas White.
//!
//! [logo]: https://raw.githubusercontent.com/FelixMcFelix/serenity/voice-rework/songbird/songbird.svg
//! [serenity]: https://github.com/serenity-rs/serenity
//! [twilight]: https://github.com/twilight-rs/twilight
//! ["Black-Capped Chickadee"]: https://www.oldbookillustrations.com/illustrations/black-capped-chickadee/

pub mod constants;
#[cfg(feature = "driver")]
pub mod driver;
pub mod error;
#[cfg(feature = "driver")]
pub mod events;
#[cfg(feature = "gateway")]
mod handler;
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
pub mod tracks;
#[cfg(feature = "driver")]
mod ws;

#[cfg(feature = "driver")]
pub use audiopus::{self as opus, Bitrate};
#[cfg(feature = "driver")]
pub use discortp as packet;
#[cfg(feature = "driver")]
pub use serenity_voice_model as model;

#[cfg(feature = "driver")]
pub use crate::{
    driver::Driver,
    events::{CoreEvent, Event, EventContext, EventHandler, TrackEvent},
    input::{ffmpeg, ytdl},
    tracks::create_player,
};

#[cfg(feature = "gateway")]
pub use crate::{handler::Call, manager::Songbird};

#[cfg(feature = "serenity")]
pub use crate::serenity::*;

pub use info::ConnectionInfo;
