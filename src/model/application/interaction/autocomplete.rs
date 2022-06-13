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
#[cfg(feature = "http")]
use crate::json::prelude::*;
use crate::model::application::command::{CommandOptionType, CommandType};
use crate::model::application::interaction::add_guild_id_to_resolved;
#[cfg(feature = "http")]
use crate::model::application::interaction::InteractionResponseType;
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
use crate::model::utils::{deserialize_val, remove_from_map, remove_from_map_opt};

/// An interaction received when the user fills in an autocomplete option
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct AutocompleteInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
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
    /// The guild's preferred locale.
    pub guild_locale: Option<String>,
    /// The selected language of the invoking user.
    pub locale: String,
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

        let guild_id = remove_from_map_opt(&mut map, "guild_id")?;

        if let Some(guild_id) = guild_id {
            add_guild_id_to_resolved(&mut map, guild_id);
        }

        let member = remove_from_map_opt::<Member, _>(&mut map, "member")?;
        let user = remove_from_map_opt(&mut map, "user")?
            .or_else(|| member.as_ref().map(|m| m.user.clone()))
            .ok_or_else(|| DeError::custom("expected user or member"))?;

        Ok(Self {
            guild_id,
            member,
            user,
            id: remove_from_map(&mut map, "id")?,
            application_id: remove_from_map(&mut map, "application_id")?,
            data: remove_from_map(&mut map, "data")?,
            channel_id: remove_from_map(&mut map, "channel_id")?,
            token: remove_from_map(&mut map, "token")?,
            version: remove_from_map(&mut map, "version")?,
            guild_locale: remove_from_map_opt(&mut map, "guild_locale")?,
            locale: remove_from_map(&mut map, "locale")?,
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
                            value = Some(map.next_value::<Value>()?);
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

                let value = || value.ok_or_else(|| DeError::missing_field("value"));
                let options = || options.ok_or_else(|| DeError::missing_field("options"));

                let value = match kind {
                    CommandOptionType::User => {
                        CommandDataOptionValue::User(deserialize_val(value()?)?)
                    },
                    CommandOptionType::Role => {
                        CommandDataOptionValue::Role(deserialize_val(value()?)?)
                    },
                    CommandOptionType::Number => {
                        CommandDataOptionValue::Number(deserialize_val(value()?)?)
                    },
                    CommandOptionType::String => {
                        CommandDataOptionValue::String(deserialize_val(value()?)?)
                    },
                    CommandOptionType::Boolean => {
                        CommandDataOptionValue::Boolean(deserialize_val(value()?)?)
                    },
                    CommandOptionType::Integer => {
                        CommandDataOptionValue::Integer(deserialize_val(value()?)?)
                    },
                    CommandOptionType::Channel => {
                        CommandDataOptionValue::Channel(deserialize_val(value()?)?)
                    },
                    CommandOptionType::Attachment => {
                        CommandDataOptionValue::Attachment(deserialize_val(value()?)?)
                    },
                    CommandOptionType::Mentionable => {
                        CommandDataOptionValue::Mentionable(deserialize_val(value()?)?)
                    },
                    CommandOptionType::SubCommandGroup => {
                        CommandDataOptionValue::SubCommandGroup(options()?)
                    },
                    CommandOptionType::SubCommand => CommandDataOptionValue::SubCommand(options()?),
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
            CommandDataOptionValue::SubCommand(o) | CommandDataOptionValue::SubCommandGroup(o)
                if !o.is_empty() =>
            {
                s.serialize_field("options", o)?;
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
