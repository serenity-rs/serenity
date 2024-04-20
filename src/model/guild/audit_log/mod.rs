//! Audit log types for administrative actions within guilds.

use nonmax::{NonMaxU32, NonMaxU64};
use serde::ser::{Serialize, Serializer};

mod change;
mod utils;

pub use change::{AffectedRole, Change, EntityType};
use utils::optional_string;

use crate::internal::prelude::*;
use crate::model::prelude::*;

/// Determines the action that was done on a target.
///
/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
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
    AutoMod(AutoModAction),
    CreatorMonetization(CreatorMonetizationAction),
    VoiceChannelStatus(VoiceChannelStatusAction),
    Unknown(u8),
}

impl Action {
    #[must_use]
    pub const fn num(self) -> u8 {
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
            Self::AutoMod(x) => x as u8,
            Self::CreatorMonetization(x) => x as u8,
            Self::VoiceChannelStatus(x) => x as u8,
            Self::Unknown(x) => x,
        }
    }

    #[must_use]
    #[allow(unknown_lints, clippy::missing_transmute_annotations)]
    pub fn from_value(value: u8) -> Action {
        match value {
            1 => Action::GuildUpdate,
            10 => Action::Channel(ChannelAction::Create),
            11 => Action::Channel(ChannelAction::Update),
            12 => Action::Channel(ChannelAction::Delete),
            13 => Action::ChannelOverwrite(ChannelOverwriteAction::Create),
            14 => Action::ChannelOverwrite(ChannelOverwriteAction::Update),
            15 => Action::ChannelOverwrite(ChannelOverwriteAction::Delete),
            20 => Action::Member(MemberAction::Kick),
            21 => Action::Member(MemberAction::Prune),
            22 => Action::Member(MemberAction::BanAdd),
            23 => Action::Member(MemberAction::BanRemove),
            24 => Action::Member(MemberAction::Update),
            25 => Action::Member(MemberAction::RoleUpdate),
            26 => Action::Member(MemberAction::MemberMove),
            27 => Action::Member(MemberAction::MemberDisconnect),
            28 => Action::Member(MemberAction::BotAdd),
            30 => Action::Role(RoleAction::Create),
            31 => Action::Role(RoleAction::Update),
            32 => Action::Role(RoleAction::Delete),
            40 => Action::Invite(InviteAction::Create),
            41 => Action::Invite(InviteAction::Update),
            42 => Action::Invite(InviteAction::Delete),
            50 => Action::Webhook(WebhookAction::Create),
            51 => Action::Webhook(WebhookAction::Update),
            52 => Action::Webhook(WebhookAction::Delete),
            60 => Action::Emoji(EmojiAction::Create),
            61 => Action::Emoji(EmojiAction::Update),
            62 => Action::Emoji(EmojiAction::Delete),
            72 => Action::Message(MessageAction::Delete),
            73 => Action::Message(MessageAction::BulkDelete),
            74 => Action::Message(MessageAction::Pin),
            75 => Action::Message(MessageAction::Unpin),
            80 => Action::Integration(IntegrationAction::Create),
            81 => Action::Integration(IntegrationAction::Update),
            82 => Action::Integration(IntegrationAction::Delete),
            83 => Action::StageInstance(StageInstanceAction::Create),
            84 => Action::StageInstance(StageInstanceAction::Update),
            85 => Action::StageInstance(StageInstanceAction::Delete),
            90 => Action::Sticker(StickerAction::Create),
            91 => Action::Sticker(StickerAction::Update),
            92 => Action::Sticker(StickerAction::Delete),
            100 => Action::ScheduledEvent(ScheduledEventAction::Create),
            101 => Action::ScheduledEvent(ScheduledEventAction::Update),
            102 => Action::ScheduledEvent(ScheduledEventAction::Delete),
            110 => Action::Thread(ThreadAction::Create),
            111 => Action::Thread(ThreadAction::Update),
            112 => Action::Thread(ThreadAction::Delete),
            140 => Action::AutoMod(AutoModAction::RuleCreate),
            141 => Action::AutoMod(AutoModAction::RuleUpdate),
            142 => Action::AutoMod(AutoModAction::RuleDelete),
            143 => Action::AutoMod(AutoModAction::BlockMessage),
            144 => Action::AutoMod(AutoModAction::FlagToChannel),
            145 => Action::AutoMod(AutoModAction::UserCommunicationDisabled),
            150 => Action::CreatorMonetization(CreatorMonetizationAction::RequestCreated),
            151 => Action::CreatorMonetization(CreatorMonetizationAction::TermsAccepted),
            192 => Action::VoiceChannelStatus(VoiceChannelStatusAction::StatusUpdate),
            193 => Action::VoiceChannelStatus(VoiceChannelStatusAction::StatusDelete),
            _ => Action::Unknown(value),
        }
    }
}

// Manual impl needed to emulate integer enum tags
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
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ChannelAction {
    Create = 10,
    Update = 11,
    Delete = 12,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ChannelOverwriteAction {
    Create = 13,
    Update = 14,
    Delete = 15,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
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
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum RoleAction {
    Create = 30,
    Update = 31,
    Delete = 32,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum InviteAction {
    Create = 40,
    Update = 41,
    Delete = 42,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum WebhookAction {
    Create = 50,
    Update = 51,
    Delete = 52,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum EmojiAction {
    Create = 60,
    Update = 61,
    Delete = 62,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum MessageAction {
    Delete = 72,
    BulkDelete = 73,
    Pin = 74,
    Unpin = 75,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum IntegrationAction {
    Create = 80,
    Update = 81,
    Delete = 82,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum StageInstanceAction {
    Create = 83,
    Update = 84,
    Delete = 85,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum StickerAction {
    Create = 90,
    Update = 91,
    Delete = 92,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ScheduledEventAction {
    Create = 100,
    Update = 101,
    Delete = 102,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum ThreadAction {
    Create = 110,
    Update = 111,
    Delete = 112,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum AutoModAction {
    RuleCreate = 140,
    RuleUpdate = 141,
    RuleDelete = 142,
    BlockMessage = 143,
    FlagToChannel = 144,
    UserCommunicationDisabled = 145,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-audit-log-events).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum CreatorMonetizationAction {
    RequestCreated = 150,
    TermsAccepted = 151,
}

/// [Incomplete documentation](https://github.com/discord/discord-api-docs/pull/6398)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum VoiceChannelStatusAction {
    StatusUpdate = 192,
    StatusDelete = 193,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AuditLogs {
    /// List of audit log entries, sorted from most to least recent.
    #[serde(rename = "audit_log_entries")]
    pub entries: FixedArray<AuditLogEntry>,
    /// List of auto moderation rules referenced in the audit log.
    pub auto_moderation_rules: FixedArray<Rule>,
    /// List of application commands referenced in the audit log.
    pub application_commands: FixedArray<Command>,
    /// List of guild scheduled events referenced in the audit log.
    pub guild_scheduled_events: FixedArray<ScheduledEvent>,
    /// List of partial integration objects.
    pub integrations: FixedArray<PartialIntegration>,
    /// List of threads referenced in the audit log.
    ///
    /// Threads referenced in THREAD_CREATE and THREAD_UPDATE events are included in the threads
    /// map since archived threads might not be kept in memory by clients.
    pub threads: FixedArray<GuildChannel>,
    /// List of users referenced in the audit log.
    pub users: ExtractMap<UserId, User>,
    /// List of webhooks referenced in the audit log.
    pub webhooks: ExtractMap<WebhookId, Webhook>,
}

/// Partial version of [`Integration`], used in [`AuditLogs::integrations`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-object-example-partial-integration-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialIntegration {
    pub id: IntegrationId,
    pub name: FixedString,
    #[serde(rename = "type")]
    pub kind: FixedString,
    pub account: IntegrationAccount,
    pub application: Option<IntegrationApplication>,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[non_exhaustive]
pub struct AuditLogEntry {
    /// Determines to what entity an [`Self::action`] was used on.
    pub target_id: Option<GenericId>,
    /// Determines what action was done on a [`Self::target_id`]
    #[serde(rename = "action_type")]
    pub action: Action,
    /// What was the reasoning by doing an action on a target? If there was one.
    pub reason: Option<FixedString>,
    /// The user that did this action on a target.
    pub user_id: Option<UserId>,
    /// What changes were made.
    #[serde(default)]
    pub changes: Vec<Change>,
    /// The id of this entry.
    pub id: AuditLogEntryId,
    /// Some optional data associated with this entry.
    pub options: Option<Options>,
}

/// [Discord docs](https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-optional-audit-entry-info).
// TODO: should be renamed to a less ambiguous name
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[non_exhaustive]
pub struct Options {
    /// Name of the Auto Moderation rule that was triggered.
    pub auto_moderation_rule_name: Option<FixedString>,
    /// Trigger type of the Auto Moderation rule that was triggered.
    pub auto_moderation_rule_trigger_type: Option<FixedString>,
    /// ID of the app whose permissions were targeted.
    pub application_id: Option<ApplicationId>,
    /// Number of days after which inactive members were kicked.
    #[serde(default, with = "optional_string")]
    pub delete_member_days: Option<NonMaxU32>,
    /// Number of members removed by the prune
    #[serde(default, with = "optional_string")]
    pub members_removed: Option<NonMaxU64>,
    /// Channel in which the messages were deleted
    #[serde(default)]
    pub channel_id: Option<ChannelId>,
    /// Number of deleted messages.
    #[serde(default, with = "optional_string")]
    pub count: Option<NonMaxU64>,
    /// Id of the overwritten entity
    #[serde(default)]
    pub id: Option<GenericId>,
    /// Type of overwritten entity ("member" or "role").
    #[serde(default, rename = "type")]
    pub kind: Option<FixedString>,
    /// Message that was pinned or unpinned.
    #[serde(default)]
    pub message_id: Option<MessageId>,
    /// Name of the role if type is "role"
    #[serde(default)]
    pub role_name: Option<FixedString>,
    /// The status of a voice channel when set.
    #[serde(default)]
    pub status: Option<FixedString>,
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
        assert_action!(Action::AutoMod(AutoModAction::RuleCreate), 140);
        assert_action!(Action::AutoMod(AutoModAction::RuleUpdate), 141);
        assert_action!(Action::AutoMod(AutoModAction::RuleDelete), 142);
        assert_action!(Action::AutoMod(AutoModAction::BlockMessage), 143);
        assert_action!(Action::AutoMod(AutoModAction::FlagToChannel), 144);
        assert_action!(Action::AutoMod(AutoModAction::UserCommunicationDisabled), 145);
        assert_action!(Action::CreatorMonetization(CreatorMonetizationAction::RequestCreated), 150);
        assert_action!(Action::CreatorMonetization(CreatorMonetizationAction::TermsAccepted), 151);
        assert_action!(Action::VoiceChannelStatus(VoiceChannelStatusAction::StatusUpdate), 192);
        assert_action!(Action::VoiceChannelStatus(VoiceChannelStatusAction::StatusDelete), 193);
        assert_action!(Action::Unknown(234), 234);
    }

    #[test]
    fn action_serde() {
        use serde_json::{from_value, json};

        #[derive(Debug, Deserialize, Serialize)]
        struct T {
            action: Action,
        }

        let value = json!({
            "action": 234,
        });

        let value = from_value::<T>(value).unwrap();
        assert_eq!(value.action.num(), 234);

        assert!(matches!(value.action, Action::Unknown(234)));
    }
}
