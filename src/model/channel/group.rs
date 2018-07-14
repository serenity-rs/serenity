use chrono::{DateTime, FixedOffset};
use model::prelude::*;
use std::borrow::Cow;
use std::fmt::Write as FmtWrite;

/// A group channel - potentially including other [`User`]s - separate from a
/// [`Guild`].
///
/// [`Guild`]: struct.Guild.html
/// [`User`]: struct.User.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group {
    /// The Id of the group channel.
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
    /// The optional icon of the group channel.
    pub icon: Option<String>,
    /// The Id of the last message sent.
    pub last_message_id: Option<MessageId>,
    /// Timestamp of the latest pinned message.
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
    /// The name of the group channel.
    pub name: Option<String>,
    /// The Id of the group owner.
    pub owner_id: UserId,
    /// A map of the group's recipients.
    #[serde(deserialize_with = "deserialize_users",
            serialize_with = "serialize_users")]
    pub recipients: HashMap<UserId, User>,
}

impl Group {
    /// Returns the formatted URI of the group's icon if one exists.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon| {
            format!(cdn!("/channel-icons/{}/{}.webp"), self.channel_id, icon)
        })
    }

    /// Determines if the channel is NSFW.
    ///
    /// Refer to [`utils::is_nsfw`] for more details.
    ///
    /// **Note**: This method is for consistency. This will always return
    /// `false`, due to groups not being considered NSFW.
    ///
    /// [`utils::is_nsfw`]: ../../utils/fn.is_nsfw.html
    #[inline]
    pub fn is_nsfw(&self) -> bool { false }

    /// Generates a name for the group.
    ///
    /// If there are no recipients in the group, the name will be "Empty Group".
    /// Otherwise, the name is generated in a Comma Separated Value list, such
    /// as "person 1, person 2, person 3".
    pub fn name(&self) -> Cow<str> {
        match self.name {
            Some(ref name) => Cow::Borrowed(name),
            None => {
                let mut name = match self.recipients.values().nth(0) {
                    Some(recipient) => recipient.name.clone(),
                    None => return Cow::Borrowed("Empty Group"),
                };

                for recipient in self.recipients.values().skip(1) {
                    let _ = write!(name, ", {}", recipient.name);
                }

                Cow::Owned(name)
            },
        }
    }
}
