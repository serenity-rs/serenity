use std::fmt;

use serde::de::{Deserializer, Error as DeError, IgnoredAny, MapAccess};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

#[cfg(feature = "http")]
use crate::builder::CreateAutocompleteResponse;
#[cfg(feature = "http")]
use crate::http::Http;
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::json;
use crate::json::prelude::*;
use crate::model::application::command::{CommandOptionType, CommandType};
#[cfg(feature = "http")]
use crate::model::application::interaction::InteractionResponseType;
use crate::model::application::interaction::InteractionType;
use crate::model::guild::Member;
use crate::model::id::{
    ApplicationId,
    AttachmentId,
    ChannelId,
    CommandId,
    GenericId,
    GuildId,
    InteractionId,
    RoleId,
    UserId,
};
use crate::model::user::User;

/// An interaction received when the user fills in an autocomplete option
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object).
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct AutocompleteInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The type of interaction.
    #[serde(rename = "type")]
    pub kind: InteractionType,
    /// The data of the interaction which was triggered.
    pub data: AutocompleteData,
    /// The guild Id this interaction was sent from, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// The channel Id this interaction was sent from.
    pub channel_id: ChannelId,
    /// The `member` data for the invoking user.
    ///
    /// **Note**: It is only present if the interaction is triggered in a guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Member>,
    /// The `user` object for the invoking user.
    pub user: User,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Always `1`.
    pub version: u8,
    /// The selected language of the invoking user.
    pub locale: String,
    /// The guild's preferred locale.
    pub guild_locale: Option<String>,
}

#[cfg(feature = "http")]
impl AutocompleteInteraction {
    /// Creates a response to an autocomplete interaction.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    pub async fn create_autocomplete_response<F>(&self, http: impl AsRef<Http>, f: F) -> Result<()>
    where
        F: FnOnce(&mut CreateAutocompleteResponse) -> &mut CreateAutocompleteResponse,
    {
        let mut response = CreateAutocompleteResponse::default();
        f(&mut response);
        let data = json::hashmap_to_json_map(response.0);

        let map = json!({
            "type": InteractionResponseType::Autocomplete as u8,
            "data": data,
        });

        http.as_ref().create_interaction_response(self.id.0, &self.token, &map).await
    }
}

impl<'de> Deserialize<'de> for AutocompleteInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("guild_id").and_then(Value::as_str).and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(member) = map.get_mut("member").and_then(Value::as_object_mut) {
                member.insert("guild_id".to_string(), from_number(guild_id));
            }

            if let Some(data) = map.get_mut("data") {
                if let Some(resolved) = data.get_mut("resolved") {
                    if let Some(roles) = resolved.get_mut("roles") {
                        if let Some(values) = roles.as_object_mut() {
                            for value in values.values_mut() {
                                value.as_object_mut().expect("couldn't deserialize").insert(
                                    "guild_id".to_string(),
                                    Value::from(guild_id.to_string()),
                                );
                            }
                        }
                    }
                }
            }
        }

        let id = map
            .remove("id")
            .ok_or_else(|| DeError::custom("expected id"))
            .and_then(InteractionId::deserialize)
            .map_err(DeError::custom)?;

        let application_id = map
            .remove("application_id")
            .ok_or_else(|| DeError::custom("expected application id"))
            .and_then(ApplicationId::deserialize)
            .map_err(DeError::custom)?;

        let kind = map
            .remove("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(InteractionType::deserialize)
            .map_err(DeError::custom)?;

        let data = map
            .remove("data")
            .ok_or_else(|| DeError::custom("expected data"))
            .and_then(AutocompleteData::deserialize)
            .map_err(DeError::custom)?;

        let guild_id = map
            .remove("guild_id")
            .map(GuildId::deserialize)
            .transpose()
            .map_err(DeError::custom)?;

        let channel_id = map
            .remove("channel_id")
            .ok_or_else(|| DeError::custom("expected channel_id"))
            .and_then(ChannelId::deserialize)
            .map_err(DeError::custom)?;

        let member =
            map.remove("member").map(Member::deserialize).transpose().map_err(DeError::custom)?;

        let user =
            map.remove("user").map(User::deserialize).transpose().map_err(DeError::custom)?;

        let user = user
            .or_else(|| member.as_ref().map(|m| m.user.clone()))
            .ok_or_else(|| DeError::custom("expected user or member"))?;

        let token = map
            .remove("token")
            .ok_or_else(|| DeError::custom("expected token"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        let version = map
            .remove("version")
            .ok_or_else(|| DeError::custom("expected version"))
            .and_then(u8::deserialize)
            .map_err(DeError::custom)?;

        let guild_locale = map
            .remove("guild_locale")
            .map(String::deserialize)
            .transpose()
            .map_err(DeError::custom)?;

        let locale = map
            .remove("locale")
            .ok_or_else(|| DeError::custom("expected locale"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        Ok(Self {
            id,
            application_id,
            kind,
            data,
            guild_id,
            channel_id,
            member,
            user,
            token,
            version,
            locale,
            guild_locale,
        })
    }
}

/// The autocomplete data payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AutocompleteData {
    /// The Id of the invoked command.
    pub id: CommandId,
    /// The name of the invoked command.
    pub name: String,
    /// The application command type of the triggered application command.
    #[serde(rename = "type")]
    pub kind: CommandType,
    /// The parameters and the given values.
    #[serde(default)]
    pub options: Vec<CommandDataOption>,
    /// The Id of the guild the command is registered to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
}

impl AutocompleteData {
    /// Returns the focused option from [`AutocompleteData::options`].
    #[must_use]
    pub fn focused_option(&self) -> Option<FocusedOption<'_>> {
        fn find_option(opts: &[CommandDataOption]) -> Option<FocusedOption<'_>> {
            for opt in opts {
                match &opt.value {
                    CommandDataOptionValue::SubCommand(opts)
                    | CommandDataOptionValue::SubCommandGroup(opts) => {
                        let opt = find_option(&*opts);
                        if opt.is_some() {
                            return opt;
                        }
                    },
                    CommandDataOptionValue::Autocomplete {
                        kind,
                        value,
                    } => {
                        return Some(FocusedOption {
                            name: &opt.name,
                            kind: *kind,
                            value,
                        });
                    },
                    _ => {},
                }
            }
            None
        }
        find_option(&*self.options)
    }
}

/// The focused option return by [`AutocompleteData::focused_option`].
#[derive(Clone, Debug)]
pub struct FocusedOption<'a> {
    pub name: &'a str,
    pub kind: CommandOptionType,
    pub value: &'a str,
}

/// A set of a parameter and a value from the user.
///
/// All options have names and an option can either be a parameter and input `value` or it can
/// denote a sub-command or group, in which case it will contain a top-level key and another vector
/// of `options`.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct CommandDataOption {
    /// The name of the parameter.
    pub name: String,
    /// The given value.
    pub value: CommandDataOptionValue,
}

impl CommandDataOption {
    /// Returns the value type.
    #[must_use]
    pub fn kind(&self) -> CommandOptionType {
        self.value.kind()
    }
}

impl<'de> Deserialize<'de> for CommandDataOption {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Name,
            Type,
            Value,
            Options,
            Focused,
            Unknown(String),
        }

        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = CommandDataOption;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("CommandDataOption")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> StdResult<Self::Value, A::Error> {
                let mut name = None;
                let mut kind = None;
                let mut value = None;
                let mut options = None;
                let mut focused = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(DeError::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        },
                        Field::Type => {
                            if kind.is_some() {
                                return Err(DeError::duplicate_field("type"));
                            }
                            kind = Some(map.next_value()?);
                        },
                        Field::Value => {
                            if value.is_some() {
                                return Err(DeError::duplicate_field("value"));
                            }
                            value = Some(map.next_value::<serde_value::Value>()?);
                        },
                        Field::Options => {
                            if options.is_some() {
                                return Err(DeError::duplicate_field("options"));
                            }
                            options = Some(map.next_value()?);
                        },
                        Field::Focused => {
                            if focused.is_some() {
                                return Err(DeError::duplicate_field("focused"));
                            }
                            focused = Some(map.next_value()?);
                        },
                        Field::Unknown(_) => {
                            map.next_value::<IgnoredAny>()?;
                        },
                    }
                }

                let name = name.ok_or_else(|| DeError::missing_field("name"))?;
                let kind = kind.ok_or_else(|| DeError::missing_field("type"))?;
                let focused = focused.unwrap_or_default();

                if focused {
                    let value = value.ok_or_else(|| DeError::missing_field("value"))?;
                    let value = String::deserialize(value).map_err(DeError::custom)?;
                    return Ok(CommandDataOption {
                        name,
                        value: CommandDataOptionValue::Autocomplete {
                            kind,
                            value,
                        },
                    });
                }

                let value = match kind {
                    CommandOptionType::Boolean => {
                        let value = value.ok_or_else(|| DeError::missing_field("value"))?;
                        let value = bool::deserialize(value).map_err(DeError::custom)?;
                        CommandDataOptionValue::Boolean(value)
                    },
                    CommandOptionType::Integer => {
                        let value = value.ok_or_else(|| DeError::missing_field("value"))?;
                        let value = i64::deserialize(value).map_err(DeError::custom)?;
                        CommandDataOptionValue::Integer(value)
                    },
                    CommandOptionType::Number => {
                        let value = value.ok_or_else(|| DeError::missing_field("value"))?;
                        let value = f64::deserialize(value).map_err(DeError::custom)?;
                        CommandDataOptionValue::Number(value)
                    },
                    CommandOptionType::String => {
                        let value = value.ok_or_else(|| DeError::missing_field("value"))?;
                        let value = String::deserialize(value).map_err(DeError::custom)?;
                        CommandDataOptionValue::String(value)
                    },
                    CommandOptionType::SubCommand => {
                        let options = options.ok_or_else(|| DeError::missing_field("options"))?;
                        CommandDataOptionValue::SubCommand(options)
                    },
                    CommandOptionType::SubCommandGroup => {
                        let options = options.ok_or_else(|| DeError::missing_field("options"))?;
                        CommandDataOptionValue::SubCommandGroup(options)
                    },
                    CommandOptionType::Attachment => {
                        let value = value.ok_or_else(|| DeError::missing_field("value"))?;
                        let value = AttachmentId::deserialize(value).map_err(DeError::custom)?;
                        CommandDataOptionValue::Attachment(value)
                    },
                    CommandOptionType::Channel => {
                        let value = value.ok_or_else(|| DeError::missing_field("value"))?;
                        let value = ChannelId::deserialize(value).map_err(DeError::custom)?;
                        CommandDataOptionValue::Channel(value)
                    },
                    CommandOptionType::Mentionable => {
                        let value = value.ok_or_else(|| DeError::missing_field("value"))?;
                        let value = GenericId::deserialize(value).map_err(DeError::custom)?;
                        CommandDataOptionValue::Mentionable(value)
                    },
                    CommandOptionType::Role => {
                        let value = value.ok_or_else(|| DeError::missing_field("value"))?;
                        let value = RoleId::deserialize(value).map_err(DeError::custom)?;
                        CommandDataOptionValue::Role(value)
                    },
                    CommandOptionType::User => {
                        let value = value.ok_or_else(|| DeError::missing_field("value"))?;
                        let value = UserId::deserialize(value).map_err(DeError::custom)?;
                        CommandDataOptionValue::User(value)
                    },
                    CommandOptionType::Unknown => CommandDataOptionValue::Unknown,
                };

                Ok(CommandDataOption {
                    name,
                    value,
                })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl Serialize for CommandDataOption {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        let (value_or_options, focused) = match &self.value {
            CommandDataOptionValue::Autocomplete {
                ..
            } => (true, true),
            CommandDataOptionValue::SubCommand(o) | CommandDataOptionValue::SubCommandGroup(o) => {
                (!o.is_empty(), false)
            },
            CommandDataOptionValue::Unknown => (false, false),
            _ => (true, false),
        };
        let len = 2 + usize::from(value_or_options) + usize::from(focused);

        let mut s = serializer.serialize_struct("CommandDataOption", len)?;

        s.serialize_field("name", &self.name)?;
        s.serialize_field("type", &self.value.kind())?;

        match &self.value {
            CommandDataOptionValue::Autocomplete {
                value, ..
            } => {
                s.serialize_field("value", value)?;
            },
            CommandDataOptionValue::Boolean(v) => s.serialize_field("value", v)?,
            CommandDataOptionValue::Integer(v) => s.serialize_field("value", v)?,
            CommandDataOptionValue::Number(v) => s.serialize_field("value", v)?,
            CommandDataOptionValue::String(v) => s.serialize_field("value", v)?,
            CommandDataOptionValue::Attachment(v) => s.serialize_field("value", v)?,
            CommandDataOptionValue::Channel(v) => s.serialize_field("value", v)?,
            CommandDataOptionValue::Mentionable(v) => s.serialize_field("value", v)?,
            CommandDataOptionValue::Role(v) => s.serialize_field("value", v)?,
            CommandDataOptionValue::User(v) => s.serialize_field("value", v)?,
            CommandDataOptionValue::SubCommand(o) | CommandDataOptionValue::SubCommandGroup(o) => {
                if !o.is_empty() {
                    s.serialize_field("options", o)?;
                }
            },
            _ => {},
        }

        if focused {
            s.serialize_field("focused", &focused)?;
        }

        s.end()
    }
}

/// The value of an [`CommandDataOption`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CommandDataOptionValue {
    Autocomplete { kind: CommandOptionType, value: String },
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    SubCommand(Vec<CommandDataOption>),
    SubCommandGroup(Vec<CommandDataOption>),
    Attachment(AttachmentId),
    Channel(ChannelId),
    Mentionable(GenericId),
    Role(RoleId),
    User(UserId),
    Unknown,
}

impl CommandDataOptionValue {
    /// Returns the value type.
    #[must_use]
    pub fn kind(&self) -> CommandOptionType {
        match self {
            Self::Autocomplete {
                kind, ..
            } => *kind,
            Self::Boolean(_) => CommandOptionType::Boolean,
            Self::Integer(_) => CommandOptionType::Integer,
            Self::Number(_) => CommandOptionType::Number,
            Self::String(_) => CommandOptionType::String,
            Self::SubCommand(_) => CommandOptionType::SubCommand,
            Self::SubCommandGroup(_) => CommandOptionType::SubCommandGroup,
            Self::Attachment(_) => CommandOptionType::Attachment,
            Self::Channel(_) => CommandOptionType::Channel,
            Self::Mentionable(_) => CommandOptionType::Mentionable,
            Self::Role(_) => CommandOptionType::Role,
            Self::User(_) => CommandOptionType::User,
            Self::Unknown => CommandOptionType::Unknown,
        }
    }

    /// If the value is a boolean, returns the associated f64. Returns None otherwise.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Self::Boolean(b) => Some(b),
            _ => None,
        }
    }

    /// If the value is an integer, returns the associated f64. Returns None otherwise.
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Self::Integer(v) => Some(v),
            _ => None,
        }
    }

    /// If the value is a number, returns the associated f64. Returns None otherwise.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Self::Number(v) => Some(v),
            _ => None,
        }
    }

    /// If the value is a string, returns the associated str. Returns None otherwise.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            Self::Autocomplete {
                value, ..
            } => Some(value),
            _ => None,
        }
    }

    /// If the value is an `AttachmentId`, returns the associated ID. Returns None otherwise.
    #[must_use]
    pub fn as_attachment_id(&self) -> Option<AttachmentId> {
        match self {
            Self::Attachment(id) => Some(*id),
            _ => None,
        }
    }

    /// If the value is an `ChannelId`, returns the associated ID. Returns None otherwise.
    #[must_use]
    pub fn as_channel_id(&self) -> Option<ChannelId> {
        match self {
            Self::Channel(id) => Some(*id),
            _ => None,
        }
    }

    /// If the value is an `GenericId`, returns the associated ID. Returns None otherwise.
    #[must_use]
    pub fn as_mentionable(&self) -> Option<GenericId> {
        match self {
            Self::Mentionable(id) => Some(*id),
            _ => None,
        }
    }

    /// If the value is an `UserId`, returns the associated ID. Returns None otherwise.
    #[must_use]
    pub fn as_user_id(&self) -> Option<UserId> {
        match self {
            Self::User(id) => Some(*id),
            _ => None,
        }
    }

    /// If the value is an `RoleId`, returns the associated ID. Returns None otherwise.
    #[must_use]
    pub fn as_role_id(&self) -> Option<RoleId> {
        match self {
            Self::Role(id) => Some(*id),
            _ => None,
        }
    }
}
