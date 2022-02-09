use std::{collections::HashMap, fmt, mem::transmute};

use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;

use crate::internal::prelude::*;
use crate::model::prelude::*;

/// Determines the action that was done on a target.
#[derive(Debug)]
#[non_exhaustive]
pub enum Action {
    GuildUpdate,
    Channel(ActionChannel),
    ChannelOverwrite(ActionChannelOverwrite),
    Member(ActionMember),
    Role(ActionRole),
    Invite(ActionInvite),
    Webhook(ActionWebhook),
    Emoji(ActionEmoji),
    Message(ActionMessage),
    Integration(ActionIntegration),
    StageInstance(ActionStageInstance),
    Sticker(ActionSticker),
    Thread(ActionThread),
}

impl Action {
    pub fn num(&self) -> u8 {
        use self::Action::*;

        match *self {
            GuildUpdate => 1,
            Action::Channel(ref x) => x.num(),
            Action::ChannelOverwrite(ref x) => x.num(),
            Action::Member(ref x) => x.num(),
            Action::Role(ref x) => x.num(),
            Action::Invite(ref x) => x.num(),
            Action::Webhook(ref x) => x.num(),
            Action::Emoji(ref x) => x.num(),
            Action::Message(ref x) => x.num(),
            Action::Integration(ref x) => x.num(),
            Action::StageInstance(ref x) => x.num(),
            Action::Sticker(ref x) => x.num(),
            Action::Thread(ref x) => x.num(),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionChannel {
    Create = 10,
    Update = 11,
    Delete = 12,
}

impl ActionChannel {
    pub fn num(&self) -> u8 {
        match *self {
            ActionChannel::Create => 10,
            ActionChannel::Update => 11,
            ActionChannel::Delete => 12,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionChannelOverwrite {
    Create = 13,
    Update = 14,
    Delete = 15,
}

impl ActionChannelOverwrite {
    pub fn num(&self) -> u8 {
        match *self {
            ActionChannelOverwrite::Create => 13,
            ActionChannelOverwrite::Update => 14,
            ActionChannelOverwrite::Delete => 15,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionMember {
    Kick = 20,
    Prune = 21,
    BanAdd = 22,
    BanRemove = 23,
    Update = 24,
    RoleUpdate = 25,
    MemberMove = 26,
    MemberDisconnect = 27,
    BotAdd = 28,
}

impl ActionMember {
    pub fn num(&self) -> u8 {
        match *self {
            ActionMember::Kick => 20,
            ActionMember::Prune => 21,
            ActionMember::BanAdd => 22,
            ActionMember::BanRemove => 23,
            ActionMember::Update => 24,
            ActionMember::RoleUpdate => 25,
            ActionMember::MemberMove => 26,
            ActionMember::MemberDisconnect => 27,
            ActionMember::BotAdd => 28,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionRole {
    Create = 30,
    Update = 31,
    Delete = 32,
}

impl ActionRole {
    pub fn num(&self) -> u8 {
        match *self {
            ActionRole::Create => 30,
            ActionRole::Update => 31,
            ActionRole::Delete => 32,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionInvite {
    Create = 40,
    Update = 41,
    Delete = 42,
}

impl ActionInvite {
    pub fn num(&self) -> u8 {
        match *self {
            ActionInvite::Create => 40,
            ActionInvite::Update => 41,
            ActionInvite::Delete => 42,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionWebhook {
    Create = 50,
    Update = 51,
    Delete = 52,
}

impl ActionWebhook {
    pub fn num(&self) -> u8 {
        match *self {
            ActionWebhook::Create => 50,
            ActionWebhook::Update => 51,
            ActionWebhook::Delete => 52,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionEmoji {
    Create = 60,
    Delete = 61,
    Update = 62,
}

impl ActionEmoji {
    pub fn num(&self) -> u8 {
        match *self {
            ActionEmoji::Create => 60,
            ActionEmoji::Update => 61,
            ActionEmoji::Delete => 62,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionMessage {
    Delete = 72,
    BulkDelete = 73,
    Pin = 74,
    Unpin = 75,
}

impl ActionMessage {
    pub fn num(&self) -> u8 {
        match *self {
            ActionMessage::Delete => 72,
            ActionMessage::BulkDelete => 73,
            ActionMessage::Pin => 74,
            ActionMessage::Unpin => 75,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionIntegration {
    Create = 80,
    Update = 81,
    Delete = 82,
}

impl ActionIntegration {
    pub fn num(&self) -> u8 {
        match *self {
            ActionIntegration::Create => 80,
            ActionIntegration::Update => 81,
            ActionIntegration::Delete => 82,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionStageInstance {
    Create = 83,
    Update = 84,
    Delete = 85,
}

impl ActionStageInstance {
    pub fn num(&self) -> u8 {
        match *self {
            ActionStageInstance::Create => 83,
            ActionStageInstance::Update => 84,
            ActionStageInstance::Delete => 85,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionSticker {
    Create = 90,
    Update = 91,
    Delete = 92,
}

impl ActionSticker {
    pub fn num(&self) -> u8 {
        match *self {
            ActionSticker::Create => 90,
            ActionSticker::Update => 91,
            ActionSticker::Delete => 92,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionThread {
    Create = 110,
    Update = 111,
    Delete = 112,
}

impl ActionThread {
    pub fn num(&self) -> u8 {
        match *self {
            ActionThread::Create => 110,
            ActionThread::Update => 111,
            ActionThread::Delete => 112,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Change {
    #[serde(rename = "key")]
    pub name: String,
    // TODO: Change these to an actual type.
    #[serde(rename = "old_value")]
    pub old: Option<Value>,
    #[serde(rename = "new_value")]
    pub new: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AuditLogs {
    #[serde(with = "entries", rename = "audit_log_entries")]
    pub entries: HashMap<AuditLogEntryId, AuditLogEntry>,
    pub webhooks: Vec<Webhook>,
    pub users: Vec<User>,
}

mod entries {
    use std::collections::HashMap;

    use serde::Deserializer;

    use super::{AuditLogEntry, AuditLogEntryId};
    use crate::model::utils::SequenceToMapVisitor;

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<AuditLogEntryId, AuditLogEntry>, D::Error> {
        deserializer.deserialize_seq(SequenceToMapVisitor::new(|e: &AuditLogEntry| e.id))
    }

    pub use crate::model::utils::serialize_map_values as serialize;
}

#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AuditLogEntry {
    /// Determines to what entity an [`Self::action`] was used on.
    #[serde(with = "option_u64_handler")]
    pub target_id: Option<u64>,
    /// Determines what action was done on a [`Self::target_id`]
    #[serde(with = "action_handler", rename = "action_type")]
    pub action: Action,
    /// What was the reasoning by doing an action on a target? If there was one.
    pub reason: Option<String>,
    /// The user that did this action on a target.
    pub user_id: UserId,
    /// What changes were made.
    pub changes: Option<Vec<Change>>,
    /// The id of this entry.
    pub id: AuditLogEntryId,
    /// Some optional data assosiated with this entry.
    pub options: Option<Options>,
}

#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Options {
    /// Number of days after which inactive members were kicked.
    #[serde(default, with = "option_u64_handler")]
    pub delete_member_days: Option<u64>,
    /// Number of members removed by the prune
    #[serde(default, with = "option_u64_handler")]
    pub members_removed: Option<u64>,
    /// Channel in which the messages were deleted
    #[serde(default)]
    pub channel_id: Option<ChannelId>,
    /// Number of deleted messages.
    #[serde(default, with = "option_u64_handler")]
    pub count: Option<u64>,
    /// Id of the overwritten entity
    #[serde(default, with = "option_u64_handler")]
    pub id: Option<u64>,
    /// Type of overwritten entity ("member" or "role").
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    /// Name of the role if type is "role"
    #[serde(default)]
    pub role_name: Option<String>,
}

mod option_u64_handler {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(des: D) -> StdResult<Option<u64>, D::Error> {
        struct OptionU64Visitor;

        impl<'de> Visitor<'de> for OptionU64Visitor {
            type Value = Option<u64>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an optional integer or a string with a valid number inside")
            }

            fn visit_some<D: Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> StdResult<Self::Value, D::Error> {
                deserializer.deserialize_any(OptionU64Visitor)
            }

            fn visit_none<E: de::Error>(self) -> StdResult<Self::Value, E> {
                Ok(None)
            }

            fn visit_u64<E: de::Error>(self, val: u64) -> StdResult<Option<u64>, E> {
                Ok(Some(val))
            }

            fn visit_str<E: de::Error>(self, string: &str) -> StdResult<Option<u64>, E> {
                string.parse().map(Some).map_err(de::Error::custom)
            }
        }

        des.deserialize_option(OptionU64Visitor)
    }

    pub fn serialize<S: Serializer>(num: &Option<u64>, s: S) -> StdResult<S::Ok, S::Error> {
        use serde::Serialize;

        Option::serialize(num, s)
    }
}

mod action_handler {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> StdResult<Action, D::Error> {
        struct ActionVisitor;

        impl<'de> Visitor<'de> for ActionVisitor {
            type Value = Action;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an integer between 1 to 72")
            }

            // NOTE: Serde internally delegates number types below `u64` to it.
            fn visit_u64<E: de::Error>(self, value: u64) -> StdResult<Action, E> {
                let value = value as u8;

                Ok(match value {
                    1 => Action::GuildUpdate,
                    10..=12 => Action::Channel(unsafe { transmute(value) }),
                    13..=15 => Action::ChannelOverwrite(unsafe { transmute(value) }),
                    20..=28 => Action::Member(unsafe { transmute(value) }),
                    30..=32 => Action::Role(unsafe { transmute(value) }),
                    40..=42 => Action::Invite(unsafe { transmute(value) }),
                    50..=52 => Action::Webhook(unsafe { transmute(value) }),
                    60..=62 => Action::Emoji(unsafe { transmute(value) }),
                    72..=75 => Action::Message(unsafe { transmute(value) }),
                    80..=82 => Action::Integration(unsafe { transmute(value) }),
                    83..=85 => Action::StageInstance(unsafe { transmute(value) }),
                    90..=92 => Action::Sticker(unsafe { transmute(value) }),
                    110..=112 => Action::Thread(unsafe { transmute(value) }),
                    _ => return Err(E::custom(format!("Unexpected action number: {}", value))),
                })
            }
        }

        de.deserialize_any(ActionVisitor)
    }

    pub fn serialize<S: Serializer>(action: &Action, serializer: S) -> StdResult<S::Ok, S::Error> {
        serializer.serialize_u8(action.num())
    }
}
