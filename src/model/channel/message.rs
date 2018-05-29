//! Models relating to Discord channels.

use chrono::{DateTime, FixedOffset};
use constants;
use model::prelude::*;
use serde_json::Value;

/// A representation of a message over a guild's text channel, a group, or a
/// private channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    /// The unique Id of the message. Can be used to calculate the creation date
    /// of the message.
    pub id: MessageId,
    /// An vector of the files attached to a message.
    pub attachments: Vec<Attachment>,
    /// The user that sent the message.
    pub author: User,
    /// The Id of the [`Channel`] that the message was sent to.
    ///
    /// [`Channel`]: enum.Channel.html
    pub channel_id: ChannelId,
    /// The content of the message.
    pub content: String,
    /// The timestamp of the last time the message was updated, if it was.
    pub edited_timestamp: Option<DateTime<FixedOffset>>,
    /// Array of embeds sent with the message.
    pub embeds: Vec<Embed>,
    /// Indicator of the type of message this is, i.e. whether it is a regular
    /// message or a system message.
    #[serde(rename = "type")]
    pub kind: MessageType,
    /// A partial amount of data about the user's member data, if this message
    /// was sent in a guild.
    pub member: Option<PartialMember>,
    /// Indicator of whether the message mentions everyone.
    pub mention_everyone: bool,
    /// Array of [`Role`]s' Ids mentioned in the message.
    ///
    /// [`Role`]: struct.Role.html
    pub mention_roles: Vec<RoleId>,
    /// Array of users mentioned in the message.
    pub mentions: Vec<User>,
    /// Non-repeating number used for ensuring message order.
    #[serde(default)]
    pub nonce: Value,
    /// Indicator of whether the message is pinned.
    pub pinned: bool,
    /// Array of reactions performed on the message.
    #[serde(default)]
    pub reactions: Vec<MessageReaction>,
    /// Initial message creation timestamp, calculated from its Id.
    pub timestamp: DateTime<FixedOffset>,
    /// Indicator of whether the command is to be played back via
    /// text-to-speech.
    ///
    /// In the client, this is done via the `/tts` slash command.
    pub tts: bool,
    /// The Id of the webhook that sent this message, if one did.
    pub webhook_id: Option<WebhookId>,
}

impl Message {
    pub fn transform_content(&mut self) {
        match self.kind {
            MessageType::PinsAdd => {
                self.content = format!(
                    "{} pinned a message to this channel. See all the pins.",
                    self.author
                );
            },
            MessageType::MemberJoin => {
                let sec = self.timestamp.timestamp() as usize;
                let chosen = constants::JOIN_MESSAGES[sec % constants::JOIN_MESSAGES.len()];

                self.content = if chosen.contains("$user") {
                    chosen.replace("$user", &self.author.mention())
                } else {
                    chosen.to_string()
                };
            },
            _ => {},
        }
    }

    /// Checks the length of a string to ensure that it is within Discord's
    /// maximum message length limit.
    ///
    /// Returns `None` if the message is within the limit, otherwise returns
    /// `Some` with an inner value of how many unicode code points the message
    /// is over.
    pub fn overflow_length(content: &str) -> Option<u64> {
        // Check if the content is over the maximum number of unicode code
        // points.
        let count = content.chars().count() as i64;
        let diff = count - i64::from(constants::MESSAGE_CODE_LIMIT);

        if diff > 0 {
            Some(diff as u64)
        } else {
            None
        }
    }

    pub fn check_content_length(map: &JsonMap) -> Result<()> {
        if let Some(content) = map.get("content") {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Model(ModelError::MessageTooLong(length_over)));
                }
            }
        }

        Ok(())
    }

    pub fn check_embed_length(map: &JsonMap) -> Result<()> {
        let embed = match map.get("embed") {
            Some(&Value::Object(ref value)) => value,
            _ => return Ok(()),
        };

        let mut total: usize = 0;

        if let Some(&Value::Object(ref author)) = embed.get("author") {
            if let Some(&Value::Object(ref name)) = author.get("name") {
                total += name.len();
            }
        }

        if let Some(&Value::String(ref description)) = embed.get("description") {
            total += description.len();
        }

        if let Some(&Value::Array(ref fields)) = embed.get("fields") {
            for field_as_value in fields {
                if let Value::Object(ref field) = *field_as_value {
                    if let Some(&Value::String(ref field_name)) = field.get("name") {
                        total += field_name.len();
                    }

                    if let Some(&Value::String(ref field_value)) = field.get("value") {
                        total += field_value.len();
                    }
                }
            }
        }

        if let Some(&Value::Object(ref footer)) = embed.get("footer") {
            if let Some(&Value::String(ref text)) = footer.get("text") {
                total += text.len();
            }
        }

        if let Some(&Value::String(ref title)) = embed.get("title") {
            total += title.len();
        }

        if total <= constants::EMBED_MAX_LENGTH as usize {
            Ok(())
        } else {
            let overflow = total as u64 - u64::from(constants::EMBED_MAX_LENGTH);

            Err(Error::Model(ModelError::EmbedTooLarge(overflow)))
        }
    }
}

impl From<Message> for MessageId {
    /// Gets the Id of a `Message`.
    fn from(message: Message) -> MessageId { message.id }
}

impl<'a> From<&'a Message> for MessageId {
    /// Gets the Id of a `Message`.
    fn from(message: &Message) -> MessageId { message.id }
}

/// A representation of a reaction to a message.
///
/// Multiple of the same [reaction type] are sent into one `MessageReaction`,
/// with an associated [`count`].
///
/// [`count`]: #structfield.count
/// [reaction type]: enum.ReactionType.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageReaction {
    /// The amount of the type of reaction that have been sent for the
    /// associated message.
    pub count: u64,
    /// Indicator of whether the current user has sent the type of reaction.
    pub me: bool,
    /// The type of reaction.
    #[serde(rename = "emoji")]
    pub reaction_type: ReactionType,
}

/// Differentiates between regular and different types of system messages.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum MessageType {
    /// A regular message.
    Regular = 0,
    /// An indicator that a recipient was added by the author.
    GroupRecipientAddition = 1,
    /// An indicator that a recipient was removed by the author.
    GroupRecipientRemoval = 2,
    /// An indicator that a call was started by the author.
    GroupCallCreation = 3,
    /// An indicator that the group name was modified by the author.
    GroupNameUpdate = 4,
    /// An indicator that the group icon was modified by the author.
    GroupIconUpdate = 5,
    /// An indicator that a message was pinned by the author.
    PinsAdd = 6,
    /// An indicator that a member joined the guild.
    MemberJoin = 7,
}

enum_number!(
    MessageType {
        Regular,
        GroupRecipientAddition,
        GroupRecipientRemoval,
        GroupCallCreation,
        GroupNameUpdate,
        GroupIconUpdate,
        PinsAdd,
        MemberJoin,
    }
);

impl MessageType {
    pub fn num(&self) -> u64 {
        use self::MessageType::*;

        match *self {
            Regular => 0,
            GroupRecipientAddition => 1,
            GroupRecipientRemoval => 2,
            GroupCallCreation => 3,
            GroupNameUpdate => 4,
            GroupIconUpdate => 5,
            PinsAdd => 6,
            MemberJoin => 7,
        }
    }
}
