use crate::internal::prelude::*;
use serde::de::{
    self,
    Deserialize,
    Deserializer,
    MapAccess,
    Visitor
};
use serde::ser::Serializer;
use super::super::prelude::*;
use std::{
    collections::HashMap,
    mem::transmute,
    fmt,
};

/// Determines to what entity an action was used on.
#[derive(Debug)]
#[repr(u8)]
pub enum Target {
    Guild = 10,
    Channel = 20,
    User = 30,
    Role = 40,
    Invite = 50,
    Webhook = 60,
    Emoji = 70,
    Integration = 80,
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Determines the action that was done on a target.
#[derive(Debug)]
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
    #[doc(hidden)]
    __Nonexhaustive,
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
            Action::__Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum ActionChannel {
    Create = 10,
    Update = 11,
    Delete = 12,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ActionChannel {
    pub fn num(&self) -> u8 {
        match *self {
            ActionChannel::Create => 10,
            ActionChannel::Update => 11,
            ActionChannel::Delete => 12,
            ActionChannel::__Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum ActionChannelOverwrite {
    Create = 13,
    Update = 14,
    Delete = 15,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ActionChannelOverwrite {
    pub fn num(&self) -> u8 {
        match *self {
            ActionChannelOverwrite::Create => 13,
            ActionChannelOverwrite::Update => 14,
            ActionChannelOverwrite::Delete => 15,
            ActionChannelOverwrite::__Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Debug)]
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
    #[doc(hidden)]
    __Nonexhaustive,
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
            ActionMember::__Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum ActionRole {
    Create = 30,
    Update = 31,
    Delete = 32,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ActionRole {
    pub fn num(&self) -> u8 {
        match *self {
            ActionRole::Create => 30,
            ActionRole::Update => 31,
            ActionRole::Delete => 32,
            ActionRole::__Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum ActionInvite {
    Create = 40,
    Update = 41,
    Delete = 42,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ActionInvite {
    pub fn num(&self) -> u8 {
        match *self {
            ActionInvite::Create => 40,
            ActionInvite::Update => 41,
            ActionInvite::Delete => 42,
            ActionInvite::__Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum ActionWebhook {
    Create = 50,
    Update = 51,
    Delete = 52,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ActionWebhook {
    pub fn num(&self) -> u8 {
        match *self {
            ActionWebhook::Create => 50,
            ActionWebhook::Update => 51,
            ActionWebhook::Delete => 52,
            ActionWebhook::__Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum ActionEmoji {
    Create = 60,
    Delete = 61,
    Update = 62,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ActionEmoji {
    pub fn num(&self) -> u8 {
        match *self {
            ActionEmoji::Create => 60,
            ActionEmoji::Update => 61,
            ActionEmoji::Delete => 62,
            ActionEmoji::__Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum ActionMessage {
    Delete = 72,
    BulkDelete = 73,
    Pin = 74,
    Unpin = 75,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ActionMessage {
    pub fn num(&self) -> u8 {
        match *self {
            ActionMessage::Delete => 72,
            ActionMessage::BulkDelete => 73,
            ActionMessage::Pin => 74,
            ActionMessage::Unpin => 75,
            ActionMessage::__Nonexhaustive => unreachable!(),
        }
    }
}


#[derive(Debug)]
#[repr(u8)]
pub enum ActionIntegration {
    Create = 80,
    Update = 81,
    Delete = 82,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ActionIntegration {
    pub fn num(&self) -> u8 {
        match *self {
            ActionIntegration::Create => 80,
            ActionIntegration::Update => 81,
            ActionIntegration::Delete => 82,
            ActionIntegration::__Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Change {
    #[serde(rename = "key")] pub name: String,
    // TODO: Change these to an actual type.
    #[serde(rename = "old_value")] pub old: Option<Value>,
    #[serde(rename = "new_value")] pub new: Option<Value>,
}

#[derive(Debug)]
pub struct AuditLogs {
    pub entries: HashMap<AuditLogEntryId, AuditLogEntry>,
    pub webhooks: Vec<Webhook>,
    pub users: Vec<User>,
    pub(crate) _nonexhaustive: (),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuditLogEntry {
    /// Determines to what entity an [`action`] was used on.
    ///
    /// [`action`]: #structfield.action
    #[serde(with = "option_u64_handler")]
    pub target_id: Option<u64>,
    /// Determines what action was done on a [`target`]
    ///
    /// [`target`]: #structfield.target
    #[serde(
        with = "action_handler",
        rename = "action_type"
    )]
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
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[derive(Debug, Deserialize, Serialize)]
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
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
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

            fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> StdResult<Self::Value, D::Error> {
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
                    _ => return Err(E::custom(format!("Unexpected action number: {}", value))),
                })
            }
        }

        de.deserialize_any(ActionVisitor)
    }

    pub fn serialize<S: Serializer>(
        action: &Action,
        serializer: S,
    ) -> StdResult<S::Ok, S::Error> {
        serializer.serialize_u8(action.num())
    }
}
impl<'de> Deserialize<'de> for AuditLogs {
    fn deserialize<D: Deserializer<'de>>(de: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            #[serde(rename = "audit_log_entries")] Entries,
            #[serde(rename = "webhooks")] Webhooks,
            #[serde(rename = "users")] Users,
            // TODO(field added by Discord, undocumented) #[serde(rename = "integrations")] Integrations,
        }

        struct EntriesVisitor;

        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = AuditLogs;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("audit log entries")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> StdResult<AuditLogs, V::Error> {
                let mut audit_log_entries = None;
                let mut users = None;
                let mut webhooks = None;

                loop {
                    match map.next_key() {
                        Ok(Some(Field::Entries)) => {
                            if audit_log_entries.is_some() {
                                return Err(de::Error::duplicate_field("entries"));
                            }

                            audit_log_entries = Some(map.next_value::<Vec<AuditLogEntry>>()?);
                        }
                        Ok(Some(Field::Webhooks)) => {
                            if webhooks.is_some() {
                                return Err(de::Error::duplicate_field("webhooks"));
                            }

                            webhooks = Some(map.next_value::<Vec<Webhook>>()?);
                        }
                        Ok(Some(Field::Users)) => {
                            if users.is_some() {
                                return Err(de::Error::duplicate_field("users"));
                            }

                            users = Some(map.next_value::<Vec<User>>()?);
                        }
                        Ok(None) => break, // No more keys
                        Err(e) => if e.to_string().contains("unknown field") {
                            // e is of type <V as MapAccess>::Error, which is a macro-defined trait, ultimately
                            // implemented by serde::de::value::Error. Seeing as every error is a simple string and not
                            // using a proper Error num, the best we can do here is to check if the string contains
                            // this error. This was added because Discord randomly started sending new fields.
                            // But no JSON deserializer should ever error over this.
                            map.next_value::<serde_json::Value>()?; // Actually read the value to avoid syntax errors
                        } else {
                            return Err(e)
                        }
                    }
                }

                Ok(AuditLogs {
                    entries: audit_log_entries
                        .unwrap()
                        .into_iter()
                        .map(|entry| (entry.id, entry))
                        .collect(),
                    webhooks: webhooks.unwrap(),
                    users: users.unwrap(),
                    _nonexhaustive: (),
                })
            }
        }

        const FIELD: &[&str] = &["audit_log_entries"];
        de.deserialize_struct("AuditLogs", FIELD, EntriesVisitor)
    }
}
