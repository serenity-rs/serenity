//! Audit log types for administrative actions within guilds.

use std::collections::HashMap;
use std::mem::transmute;

use serde::de::Deserializer;
use serde::ser::{Serialize, Serializer};

mod change;
mod utils;

pub use change::{AffectedRole, Change, EntityType};
use utils::{optional_string, users, webhooks};

use crate::model::prelude::*;

/// Determines the action that was done on a target.
///
/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Action {
    GuildUpdate,
    Channel(ChannelAction),
    ChannelOverwrite(ChannelOverwriteAction),
    Member(MemberAction),
    Role(RoleAction),
    Invite(InviteAction),
    Webhook(WebhookAction),
    Emoji(EmojiAction),
    Message(MessageAction),
    Integration(IntegrationAction),
    StageInstance(StageInstanceAction),
    Sticker(StickerAction),
    ScheduledEvent(ScheduledEventAction),
    Thread(ThreadAction),
    AutoModeration(AutoModerationAction),
    Unknown(u8),
}

impl Action {
    #[must_use]
    pub fn num(self) -> u8 {
        match self {
            Self::GuildUpdate => 1,
            Self::Channel(x) => x as u8,
            Self::ChannelOverwrite(x) => x as u8,
            Self::Member(x) => x as u8,
            Self::Role(x) => x as u8,
            Self::Invite(x) => x as u8,
            Self::Webhook(x) => x as u8,
            Self::Emoji(x) => x as u8,
            Self::Message(x) => x as u8,
            Self::Integration(x) => x as u8,
            Self::StageInstance(x) => x as u8,
            Self::Sticker(x) => x as u8,
            Self::ScheduledEvent(x) => x as u8,
            Self::Thread(x) => x as u8,
            Self::AutoModeration(x) => x as u8,
            Self::Unknown(x) => x,
        }
    }

    #[must_use]
    pub fn from_value(value: u8) -> Action {
        match value {
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
            100..=102 => Action::ScheduledEvent(unsafe { transmute(value) }),
            110..=112 => Action::Thread(unsafe { transmute(value) }),
            140..=143 => Action::AutoModeration(unsafe { transmute(value) }),
            _ => Action::Unknown(value),
        }
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        Ok(Action::from_value(value))
    }
}

impl Serialize for Action {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        serializer.serialize_u8(self.num())
    }
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ChannelAction {
    Create = 10,
    Update = 11,
    Delete = 12,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ChannelOverwriteAction {
    Create = 13,
    Update = 14,
    Delete = 15,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum MemberAction {
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

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum RoleAction {
    Create = 30,
    Update = 31,
    Delete = 32,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum InviteAction {
    Create = 40,
    Update = 41,
    Delete = 42,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum WebhookAction {
    Create = 50,
    Update = 51,
    Delete = 52,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum EmojiAction {
    Create = 60,
    Update = 61,
    Delete = 62,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum MessageAction {
    Delete = 72,
    BulkDelete = 73,
    Pin = 74,
    Unpin = 75,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum IntegrationAction {
    Create = 80,
    Update = 81,
    Delete = 82,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum StageInstanceAction {
    Create = 83,
    Update = 84,
    Delete = 85,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum StickerAction {
    Create = 90,
    Update = 91,
    Delete = 92,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ScheduledEventAction {
    Create = 100,
    Update = 101,
    Delete = 102,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ThreadAction {
    Create = 110,
    Update = 111,
    Delete = 112,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum AutoModerationAction {
    RuleCreate = 140,
    RuleUpdate = 141,
    RuleDelete = 142,
    BlockMessage = 143,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-object).
#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AuditLogs {
    #[serde(rename = "audit_log_entries")]
    pub entries: Vec<AuditLogEntry>,
    #[serde(with = "users")]
    pub users: HashMap<UserId, User>,
    #[serde(with = "webhooks")]
    pub webhooks: HashMap<WebhookId, Webhook>,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object).
#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AuditLogEntry {
    /// Determines to what entity an [`Self::action`] was used on.
    #[serde(with = "optional_string")]
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
    /// Some optional data associated with this entry.
    pub options: Option<Options>,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-optional-audit-entry-info).
#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Options {
    /// Number of days after which inactive members were kicked.
    #[serde(default, with = "optional_string")]
    pub delete_member_days: Option<u64>,
    /// Number of members removed by the prune
    #[serde(default, with = "optional_string")]
    pub members_removed: Option<u64>,
    /// Channel in which the messages were deleted
    #[serde(default)]
    pub channel_id: Option<ChannelId>,
    /// Number of deleted messages.
    #[serde(default, with = "optional_string")]
    pub count: Option<u64>,
    /// Id of the overwritten entity
    #[serde(default)]
    pub id: Option<GenericId>,
    /// Type of overwritten entity ("member" or "role").
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    /// Message that was pinned or unpinned.
    #[serde(default)]
    pub message_id: Option<MessageId>,
    /// Name of the role if type is "role"
    #[serde(default)]
    pub role_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_value() {
        macro_rules! assert_action {
            ($action:pat, $num:literal) => {{
                let a = Action::from_value($num);
                assert!(matches!(a, $action), "{:?} didn't match the variant", a);
                assert_eq!(a.num(), $num);
            }};
        }

        assert_action!(Action::GuildUpdate, 1);
        assert_action!(Action::Channel(ChannelAction::Create), 10);
        assert_action!(Action::Channel(ChannelAction::Update), 11);
        assert_action!(Action::Channel(ChannelAction::Delete), 12);
        assert_action!(Action::ChannelOverwrite(ChannelOverwriteAction::Create), 13);
        assert_action!(Action::ChannelOverwrite(ChannelOverwriteAction::Update), 14);
        assert_action!(Action::ChannelOverwrite(ChannelOverwriteAction::Delete), 15);
        assert_action!(Action::Member(MemberAction::Kick), 20);
        assert_action!(Action::Member(MemberAction::Prune), 21);
        assert_action!(Action::Member(MemberAction::BanAdd), 22);
        assert_action!(Action::Member(MemberAction::BanRemove), 23);
        assert_action!(Action::Member(MemberAction::Update), 24);
        assert_action!(Action::Member(MemberAction::RoleUpdate), 25);
        assert_action!(Action::Member(MemberAction::MemberMove), 26);
        assert_action!(Action::Member(MemberAction::MemberDisconnect), 27);
        assert_action!(Action::Member(MemberAction::BotAdd), 28);
        assert_action!(Action::Role(RoleAction::Create), 30);
        assert_action!(Action::Role(RoleAction::Update), 31);
        assert_action!(Action::Role(RoleAction::Delete), 32);
        assert_action!(Action::Invite(InviteAction::Create), 40);
        assert_action!(Action::Invite(InviteAction::Update), 41);
        assert_action!(Action::Invite(InviteAction::Delete), 42);
        assert_action!(Action::Webhook(WebhookAction::Create), 50);
        assert_action!(Action::Webhook(WebhookAction::Update), 51);
        assert_action!(Action::Webhook(WebhookAction::Delete), 52);
        assert_action!(Action::Emoji(EmojiAction::Create), 60);
        assert_action!(Action::Emoji(EmojiAction::Update), 61);
        assert_action!(Action::Emoji(EmojiAction::Delete), 62);
        assert_action!(Action::Message(MessageAction::Delete), 72);
        assert_action!(Action::Message(MessageAction::BulkDelete), 73);
        assert_action!(Action::Message(MessageAction::Pin), 74);
        assert_action!(Action::Message(MessageAction::Unpin), 75);
        assert_action!(Action::Integration(IntegrationAction::Create), 80);
        assert_action!(Action::Integration(IntegrationAction::Update), 81);
        assert_action!(Action::Integration(IntegrationAction::Delete), 82);
        assert_action!(Action::StageInstance(StageInstanceAction::Create), 83);
        assert_action!(Action::StageInstance(StageInstanceAction::Update), 84);
        assert_action!(Action::StageInstance(StageInstanceAction::Delete), 85);
        assert_action!(Action::Sticker(StickerAction::Create), 90);
        assert_action!(Action::Sticker(StickerAction::Update), 91);
        assert_action!(Action::Sticker(StickerAction::Delete), 92);
        assert_action!(Action::ScheduledEvent(ScheduledEventAction::Create), 100);
        assert_action!(Action::ScheduledEvent(ScheduledEventAction::Update), 101);
        assert_action!(Action::ScheduledEvent(ScheduledEventAction::Delete), 102);
        assert_action!(Action::Thread(ThreadAction::Create), 110);
        assert_action!(Action::Thread(ThreadAction::Update), 111);
        assert_action!(Action::Thread(ThreadAction::Delete), 112);
        assert_action!(Action::AutoModeration(AutoModerationAction::RuleCreate), 140);
        assert_action!(Action::AutoModeration(AutoModerationAction::RuleUpdate), 141);
        assert_action!(Action::AutoModeration(AutoModerationAction::RuleDelete), 142);
        assert_action!(Action::AutoModeration(AutoModerationAction::BlockMessage), 143);
        assert_action!(Action::Unknown(234), 234);
    }

    #[test]
    fn action_serde() {
        use serde_json::json;

        #[derive(Debug, Deserialize, Serialize)]
        struct T {
            action: Action,
        }

        let value = json!({
            "action": 234,
        });

        let value = serde_json::from_value::<T>(value).unwrap();
        assert_eq!(value.action.num(), 234);

        assert!(matches!(value.action, Action::Unknown(234)));
    }
}
