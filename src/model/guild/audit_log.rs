use std::{collections::HashMap, fmt, mem::transmute};

use serde::de::{self, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

use crate::model::prelude::*;

/// Determines the action that was done on a target.
#[derive(Copy, Clone, Debug)]
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
    pub fn num(self) -> u8 {
        match self {
            Action::GuildUpdate => 1,
            Action::Channel(x) => x as u8,
            Action::ChannelOverwrite(x) => x as u8,
            Action::Member(x) => x as u8,
            Action::Role(x) => x as u8,
            Action::Invite(x) => x as u8,
            Action::Webhook(x) => x as u8,
            Action::Emoji(x) => x as u8,
            Action::Message(x) => x as u8,
            Action::Integration(x) => x as u8,
            Action::StageInstance(x) => x as u8,
            Action::Sticker(x) => x as u8,
            Action::Thread(x) => x as u8,
        }
    }

    pub fn from_value(value: u8) -> Option<Action> {
        let action = match value {
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
            _ => return None,
        };

        Some(action)
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        Action::from_value(value)
            .ok_or_else(|| de::Error::custom(format!("Unexpected action number: {}", value)))
    }
}

impl Serialize for Action {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        serializer.serialize_u8(self.num())
    }
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionChannel {
    Create = 10,
    Update = 11,
    Delete = 12,
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionChannelOverwrite {
    Create = 13,
    Update = 14,
    Delete = 15,
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionRole {
    Create = 30,
    Update = 31,
    Delete = 32,
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionInvite {
    Create = 40,
    Update = 41,
    Delete = 42,
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionWebhook {
    Create = 50,
    Update = 51,
    Delete = 52,
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionEmoji {
    Create = 60,
    Update = 61,
    Delete = 62,
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionMessage {
    Delete = 72,
    BulkDelete = 73,
    Pin = 74,
    Unpin = 75,
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionIntegration {
    Create = 80,
    Update = 81,
    Delete = 82,
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionStageInstance {
    Create = 83,
    Update = 84,
    Delete = 85,
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionSticker {
    Create = 90,
    Update = 91,
    Delete = 92,
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ActionThread {
    Create = 110,
    Update = 111,
    Delete = 112,
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
    #[serde(rename = "action_type")]
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
        Option::serialize(num, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_value() {
        macro_rules! assert_action {
            ($action:pat, $num:literal) => {{
                let a = Action::from_value($num).expect("invalid action value");
                assert!(matches!(a, $action), "{:?} didn't match the variant", a);
                assert_eq!(a.num(), $num);
            }};
        }

        assert_action!(Action::GuildUpdate, 1);
        assert_action!(Action::Channel(ActionChannel::Create), 10);
        assert_action!(Action::Channel(ActionChannel::Update), 11);
        assert_action!(Action::Channel(ActionChannel::Delete), 12);
        assert_action!(Action::ChannelOverwrite(ActionChannelOverwrite::Create), 13);
        assert_action!(Action::ChannelOverwrite(ActionChannelOverwrite::Update), 14);
        assert_action!(Action::ChannelOverwrite(ActionChannelOverwrite::Delete), 15);
        assert_action!(Action::Member(ActionMember::Kick), 20);
        assert_action!(Action::Member(ActionMember::Prune), 21);
        assert_action!(Action::Member(ActionMember::BanAdd), 22);
        assert_action!(Action::Member(ActionMember::BanRemove), 23);
        assert_action!(Action::Member(ActionMember::Update), 24);
        assert_action!(Action::Member(ActionMember::RoleUpdate), 25);
        assert_action!(Action::Member(ActionMember::MemberMove), 26);
        assert_action!(Action::Member(ActionMember::MemberDisconnect), 27);
        assert_action!(Action::Member(ActionMember::BotAdd), 28);
        assert_action!(Action::Role(ActionRole::Create), 30);
        assert_action!(Action::Role(ActionRole::Update), 31);
        assert_action!(Action::Role(ActionRole::Delete), 32);
        assert_action!(Action::Invite(ActionInvite::Create), 40);
        assert_action!(Action::Invite(ActionInvite::Update), 41);
        assert_action!(Action::Invite(ActionInvite::Delete), 42);
        assert_action!(Action::Webhook(ActionWebhook::Create), 50);
        assert_action!(Action::Webhook(ActionWebhook::Update), 51);
        assert_action!(Action::Webhook(ActionWebhook::Delete), 52);
        assert_action!(Action::Emoji(ActionEmoji::Create), 60);
        assert_action!(Action::Emoji(ActionEmoji::Update), 61);
        assert_action!(Action::Emoji(ActionEmoji::Delete), 62);
        assert_action!(Action::Message(ActionMessage::Delete), 72);
        assert_action!(Action::Message(ActionMessage::BulkDelete), 73);
        assert_action!(Action::Message(ActionMessage::Pin), 74);
        assert_action!(Action::Message(ActionMessage::Unpin), 75);
        assert_action!(Action::Integration(ActionIntegration::Create), 80);
        assert_action!(Action::Integration(ActionIntegration::Update), 81);
        assert_action!(Action::Integration(ActionIntegration::Delete), 82);
        assert_action!(Action::StageInstance(ActionStageInstance::Create), 83);
        assert_action!(Action::StageInstance(ActionStageInstance::Update), 84);
        assert_action!(Action::StageInstance(ActionStageInstance::Delete), 85);
        assert_action!(Action::Sticker(ActionSticker::Create), 90);
        assert_action!(Action::Sticker(ActionSticker::Update), 91);
        assert_action!(Action::Sticker(ActionSticker::Delete), 92);
        assert_action!(Action::Thread(ActionThread::Create), 110);
        assert_action!(Action::Thread(ActionThread::Update), 111);
        assert_action!(Action::Thread(ActionThread::Delete), 112);
    }
}
