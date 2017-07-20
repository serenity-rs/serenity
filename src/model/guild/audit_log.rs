use super::super::{UserId, AuditLogEntryId};
use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};
use std::fmt;
use std::collections::HashMap;

/// Determines to what entity an action was used on.
pub enum Target {
    Guild,
    Channel,
    User,
    Role,
    Invite,
    Webhook,
    Emoji,
}

/// Determines what the action was done on a target.
pub enum Action {
    GuildUpdate,
    ChannelCreate,
    ChannelUpdate,
    ChannelDelete,
    ChannelOverwriteCreate,
    ChannelOverwriteUpdate,
    ChannelOverwriteDelete,
    MemberKick,
    MemberPrune,
    MemberBanAdd,
    MemberBanRemove,
    MemberUpdate,
    MemberRoleUpdate,
    RoleCreate,
    RoleUpdate,
    RoleDelete,
    InviteCreate,
    InviteUpdate,
    InviteDelete,
    WebhookCreate,
    WebhookUpdate,
    WebhookDelete,
    EmojiCreate,
    EmojiUpdate,
    EmojiDelete,
}

#[derive(Deserialize)]
pub struct Change {
    #[serde(rename="key")]
    pub name: String,
    #[serde(rename="old_value")]
    pub old: String,
    #[serde(rename="new_value")]
    pub new: String,
}

pub struct AuditLogs {
    pub entries: HashMap<AuditLogEntryId, AuditLogEntry>,
}

#[derive(Deserialize)]
pub struct AuditLogEntry {
    /// Determines to what entity an [`action`] was used on.
    ///
    /// [`action`]: #structfield.action
    #[serde(deserialize_with="deserialize_target", rename="target_type")] 
    pub target: Target,
    /// Determines what action was done on a [`target`]
    ///
    /// [`target`]: #structfield.target
    #[serde(deserialize_with="deserialize_action", rename="action_type")] 
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
            Ok(if value < 10 {
                Target::Guild
            } else if value < 20 {
                Target::Channel
            } else if value < 30 {
                Target::User
            } else if value < 40 {
                Target::Role
            } else if value < 50 {
                Target::Invite
            } else if value < 60 {
                Target::Webhook
            } else if value < 70 {
                Target::Emoji
            } else {
                return Err(E::custom(format!("Unexpected target number: {}", value)));
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
            // todo: improve this
            Ok(if value == 1 {
                Action::GuildUpdate
            } else if value == 10 {
                Action::ChannelCreate
            } else if value == 11 {
                Action::ChannelUpdate
            } else if value == 12 {
                Action::ChannelDelete
            } else if value == 13 {
                Action::ChannelOverwriteCreate
            } else if value == 14 {
                Action::ChannelOverwriteUpdate
            } else if value == 15 {
                Action::ChannelOverwriteDelete
            } else if value == 20 {
                Action::MemberKick
            } else if value == 21 {
                Action::MemberPrune
            } else if value == 22 {
                Action::MemberBanAdd
            } else if value == 23 {
                Action::MemberBanRemove
            } else if value == 24 {
                Action::MemberUpdate
            } else if value == 25 {
                Action::MemberRoleUpdate
            } else if value == 30 {
                Action::RoleCreate
            } else if value == 31 {
                Action::RoleUpdate
            } else if value == 32 {
                Action::RoleDelete
            } else if value == 40 {
                Action::InviteCreate
            } else if value == 41 {
                Action::InviteUpdate
            } else if value == 42 {
                Action::InviteDelete
            } else if value == 50 {
                Action::WebhookCreate
            } else if value == 51 {
                Action::WebhookUpdate
            } else if value == 52 {
                Action::WebhookDelete
            } else if value == 60 {
                Action::EmojiCreate
            } else if value == 61 {
                Action::EmojiUpdate
            } else if value == 62 {
                Action::EmojiDelete
            } else {
                return Err(E::custom(format!("Unexpected action number: {}", value)));
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
            #[serde(rename="audit_log_entries")] 
            AuditLogEntries 
        }

        struct EntriesVisitor;
    
        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = AuditLogs;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("audit log entries")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<AuditLogs, V::Error> {
                let mut audit_log_entries = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::AuditLogEntries => {
                            if audit_log_entries.is_some() {
                                return Err(de::Error::duplicate_field("audit_log_entries"));
                            }

                            audit_log_entries = Some(map.next_value()?);
                        }
                    }
                }

                let entries: Vec<AuditLogEntry> = audit_log_entries.ok_or_else(|| de::Error::missing_field("audit_log_entries"))?;

                Ok(AuditLogs { entries: entries.into_iter().map(|entry| (entry.id, entry)).collect() })
            }
        }

        const FIELD: &'static [&'static str] = &["audit_log_entries"];
        de.deserialize_struct("AuditLogs", FIELD, EntriesVisitor)
    }
}