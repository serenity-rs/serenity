use super::super::{AuditLogEntryId, UserId};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use std::fmt;
use std::collections::HashMap;

trait FromNum {
    fn from_num(num: i32) -> Result<Self, String> where Self: Sized;
}

/// Determines to what entity an action was used on.
#[derive(Debug)]
pub enum Target {
    Guild,
    Channel,
    User,
    Role,
    Invite,
    Webhook,
    Emoji,
}

impl FromNum for Target {
    fn from_num(num: i32) -> Result<Self, String> {
        use self::Target::*;

        Ok(match num {
            10 => Guild,
            20 => Channel,
            30 => User,
            40 => Role,
            50 => Invite,
            60 => Webhook,
            70 => Emoji,
            _ => return Err(format!("Unexpected target number: {}", num)),
        })
    }
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

impl FromNum for Action {
    fn from_num(num: i32) -> Result<Self, String> {
        Ok(match num {
            1 => Action::GuildUpdate,
            num if num >= 10 && num <= 13 => Action::Channel(ActionChannel::from_num(num)?),
            num if num >= 13 && num <= 15 => {
                Action::ChannelOverwrite(ActionChannelOverwrite::from_num(num)?)
            },
            num if num >= 20 && num <= 25 => Action::Member(ActionMember::from_num(num)?),
            num if num >= 30 && num <= 32 => Action::Role(ActionRole::from_num(num)?),
            num if num >= 40 && num <= 42 => Action::Invite(ActionInvite::from_num(num)?),
            num if num >= 50 && num <= 52 => Action::Webhook(ActionWebhook::from_num(num)?),
            num if num >= 60 && num <= 62 => Action::Emoji(ActionEmoji::from_num(num)?),
            _ => return Err(format!("Unexpected action number: {}", num)),
        })
    }
}

#[derive(Debug)]
pub enum ActionChannel {
    Create,
    Update,
    Delete,
}

impl FromNum for ActionChannel {
    fn from_num(num: i32) -> Result<Self, String> {
        use self::ActionChannel::*;

        Ok(match num {
            10 => Create, 
            11 => Update, 
            12 => Delete,
            _ => return Err(format!("Unexpected action channel num: {}", num)),
        })
    }
}

#[derive(Debug)]
pub enum ActionChannelOverwrite {
    Create,
    Update,
    Delete,
}

impl FromNum for ActionChannelOverwrite {
    fn from_num(num: i32) -> Result<Self, String> {
        use self::ActionChannelOverwrite::*;

        Ok(match num {
            13 => Create, 
            14 => Update, 
            15 => Delete,
            _ => return Err(format!("Unexpected action channel overwrite num: {}", num)),
        })
    }
}

#[derive(Debug)]
pub enum ActionMember {
    Kick,
    Prune,
    BanAdd,
    BanRemove,
    Update,
    RoleUpdate,
}

impl FromNum for ActionMember {
    fn from_num(num: i32) -> Result<Self, String> {
        use self::ActionMember::*;

        Ok(match num {
            20 => Kick,
            21 => Prune,
            22 => BanAdd, 
            23 => BanRemove, 
            24 => Update,
            25 => RoleUpdate,
            _ => return Err(format!("Unexpected action member num: {}", num)),
        })
    }
}

#[derive(Debug)]
pub enum ActionRole {
    Create,
    Update,
    Delete,
}

impl FromNum for ActionRole {
    fn from_num(num: i32) -> Result<Self, String> {
        use self::ActionRole::*;

        Ok(match num {
            30 => Create, 
            31 => Update, 
            32 => Delete,
            _ => return Err(format!("Unexpected action role num: {}", num)),
        })
    }
}

#[derive(Debug)]
pub enum ActionInvite {
    Create,
    Update,
    Delete,
}

impl FromNum for ActionInvite {
    fn from_num(num: i32) -> Result<Self, String> {
        use self::ActionInvite::*;

        Ok(match num {
            40 => Create, 
            41 => Update, 
            42 => Delete,
            _ => return Err(format!("Unexpected action invite num: {}", num)),
        })
    }
}

#[derive(Debug)]
pub enum ActionWebhook {
    Create,
    Update,
    Delete,
}

impl FromNum for ActionWebhook {
    fn from_num(num: i32) -> Result<Self, String> {
        use self::ActionWebhook::*;

        Ok(match num {
            50 => Create, 
            51 => Update, 
            52 => Delete,
            _ => return Err(format!("Unexpected action webhook num: {}", num)),
        })
    }
}

#[derive(Debug)]
pub enum ActionEmoji {
    Create,
    Delete,
    Update,
}

impl FromNum for ActionEmoji {
    fn from_num(num: i32) -> Result<Self, String> {
        use self::ActionEmoji::*;

        Ok(match num {
            60 => Create, 
            61 => Update, 
            62 => Delete,
            _ => return Err(format!("Unexpected action emoji num: {}", num)),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Change {
    #[serde(rename = "key")]
    pub name: String,
    #[serde(rename = "old_value")]
    pub old: String,
    #[serde(rename = "new_value")]
    pub new: String,
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
            Target::from_num(value).map_err(E::custom)
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
            Action::from_num(value).map_err(E::custom)
        }
    }

    de.deserialize_i32(ActionVisitor)
}

impl<'de> Deserialize<'de> for AuditLogs {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            #[serde(rename = "audit_log_entries")]
            Entries,
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
