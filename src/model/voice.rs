//! Representations of voice information.

use std::fmt;

use serde::de::{Deserialize, Deserializer};

use crate::model::guild::{InterimMember, Member};
use crate::model::id::{ChannelId, GuildId, UserId};
use crate::model::Timestamp;

/// Information about an available voice region.
///
/// [Discord docs](https://discord.com/developers/docs/resources/voice#voice-region-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VoiceRegion {
    /// Whether it is a custom voice region, which is used for events.
    pub custom: bool,
    /// Whether it is a deprecated voice region, which you should avoid using.
    pub deprecated: bool,
    /// The internal Id of the voice region.
    pub id: String,
    /// A recognizable name of the location of the voice region.
    pub name: String,
    /// Whether the voice region is optimal for use by the current user.
    pub optimal: bool,
}

/// A user's state within a voice channel.
///
/// [Discord docs](https://discord.com/developers/docs/resources/voice#voice-state-object).
#[derive(Clone, Serialize)]
#[non_exhaustive]
pub struct VoiceState {
    pub channel_id: Option<ChannelId>,
    pub deaf: bool,
    pub guild_id: Option<GuildId>,
    pub member: Option<Member>,
    pub mute: bool,
    pub self_deaf: bool,
    pub self_mute: bool,
    pub self_stream: Option<bool>,
    pub self_video: bool,
    pub session_id: String,
    pub suppress: bool,
    pub token: Option<String>,
    pub user_id: UserId,
    /// When unsuppressed, non-bot users will have this set to the current time.
    /// Bot users will be set to [`None`]. When suppressed, the user will have
    /// their [`Self::request_to_speak_timestamp`] removed.
    pub request_to_speak_timestamp: Option<Timestamp>,
}

impl fmt::Debug for VoiceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VoiceState")
            .field("channel_id", &self.channel_id)
            .field("deaf", &self.deaf)
            .field("guild_id", &self.guild_id)
            .field("member", &self.member)
            .field("mute", &self.mute)
            .field("self_deaf", &self.self_deaf)
            .field("self_mute", &self.self_mute)
            .field("self_stream", &self.self_stream)
            .field("self_video", &self.self_video)
            .field("session_id", &self.session_id)
            .field("suppress", &self.suppress)
            .field("user_id", &self.user_id)
            .field("request_to_speak_timestamp", &self.request_to_speak_timestamp)
            .finish()
    }
}

impl<'de> Deserialize<'de> for VoiceState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct InterimVoiceState {
            channel_id: Option<ChannelId>,
            deaf: bool,
            guild_id: Option<GuildId>,
            member: Option<InterimMember>,
            mute: bool,
            self_deaf: bool,
            self_mute: bool,
            self_stream: Option<bool>,
            self_video: bool,
            session_id: String,
            suppress: bool,
            token: Option<String>,
            user_id: UserId,
            request_to_speak_timestamp: Option<Timestamp>,
        }

        let mut state = InterimVoiceState::deserialize(deserializer)?;

        if let (Some(guild_id), Some(member)) = (state.guild_id, state.member.as_mut()) {
            member.guild_id = guild_id;
        }

        Ok(VoiceState {
            channel_id: state.channel_id,
            deaf: state.deaf,
            guild_id: state.guild_id,
            member: state.member.map(Member::from),
            mute: state.mute,
            self_deaf: state.self_deaf,
            self_mute: state.self_mute,
            self_stream: state.self_stream,
            self_video: state.self_video,
            session_id: state.session_id,
            suppress: state.suppress,
            token: state.token,
            user_id: state.user_id,
            request_to_speak_timestamp: state.request_to_speak_timestamp,
        })
    }
}
