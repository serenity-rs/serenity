//! Representations of voice information.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::de::{self, Deserialize, Deserializer, IgnoredAny, MapAccess, Visitor};

use super::{
    guild::Member,
    id::{ChannelId, GuildId, RoleId, UserId},
    user::User,
};
#[cfg(feature = "unstable_discord_api")]
use crate::model::permissions::Permissions;

/// Information about an available voice region.
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
    /// Indicator of whether the voice region is only for VIP guilds.
    pub vip: bool,
}

/// A user's state within a voice channel.
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
    pub request_to_speak_timestamp: Option<DateTime<Utc>>,
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
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            ChannelId,
            Deaf,
            GuildId,
            Member,
            Mute,
            SelfDeaf,
            SelfMute,
            SelfStream,
            SelfVideo,
            SessionId,
            Suppress,
            Token,
            UserId,
            RequestToSpeakTimestamp,
        }

        #[derive(Deserialize)]
        #[non_exhaustive]
        struct PartialMember {
            deaf: bool,
            joined_at: Option<DateTime<Utc>>,
            mute: bool,
            nick: Option<String>,
            roles: Vec<RoleId>,
            user: User,
            #[serde(default)]
            pending: bool,
            premium_since: Option<DateTime<Utc>>,
            #[cfg(feature = "unstable_discord_api")]
            permissions: Option<Permissions>,
        }

        struct VoiceStateVisitor;

        impl<'de> Visitor<'de> for VoiceStateVisitor {
            type Value = VoiceState;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("struct VoiceState")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut channel_id = None;
                let mut deaf = None;
                let mut guild_id = None;
                let mut member = None;
                let mut mute = None;
                let mut self_deaf = None;
                let mut self_mute = None;
                let mut self_stream = None;
                let mut self_video = None;
                let mut session_id = None;
                let mut suppress = None;
                let mut token = None;
                let mut user_id = None;
                let mut request_to_speak_timestamp = None;

                loop {
                    let key = match map.next_key() {
                        Ok(Some(key)) => key,
                        Ok(None) => break,
                        Err(_) => {
                            map.next_value::<IgnoredAny>()?;
                            continue;
                        },
                    };

                    match key {
                        Field::ChannelId => {
                            if channel_id.is_some() {
                                return Err(de::Error::duplicate_field("channel_id"));
                            }
                            channel_id = map.next_value()?;
                        },
                        Field::Deaf => {
                            if deaf.is_some() {
                                return Err(de::Error::duplicate_field("deaf"));
                            }
                            deaf = Some(map.next_value()?);
                        },
                        Field::GuildId => {
                            if guild_id.is_some() {
                                return Err(de::Error::duplicate_field("guild_id"));
                            }
                            guild_id = map.next_value()?;
                        },
                        Field::Member => {
                            if member.is_some() {
                                return Err(de::Error::duplicate_field("member"));
                            }

                            let partial_member: Option<PartialMember> = map.next_value()?;
                            if let Some(partial_member) = partial_member {
                                member = Some(Member {
                                    deaf: partial_member.deaf,
                                    guild_id: GuildId(0),
                                    joined_at: partial_member.joined_at,
                                    mute: partial_member.mute,
                                    nick: partial_member.nick,
                                    roles: partial_member.roles,
                                    user: partial_member.user,
                                    pending: partial_member.pending,
                                    premium_since: partial_member.premium_since,
                                    #[cfg(feature = "unstable_discord_api")]
                                    permissions: partial_member.permissions,
                                });
                            }
                        },
                        Field::Mute => {
                            if mute.is_some() {
                                return Err(de::Error::duplicate_field("mute"));
                            }
                            mute = Some(map.next_value()?);
                        },
                        Field::SelfDeaf => {
                            if self_deaf.is_some() {
                                return Err(de::Error::duplicate_field("self_deaf"));
                            }
                            self_deaf = Some(map.next_value()?);
                        },
                        Field::SelfMute => {
                            if self_mute.is_some() {
                                return Err(de::Error::duplicate_field("self_mute"));
                            }
                            self_mute = Some(map.next_value()?);
                        },
                        Field::SelfStream => {
                            if self_stream.is_some() {
                                return Err(de::Error::duplicate_field("self_stream"));
                            }
                            self_stream = map.next_value()?;
                        },
                        Field::SelfVideo => {
                            if self_video.is_some() {
                                return Err(de::Error::duplicate_field("self_video"));
                            }
                            self_video = Some(map.next_value()?);
                        },
                        Field::SessionId => {
                            if session_id.is_some() {
                                return Err(de::Error::duplicate_field("session_id"));
                            }
                            session_id = Some(map.next_value()?);
                        },
                        Field::Suppress => {
                            if suppress.is_some() {
                                return Err(de::Error::duplicate_field("suppress"));
                            }
                            suppress = Some(map.next_value()?);
                        },
                        Field::Token => {
                            if token.is_some() {
                                return Err(de::Error::duplicate_field("token"));
                            }
                            token = map.next_value()?;
                        },
                        Field::UserId => {
                            if user_id.is_some() {
                                return Err(de::Error::duplicate_field("user_id"));
                            }
                            user_id = Some(map.next_value()?);
                        },
                        Field::RequestToSpeakTimestamp => {
                            if request_to_speak_timestamp.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "request_to_speak_timestamp",
                                ));
                            }
                            request_to_speak_timestamp = Some(map.next_value()?);
                        },
                    }
                }

                let deaf = deaf.ok_or_else(|| de::Error::missing_field("deaf"))?;
                let mute = mute.ok_or_else(|| de::Error::missing_field("mute"))?;
                let self_deaf = self_deaf.ok_or_else(|| de::Error::missing_field("self_deaf"))?;
                let self_mute = self_mute.ok_or_else(|| de::Error::missing_field("self_mute"))?;
                let self_video =
                    self_video.ok_or_else(|| de::Error::missing_field("self_video"))?;
                let session_id =
                    session_id.ok_or_else(|| de::Error::missing_field("session_id"))?;
                let suppress = suppress.ok_or_else(|| de::Error::missing_field("suppress"))?;
                let user_id = user_id.ok_or_else(|| de::Error::missing_field("user_id"))?;
                let request_to_speak_timestamp = request_to_speak_timestamp.unwrap_or(None);

                if let (Some(guild_id), Some(member)) = (guild_id, member.as_mut()) {
                    member.guild_id = guild_id;
                }

                Ok(VoiceState {
                    channel_id,
                    deaf,
                    guild_id,
                    member,
                    mute,
                    self_deaf,
                    self_mute,
                    self_stream,
                    self_video,
                    session_id,
                    suppress,
                    token,
                    user_id,
                    request_to_speak_timestamp,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "channel_id",
            "deaf",
            "guild_id",
            "member",
            "mute",
            "self_deaf",
            "self_mute",
            "self_stream",
            "self_video",
            "session_id",
            "suppress",
            "token",
            "user_id",
            "request_to_speak_timestamp",
        ];

        deserializer.deserialize_struct("VoiceState", FIELDS, VoiceStateVisitor)
    }
}
