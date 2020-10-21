#![doc(
    html_logo_url = "https://raw.githubusercontent.com/FelixMcFelix/serenity/voice-rework/songbird/songbird.png",
    html_favicon_url = "https://raw.githubusercontent.com/FelixMcFelix/serenity/voice-rework/songbird/songbird-ico.png"
)]
#![deny(missing_docs)]
//! ![project logo][logo]
//!
//! Songbird is an async, cross-library compatible voice system for Discord, written in Rust.
//! The library offers:
//!  * A standalone gateway frontend compatible with [serenity] and [twilight] using the
//!  `"gateway"` and `"[serenity/twilight]-[rustls/native]"` features. You can even run
//!  driverless, to help manage your [lavalink] sessions.
//!  * A standalone driver for voice calls, via the `"driver"` feature. If you can create
//!  a [`ConnectionInfo`] using any other gateway, or language for your bot, then you
//!  can run the songbird voice driver.
//!  * And, by default, a fully featured voice system featuring events, queues, RT(C)P packet
//!  handling, seeking on compatible streams, shared multithreaded audio stream caches,
//!  and direct Opus data passthrough from DCA files.
//!
//! ## Examples
//! Full examples showing various types of functionality and integrations can be found as part of [serenity's examples],
//! and in [this crate's examples directory].
//!
//! ## Attribution
//!
//! Songbird's logo is based upon the copyright-free image ["Black-Capped Chickadee"] by George Gorgas White.
//!
//! [logo]: https://raw.githubusercontent.com/FelixMcFelix/serenity/voice-rework/songbird/songbird.png
//! [serenity]: https://github.com/serenity-rs/serenity
//! [twilight]: https://github.com/twilight-rs/twilight
//! [serenity's examples]: https://github.com/serenity-rs/serenity/tree/current/examples
//! [this crate's examples directory]: https://github.com/serenity-rs/serenity/tree/current/songbird/examples
//! ["Black-Capped Chickadee"]: https://www.oldbookillustrations.com/illustrations/black-capped-chickadee/
//! [`ConnectionInfo`]: struct.ConnectionInfo.html
//! [lavalink]: https://github.com/Frederikam/Lavalink

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

#[cfg(test)]
use utils as test_utils;

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
