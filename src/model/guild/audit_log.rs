use super::super::{AuditLogEntryId, UserId};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use std::fmt;
use std::collections::HashMap;
use std::mem::transmute;

/// Determines to what entity an action was used on.
#[derive(Debug)]
#[repr(i32)]
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
}

#[derive(Debug)]
#[repr(i32)]
pub enum ActionChannel {
    Create = 10,
    Update = 11,
    Delete = 12,
}

#[derive(Debug)]
#[repr(i32)]
pub enum ActionChannelOverwrite {
    Create = 13,
    Update = 14,
    Delete = 15,
}

#[derive(Debug)]
#[repr(i32)]
pub enum ActionMember {
    Kick = 20,
    Prune = 21,
    BanAdd = 22,
    BanRemove = 23,
    Update = 24,
    RoleUpdate = 25,
}

#[derive(Debug)]
#[repr(i32)]
pub enum ActionRole {
    Create = 30,
    Update = 31,
    Delete = 32,
}

#[derive(Debug)]
#[repr(i32)]
pub enum ActionInvite {
    Create = 40,
    Update = 41,
    Delete = 42,
}

#[derive(Debug)]
#[repr(i32)]
pub enum ActionWebhook {
    Create = 50,
    Update = 51,
    Delete = 52,
}

#[derive(Debug)]
#[repr(i32)]
pub enum ActionEmoji {
    Create = 60,
    Delete = 61,
    Update = 62,
}

#[derive(Debug, Deserialize)]
pub struct Change {
    #[serde(rename = "key")] pub name: String,
    #[serde(rename = "old_value")] pub old: String,
    #[serde(rename = "new_value")] pub new: String,
}

#[derive(Debug)]
pub struct AuditLogs {
    pub entries: HashMap<AuditLogEntryId, AuditLogEntry>,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogEntry {
    /// Determines to what entity an [`action`] was used on.
    ///
    /// [`action`]: #structfield.action
    #[serde(deserialize_with = "deserialize_target", rename = "target_type")]
    pub target: Target,
    /// Determines what action was done on a [`target`]
    ///
    /// [`target`]: #structfield.target
    #[serde(deserialize_with = "deserialize_action", rename = "action_type")]
    pub action: Action,
    /// What was the reasoning by doing an action on a target? If there was one.
    pub reason: Option<String>,
    /// The user that did this action on a target.
    pub user_id: UserId,
    /// What changes were made.
    pub changes: Vec<Change>,
    /// The id of this entry.
    pub id: AuditLogEntryId,
}

fn deserialize_target<'de, D: Deserializer<'de>>(de: D) -> Result<Target, D::Error> {
    struct TargetVisitor;

    impl<'de> Visitor<'de> for TargetVisitor {
        type Value = Target;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer between 0 to 70")
        }

        fn visit_i32<E: de::Error>(self, value: i32) -> Result<Target, E> {
            Ok(match value {
                10...70 => unsafe { transmute(value) },
                _ => return Err(E::custom(format!("unexpected target number: {}", value))),
            })
        }
    }

    de.deserialize_i32(TargetVisitor)
}

fn deserialize_action<'de, D: Deserializer<'de>>(de: D) -> Result<Action, D::Error> {
    struct ActionVisitor;

    impl<'de> Visitor<'de> for ActionVisitor {
        type Value = Action;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer between 1 to 62")
        }

        fn visit_i32<E: de::Error>(self, value: i32) -> Result<Action, E> {
            Ok(match value {
                1 => Action::GuildUpdate,
                10...12 => Action::Channel(unsafe { transmute(value) }),
                13...15 => Action::ChannelOverwrite(unsafe { transmute(value) }),
                20...25 => Action::Member(unsafe { transmute(value) }),
                30...32 => Action::Role(unsafe { transmute(value) }),
                40...42 => Action::Invite(unsafe { transmute(value) }),
                50...52 => Action::Webhook(unsafe { transmute(value) }),
                60...62 => Action::Emoji(unsafe { transmute(value) }),
                _ => return Err(E::custom(format!("Unexpected action number: {}", value))),
            })
        }
    }

    de.deserialize_i32(ActionVisitor)
}

impl<'de> Deserialize<'de> for AuditLogs {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            #[serde(rename = "audit_log_entries")] Entries,
        }

        struct EntriesVisitor;

        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = AuditLogs;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("audit log entries")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<AuditLogs, V::Error> {
                let audit_log_entries = loop {
                    if let Some(Field::Entries) = map.next_key()? {
                        break map.next_value::<Vec<AuditLogEntry>>()?;
                    }
                };

                Ok(AuditLogs {
                    entries: audit_log_entries
                        .into_iter()
                        .map(|entry| (entry.id, entry))
                        .collect(),
                })
            }
        }

        const FIELD: &'static [&'static str] = &["audit_log_entries"];
        de.deserialize_struct("AuditLogs", FIELD, EntriesVisitor)
    }
}
