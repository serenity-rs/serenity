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
pub use self::streamer::{ffmpeg, ytdl};

use self::connection_info::ConnectionInfo;
use ::model::{ChannelId, GuildId};

const CRYPTO_MODE: &'static str = "xsalsa20_poly1305";

#[doc(hidden)]
pub enum Status {
    Connect(ConnectionInfo),
    Disconnect,
    SetReceiver(Option<Box<AudioReceiver>>),
    SetSender(Option<Box<AudioSource>>),
}

/// Denotes the target to manage a connection for.
///
/// For most cases, targets should entirely be guilds, except for the one case
/// where a user account can be in a 1-to-1 or group call.
///
/// It _may_ be possible in the future for bots to be in multiple groups. If
/// this turns out to be the case, supporting that now rather than messily in
/// the future is the best option. Thus, these types of calls are specified by
/// the group's channel Id.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Target {
    /// Used for managing a voice handler for a 1-on-1 (user-to-user) or group
    /// call.
    Channel(ChannelId),
    /// Used for managing a voice handler for a guild.
    Guild(GuildId),
}

impl From<ChannelId> for Target {
    fn from(channel_id: ChannelId) -> Target {
        Target::Channel(channel_id)
    }
}

impl From<GuildId> for Target {
    fn from(guild_id: GuildId) -> Target {
        Target::Guild(guild_id)
    }
}
