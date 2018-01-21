use internal::prelude::*;
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::Serializer;
use super::super::prelude::*;
use std::collections::HashMap;
use std::mem::transmute;
use std::fmt;

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
    MessageDelete,
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
            Action::MessageDelete => 72,
        }
    }
}

#[derive(Debug)]
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
#[repr(u8)]
pub enum ActionMember {
    Kick = 20,
    Prune = 21,
    BanAdd = 22,
    BanRemove = 23,
    Update = 24,
    RoleUpdate = 25,
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
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Change {
    #[serde(rename = "key")] pub name: String,
    // TODO: Change these to an actual type.
    #[serde(rename = "old_value")] pub old: String,
    #[serde(rename = "new_value")] pub new: String,
}

#[derive(Debug)]
pub struct AuditLogs {
    pub entries: HashMap<AuditLogEntryId, AuditLogEntry>,
    pub webhooks: Vec<Webhook>,
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuditLogEntry {
    /// Determines to what entity an [`action`] was used on.
    ///
    /// [`action`]: #structfield.action
    pub target_id: u64,
    /// Determines what action was done on a [`target`]
    ///
    /// [`target`]: #structfield.target
    #[serde(deserialize_with = "deserialize_action",
            rename = "action_type",
            serialize_with = "serialize_action")]
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
pub struct Options {
    /// Number of days after which inactive members were kicked.
    pub delete_member_days: String,
    /// Number of members removed by the prune
    pub members_removed: String,
    /// Channel in which the messages were deleted
    pub channel_id: ChannelId,
    /// Number of deleted messages.
    pub count: u32,
    /// Id of the overwritten entity
    pub id: u64,
    /// Type of overwritten entity ("member" or "role").
    #[serde(rename = "type")] pub kind: String,
    /// Name of the role if type is "role"
    pub role_name: String,
}

fn deserialize_action<'de, D: Deserializer<'de>>(de: D) -> StdResult<Action, D::Error> {
    struct ActionVisitor;

    impl<'de> Visitor<'de> for ActionVisitor {
        type Value = Action;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer between 1 to 72")
        }

        fn visit_u8<E: de::Error>(self, value: u8) -> StdResult<Action, E> {
            Ok(match value {
                1 => Action::GuildUpdate,
                10...12 => Action::Channel(unsafe { transmute(value) }),
                13...15 => Action::ChannelOverwrite(unsafe { transmute(value) }),
                20...25 => Action::Member(unsafe { transmute(value) }),
                30...32 => Action::Role(unsafe { transmute(value) }),
                40...42 => Action::Invite(unsafe { transmute(value) }),
                50...52 => Action::Webhook(unsafe { transmute(value) }),
                60...62 => Action::Emoji(unsafe { transmute(value) }),
                72 => Action::MessageDelete,
                _ => return Err(E::custom(format!("Unexpected action number: {}", value))),
            })
        }
    }

    de.deserialize_u8(ActionVisitor)
}

fn serialize_action<S: Serializer>(
    action: &Action,
    serializer: S,
) -> StdResult<S::Ok, S::Error> {
    serializer.serialize_u8(action.num())
}

impl<'de> Deserialize<'de> for AuditLogs {
    fn deserialize<D: Deserializer<'de>>(de: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            #[serde(rename = "audit_log_entries")] Entries,
            #[serde(rename = "webhooks")] Webhooks,
            #[serde(rename = "users")] Users,
        }

        struct EntriesVisitor;

        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = AuditLogs;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("audit log entries")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> StdResult<AuditLogs, V::Error> {
                let mut audit_log_entries = None;
                let mut users = None;
                let mut webhooks = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::Entries => {
                            if audit_log_entries.is_some() {
                                return Err(de::Error::duplicate_field("entries"));
                            }

                            audit_log_entries = Some(map.next_value::<Vec<AuditLogEntry>>()?);
                        },
                        Field::Webhooks => {
                            if webhooks.is_some() {
                                return Err(de::Error::duplicate_field("webhooks"));
                            }

                            webhooks = Some(map.next_value::<Vec<Webhook>>()?);
                        },
                        Field::Users => {
                            if users.is_some() {
                                return Err(de::Error::duplicate_field("users"));
                            }

                            users = Some(map.next_value::<Vec<User>>()?);
                        },
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
                })
            }
        }

        const FIELD: &[&str] = &["audit_log_entries"];
        de.deserialize_struct("AuditLogs", FIELD, EntriesVisitor)
    }
}
