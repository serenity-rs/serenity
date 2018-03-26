use chrono::{DateTime, FixedOffset};
use model::prelude::*;

use std::fmt::{Display, Formatter, Result as FmtResult};
#[cfg(feature = "utils")]
use utils as serenity_utils;

/// Represents a guild's text or voice channel. Some methods are available only
/// for voice channels and some are only available for text channels.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildChannel {
    /// The unique Id of the channel.
    ///
    /// The default channel Id shares the Id of the guild and the default role.
    pub id: ChannelId,
    /// The bitrate of the channel.
    ///
    /// **Note**: This is only available for voice channels.
    pub bitrate: Option<u64>,
    /// Whether this guild channel belongs in a category.
    #[serde(rename = "parent_id")]
    pub category_id: Option<ChannelId>,
    /// The Id of the guild the channel is located in.
    ///
    /// If this matches with the [`id`], then this is the default text channel.
    ///
    /// The original voice channel has an Id equal to the guild's Id,
    /// incremented by one.
    pub guild_id: GuildId,
    /// The type of the channel.
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The Id of the last message sent in the channel.
    ///
    /// **Note**: This is only available for text channels.
    pub last_message_id: Option<MessageId>,
    /// The timestamp of the time a pin was most recently made.
    ///
    /// **Note**: This is only available for text channels.
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
    /// The name of the channel.
    pub name: String,
    /// Permission overwrites for [`Member`]s and for [`Role`]s.
    ///
    /// [`Member`]: struct.Member.html
    /// [`Role`]: struct.Role.html
    pub permission_overwrites: Vec<PermissionOverwrite>,
    /// The position of the channel.
    ///
    /// The default text channel will _almost always_ have a position of `-1` or
    /// `0`.
    pub position: i64,
    /// The topic of the channel.
    ///
    /// **Note**: This is only available for text channels.
    pub topic: Option<String>,
    /// The maximum number of members allowed in the channel.
    ///
    /// **Note**: This is only available for voice channels.
    pub user_limit: Option<u64>,
    /// Used to tell if the channel is not safe for work.
    /// Note however, it's recommended to use [`is_nsfw`] as it's gonna be more accurate.
    ///
    /// [`is_nsfw`]: struct.GuildChannel.html#method.is_nsfw
    // This field can or can not be present sometimes, but if it isn't
    // default to `false`.
    #[serde(default)]
    pub nsfw: bool,
}

impl GuildChannel {
    /// Determines if the channel is NSFW.
    ///
    /// Refer to [`utils::is_nsfw`] for more details.
    ///
    /// Only [text channels][`ChannelType::Text`] are taken into consideration
    /// as being NSFW. [voice channels][`ChannelType::Voice`] are never NSFW.
    ///
    /// [`ChannelType::Text`]: enum.ChannelType.html#variant.Text
    /// [`ChannelType::Voice`]: enum.ChannelType.html#variant.Voice
    /// [`utils::is_nsfw`]: ../utils/fn.is_nsfw.html
    #[cfg(feature = "utils")]
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        self.kind == ChannelType::Text && (self.nsfw || serenity_utils::is_nsfw(&self.name))
    }

    /// Returns the name of the guild channel.
    pub fn name(&self) -> &str { &self.name }
}

impl Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut Formatter) -> FmtResult { Display::fmt(&self.id.mention(), f) }
}
