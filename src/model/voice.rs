//! Representations of voice information.

use serde::Serialize;
use serde::de::{Deserialize, Deserializer};

use crate::model::prelude::*;

/// Information about an available voice region.
///
/// [Discord docs](https://docs.discord.com/developers/resources/voice#voice-region-object).
#[bool_to_bitflags::bool_to_bitflags]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct VoiceRegion {
    /// Whether it is a custom voice region, which is used for events.
    pub custom: bool,
    /// Whether it is a deprecated voice region, which you should avoid using.
    pub deprecated: bool,
    /// The internal Id of the voice region.
    pub id: FixedString,
    /// A recognizable name of the location of the voice region.
    pub name: FixedString,
    /// Whether the voice region is optimal for use by the current user.
    pub optimal: bool,
}

/// A user's state within a voice channel.
///
/// [Discord docs](https://docs.discord.com/developers/resources/voice#voice-state-object).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Self")]
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
    pub session_id: FixedString,
    pub suppress: bool,
    pub user_id: UserId,
    /// When unsuppressed, non-bot users will have this set to the current time. Bot users will be
    /// set to [`None`]. When suppressed, the user will have their
    /// [`Self::request_to_speak_timestamp`] removed.
    pub request_to_speak_timestamp: Option<Timestamp>,
}

impl extract_map::ExtractKey<UserId> for VoiceState {
    fn extract_key(&self) -> &UserId {
        &self.user_id
    }
}

// Manual impl needed to insert guild_id into Member
impl<'de> Deserialize<'de> for VoiceStateGeneratedOriginal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // calls #[serde(remote)]-generated inherent method
        let mut state = Self::deserialize(deserializer)?;
        if let (Some(guild_id), Some(member)) = (state.guild_id, state.member.as_mut()) {
            member.guild_id = guild_id;
        }
        Ok(state)
    }
}

impl Serialize for VoiceStateGeneratedOriginal {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // calls #[serde(remote)]-generated inherent method
        Self::serialize(self, serializer)
    }
}

/// Information about an effect, such as an emoji reaction or a soundboard sound, sent in a voice
/// channel.
///
/// [Discord docs](https://docs.discord.com/developers/events/gateway-events#voice-channel-effect-send).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VoiceChannelEffect {
    /// The Id of the channel the effect was sent in.
    pub channel_id: ChannelId,
    /// The Id of the guild the effect was sent in.
    pub guild_id: GuildId,
    /// The Id of the user who sent the effect.
    pub user_id: UserId,
    /// The emoji sent, for emoji reaction and soundboard effects.
    pub emoji: Option<ReactionType>,
    /// The type of emoji animation, for emoji reaction and soundboard effects.
    pub animation_type: Option<AnimationType>,
    /// The Id of the emoji animation, for emoji reaction and soundboard effects.
    pub animation_id: Option<u32>,
    /// The Id of the soundboard sound, for soundboard effects.
    pub sound_id: Option<SoundId>,
    /// The volume of the soundboard sound, from 0 to 1, for soundboard effects.
    pub sound_volume: Option<f64>,
}

enum_number! {
    /// The type of emoji animation, for emoji reaction and soundboard effects.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum AnimationType {
        Premium = 0,
        Basic = 1,
        _ => Unknown(u8),
    }
}
