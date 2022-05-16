use std::collections::HashMap;
use std::fmt;

use serde::de::{Deserializer, Error as DeError, IgnoredAny, MapAccess};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_value::{DeserializerError, Value};

#[cfg(feature = "http")]
use crate::builder::{
    CreateAutocompleteResponse,
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    EditInteractionResponse,
};
#[cfg(feature = "http")]
use crate::http::Http;
use crate::internal::prelude::*;
use crate::model::application::command::{CommandOptionType, CommandType};
use crate::model::application::interaction::add_guild_id_to_resolved;
#[cfg(feature = "http")]
use crate::model::application::interaction::InteractionResponseType;
use crate::model::channel::{Attachment, Message, PartialChannel};
use crate::model::guild::{Member, PartialMember, Role};
use crate::model::id::{
    ApplicationId,
    AttachmentId,
    ChannelId,
    CommandId,
    GenericId,
    GuildId,
    InteractionId,
    MessageId,
    RoleId,
    TargetId,
    UserId,
};
use crate::model::user::User;
use crate::model::utils::{remove_from_map, remove_from_map_opt};

/// An interaction when a user invokes a slash command.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object).
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The data of the interaction which was triggered.
    pub data: CommandData,
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
impl ApplicationCommandInteraction {
    /// Gets the interaction response.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if there is no interaction response.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    pub async fn get_interaction_response(&self, http: impl AsRef<Http>) -> Result<Message> {
        http.as_ref().get_original_interaction_response(&self.token).await
    }

    /// Creates a response to the interaction received.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long.
    /// May also return an [`Error::Http`] if the API returns an error,
    /// or an [`Error::Json`] if there is an error in deserializing the
    /// API response.
    ///
    /// [`Error::Model`]: crate::error::Error::Model
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn create_interaction_response<'a, F>(
        &self,
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<()>
    where
        for<'b> F:
            FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a>,
    {
        let mut interaction_response = CreateInteractionResponse::default();
        f(&mut interaction_response);
        self._create_interaction_response(http.as_ref(), interaction_response).await
    }

    async fn _create_interaction_response<'a>(
        &self,
        http: &Http,
        mut interaction_response: CreateInteractionResponse<'a>,
    ) -> Result<()> {
        let files = interaction_response
            .data
            .as_mut()
            .map_or_else(Vec::new, |d| std::mem::take(&mut d.files));

        if files.is_empty() {
            http.create_interaction_response(self.id.get(), &self.token, &interaction_response)
                .await
        } else {
            http.create_interaction_response_with_files(
                self.id.get(),
                &self.token,
                &interaction_response,
                files,
            )
            .await
        }
    }

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
        #[derive(Serialize)]
        struct AutocompleteResponse {
            data: CreateAutocompleteResponse,
            #[serde(rename = "type")]
            kind: InteractionResponseType,
        }

        let mut response = CreateAutocompleteResponse::default();
        f(&mut response);

        let map = AutocompleteResponse {
            data: response,
            kind: InteractionResponseType::Autocomplete,
        };

        http.as_ref().create_interaction_response(self.id.get(), &self.token, &map).await
    }

    /// Edits the initial interaction response.
    ///
    /// `application_id` will usually be the bot's [`UserId`], except in cases of bots being very old.
    ///
    /// Refer to Discord's docs for Edit Webhook Message for field information.
    ///
    /// **Note**:   Message contents must be under 2000 unicode code points, does not work on ephemeral messages.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the edited content is too long.
    /// May also return [`Error::Http`] if the API returns an error,
    /// or an [`Error::Json`] if there is an error deserializing the response.
    ///
    /// [`Error::Model`]: crate::error::Error::Model
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn edit_original_interaction_response<F>(
        &self,
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<Message>
    where
        F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse,
    {
        let mut interaction_response = EditInteractionResponse::default();
        f(&mut interaction_response);

        http.as_ref().edit_original_interaction_response(&self.token, &interaction_response).await
    }

    /// Deletes the initial interaction response.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was already deleted.
    pub async fn delete_original_interaction_response(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_original_interaction_response(&self.token).await
    }

    /// Creates a followup response to the response sent.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Will return [`Error::Model`] if the content is too long.
    /// May also return [`Error::Http`] if the API returns an error,
    /// or a [`Error::Json`] if there is an error in deserializing the response.
    ///
    /// [`Error::Model`]: crate::error::Error::Model
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn create_followup_message<'a, F>(
        &self,
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<Message>
    where
        for<'b> F: FnOnce(
            &'b mut CreateInteractionResponseFollowup<'a>,
        ) -> &'b mut CreateInteractionResponseFollowup<'a>,
    {
        let mut interaction_response = CreateInteractionResponseFollowup::default();
        f(&mut interaction_response);
        self._create_followup_message(http.as_ref(), interaction_response).await
    }

    async fn _create_followup_message<'a>(
        &self,
        http: &Http,
        mut interaction_response: CreateInteractionResponseFollowup<'a>,
    ) -> Result<Message> {
        let files = std::mem::take(&mut interaction_response.files);

        if files.is_empty() {
            http.create_followup_message(&self.token, &interaction_response).await
        } else {
            http.create_followup_message_with_files(&self.token, &interaction_response, files).await
        }
    }

    /// Edits a followup response to the response sent.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Will return [`Error::Model`] if the content is too long.
    /// May also return [`Error::Http`] if the API returns an error,
    /// or a [`Error::Json`] if there is an error in deserializing the response.
    ///
    /// [`Error::Model`]: crate::error::Error::Model
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn edit_followup_message<'a, F, M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
        f: F,
    ) -> Result<Message>
    where
        for<'b> F: FnOnce(
            &'b mut CreateInteractionResponseFollowup<'a>,
        ) -> &'b mut CreateInteractionResponseFollowup<'a>,
    {
        let mut builder = CreateInteractionResponseFollowup::default();
        f(&mut builder);

        let http = http.as_ref();
        let message_id = message_id.into().into();
        let files = std::mem::take(&mut builder.files);

        if files.is_empty() {
            http.edit_followup_message(&self.token, message_id, &builder).await
        } else {
            http.edit_followup_message_and_attachments(&self.token, message_id, &builder, files)
                .await
        }
    }

    /// Deletes a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was already deleted.
    pub async fn delete_followup_message<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<()> {
        http.as_ref().delete_followup_message(&self.token, message_id.into().into()).await
    }

    /// Gets a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was deleted.
    pub async fn get_followup_message<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<Message> {
        http.as_ref().get_followup_message(&self.token, message_id.into().into()).await
    }

    /// Helper function to defer an interaction
    ///
    /// # Errors
    ///
    /// May also return an [`Error::Http`] if the API returns an error,
    /// or an [`Error::Json`] if there is an error in deserializing the
    /// API response.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn defer(&self, http: impl AsRef<Http>) -> Result<()> {
        self.create_interaction_response(http, |f| {
            f.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    }
}

impl<'de> Deserialize<'de> for ApplicationCommandInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = remove_from_map_opt::<GuildId, _>(&mut map, "guild_id")?;

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
            locale: remove_from_map(&mut map, "locale")?,
            guild_locale: remove_from_map_opt(&mut map, "guild_locale")?,
        })
    }
}

/// The command data payload.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandData {
    /// The Id of the invoked command.
    pub id: CommandId,
    /// The name of the invoked command.
    pub name: String,
    /// The application command type of the triggered application command.
    #[serde(rename = "type")]
    pub kind: CommandType,
    /// The parameters and the given values.
    /// The converted objects from the given options.
    #[serde(default)]
    pub resolved: CommandDataResolved,
    #[serde(default)]
    pub options: Vec<CommandDataOption>,
    /// The Id of the guild the command is registered to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// The targeted user or message, if the triggered application command type
    /// is [`User`] or [`Message`].
    ///
    /// Its object data can be found in the [`resolved`] field.
    ///
    /// [`resolved`]: Self::resolved
    /// [`User`]: CommandType::User
    /// [`Message`]: CommandType::Message
    pub target_id: Option<TargetId>,
}

impl CommandData {
    /// Returns the autocomplete option from `CommandData::options`.
    #[must_use]
    pub fn autocomplete(&self) -> Option<AutocompleteOption<'_>> {
        fn find_option(opts: &[CommandDataOption]) -> Option<AutocompleteOption<'_>> {
            for opt in opts {
                match &opt.value {
                    CommandDataOptionValue::SubCommand(opts)
                    | CommandDataOptionValue::SubCommandGroup(opts) => {
                        return find_option(opts);
                    },
                    CommandDataOptionValue::Autocomplete {
                        kind,
                        value,
                    } => {
                        return Some(AutocompleteOption {
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
        find_option(&self.options)
    }

    /// Returns the resolved options from `CommandData::options` and
    /// [`CommandData::resolved`].
    #[must_use]
    pub fn options(&self) -> Vec<ResolvedOption<'_>> {
        fn resolve_options<'a>(
            opts: &'a [CommandDataOption],
            resolved: &'a CommandDataResolved,
        ) -> Vec<ResolvedOption<'a>> {
            let mut options = Vec::new();
            for opt in opts {
                let value = match &opt.value {
                    CommandDataOptionValue::SubCommand(opts) => {
                        ResolvedValue::SubCommand(resolve_options(opts, resolved))
                    },
                    CommandDataOptionValue::SubCommandGroup(opts) => {
                        ResolvedValue::SubCommandGroup(resolve_options(opts, resolved))
                    },
                    CommandDataOptionValue::Autocomplete {
                        kind,
                        value,
                    } => ResolvedValue::Autocomplete {
                        kind: *kind,
                        value,
                    },
                    CommandDataOptionValue::Boolean(v) => ResolvedValue::Boolean(*v),
                    CommandDataOptionValue::Integer(v) => ResolvedValue::Integer(*v),
                    CommandDataOptionValue::Number(v) => ResolvedValue::Number(*v),
                    CommandDataOptionValue::String(v) => ResolvedValue::String(v),
                    CommandDataOptionValue::Attachment(id) => resolved.attachments.get(id).map_or(
                        ResolvedValue::Unresolved(Unresolved::Attachment(*id)),
                        ResolvedValue::Attachment,
                    ),
                    CommandDataOptionValue::Channel(id) => resolved.channels.get(id).map_or(
                        ResolvedValue::Unresolved(Unresolved::Channel(*id)),
                        ResolvedValue::Channel,
                    ),
                    CommandDataOptionValue::Mentionable(id) => {
                        let value = if let Some(user) = resolved.users.get(&UserId(id.0)) {
                            Some(ResolvedValue::User(user, resolved.members.get(&UserId(id.0))))
                        } else {
                            resolved.roles.get(&RoleId(id.0)).map(ResolvedValue::Role)
                        };
                        value.unwrap_or(ResolvedValue::Unresolved(Unresolved::Mentionable(*id)))
                    },
                    CommandDataOptionValue::User(id) => resolved
                        .users
                        .get(id)
                        .map(|u| ResolvedValue::User(u, resolved.members.get(id)))
                        .unwrap_or(ResolvedValue::Unresolved(Unresolved::User(*id))),
                    CommandDataOptionValue::Role(id) => resolved.roles.get(id).map_or(
                        ResolvedValue::Unresolved(Unresolved::RoleId(*id)),
                        ResolvedValue::Role,
                    ),
                    CommandDataOptionValue::Unknown(unknown) => {
                        ResolvedValue::Unresolved(Unresolved::Unknown(*unknown))
                    },
                };

                options.push(ResolvedOption {
                    name: &opt.name,
                    value,
                });
            }
            options
        }

        resolve_options(&*self.options, &self.resolved)
    }

    /// The target resolved data of [`target_id`]
    ///
    /// [`target_id`]: Self::target_id
    #[must_use]
    pub fn target(&self) -> Option<ResolvedTarget<'_>> {
        match (self.kind, self.target_id) {
            (CommandType::User, Some(id)) => {
                let user_id = id.to_user_id();

                let user = self.resolved.users.get(&user_id)?;
                let member = self.resolved.members.get(&user_id);

                Some(ResolvedTarget::User(user, member))
            },
            (CommandType::Message, Some(id)) => {
                let message_id = id.to_message_id();
                let message = self.resolved.messages.get(&message_id)?;

                Some(ResolvedTarget::Message(message))
            },
            _ => None,
        }
    }
}

/// The focused option for autocomplete interactions return by [`CommandData::autocomplete`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AutocompleteOption<'a> {
    pub name: &'a str,
    pub kind: CommandOptionType,
    pub value: &'a str,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ResolvedOption<'a> {
    pub name: &'a str,
    pub value: ResolvedValue<'a>,
}

/// The resolved value of a [`CommandDataOption`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ResolvedValue<'a> {
    Autocomplete { kind: CommandOptionType, value: &'a str },
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(&'a str),
    SubCommand(Vec<ResolvedOption<'a>>),
    SubCommandGroup(Vec<ResolvedOption<'a>>),
    Attachment(&'a Attachment),
    Channel(&'a PartialChannel),
    Role(&'a Role),
    User(&'a User, Option<&'a PartialMember>),
    Unresolved(Unresolved),
}

/// Option value variants that couldn't be resolved by `CommandData::options()`.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Unresolved {
    Attachment(AttachmentId),
    Channel(ChannelId),
    Mentionable(GenericId),
    RoleId(RoleId),
    User(UserId),
    /// Variant value for unknown option types.
    Unknown(u8),
}

/// The resolved value of a [`CommandData::target_id`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ResolvedTarget<'a> {
    User(&'a User, Option<&'a PartialMember>),
    Message(&'a Message),
}

/// The resolved data of a command data interaction payload.
/// It contains the objects of [`CommandDataOption`]s.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-resolved-data-structure).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandDataResolved {
    /// The resolved users.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub users: HashMap<UserId, User>,
    /// The resolved partial members.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub members: HashMap<UserId, PartialMember>,
    /// The resolved roles.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub roles: HashMap<RoleId, Role>,
    /// The resolved partial channels.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub channels: HashMap<ChannelId, PartialChannel>,
    /// The resolved messages.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub messages: HashMap<MessageId, Message>,
    /// The resolved attachments.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attachments: HashMap<AttachmentId, Attachment>,
}

/// A set of a parameter and a value from the user.
///
/// All options have names and an option can either be a parameter and input `value` or it can denote a sub-command or group, in which case it will contain a
/// top-level key and another vector of `options`.
///
/// Their resolved objects can be found on [`CommandData::resolved`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-interaction-data-option-structure).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CommandDataOption {
    /// The name of the parameter.
    pub name: String,
    /// The given value.
    pub value: CommandDataOptionValue,
}

impl CommandDataOption {
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
                let mut value: Option<Value> = None;
                let mut options = None;
                let mut focused = None;

                macro_rules! next_value {
                    ($field:ident, $name:literal) => {
                        if $field.is_some() {
                            return Err(DeError::duplicate_field($name));
                        }
                        $field = Some(map.next_value()?);
                    };
                }

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            next_value!(name, "name");
                        },
                        Field::Type => {
                            next_value!(kind, "type");
                        },
                        Field::Value => {
                            next_value!(value, "value");
                        },
                        Field::Options => {
                            next_value!(options, "options");
                        },
                        Field::Focused => {
                            next_value!(focused, "focused");
                        },
                        Field::Unknown(_) => {
                            map.next_value::<IgnoredAny>()?;
                        },
                    }
                }

                let name = name.ok_or_else(|| DeError::missing_field("name"))?;
                let kind = kind.ok_or_else(|| DeError::missing_field("type"))?;
                let focused = focused.unwrap_or_default();

                macro_rules! value {
                    () => {
                        value
                            .ok_or_else(|| DeError::missing_field("value"))?
                            .deserialize_into()
                            .map_err(DeserializerError::into_error)?
                    };
                }

                if focused {
                    return Ok(CommandDataOption {
                        name,
                        value: CommandDataOptionValue::Autocomplete {
                            kind,
                            value: value!(),
                        },
                    });
                }

                let value = match kind {
                    CommandOptionType::Boolean => CommandDataOptionValue::Boolean(value!()),
                    CommandOptionType::Integer => CommandDataOptionValue::Integer(value!()),
                    CommandOptionType::Number => CommandDataOptionValue::Number(value!()),
                    CommandOptionType::String => CommandDataOptionValue::String(value!()),
                    CommandOptionType::SubCommand => {
                        let options = options.ok_or_else(|| DeError::missing_field("options"))?;
                        CommandDataOptionValue::SubCommand(options)
                    },
                    CommandOptionType::SubCommandGroup => {
                        let options = options.ok_or_else(|| DeError::missing_field("options"))?;
                        CommandDataOptionValue::SubCommandGroup(options)
                    },
                    CommandOptionType::Attachment => CommandDataOptionValue::Attachment(value!()),
                    CommandOptionType::Channel => CommandDataOptionValue::Channel(value!()),
                    CommandOptionType::Mentionable => CommandDataOptionValue::Mentionable(value!()),
                    CommandOptionType::Role => CommandDataOptionValue::Role(value!()),
                    CommandOptionType::User => CommandDataOptionValue::User(value!()),
                    CommandOptionType::Unknown(unknown) => CommandDataOptionValue::Unknown(unknown),
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
            CommandDataOptionValue::Unknown(_) => (false, false),
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
                s.serialize_field("focused", &true)?;
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
                s.serialize_field("options", o)?;
            },
            _ => {},
        }

        s.end()
    }
}

/// The value of an [`CommandDataOption`].
#[derive(Clone, Debug, PartialEq)]
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
    Unknown(u8),
}

impl CommandDataOptionValue {
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
            Self::Unknown(unknown) => CommandOptionType::Unknown(*unknown),
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

impl TargetId {
    /// Converts this [`TargetId`] to [`UserId`].
    #[must_use]
    pub fn to_user_id(self) -> UserId {
        self.0.into()
    }

    /// Converts this [`TargetId`] to [`MessageId`].
    #[must_use]
    pub fn to_message_id(self) -> MessageId {
        self.0.into()
    }
}

impl From<MessageId> for TargetId {
    fn from(id: MessageId) -> Self {
        Self(id.0)
    }
}

impl<'a> From<&'a MessageId> for TargetId {
    fn from(id: &MessageId) -> Self {
        Self(id.0)
    }
}

impl From<UserId> for TargetId {
    fn from(id: UserId) -> Self {
        Self(id.0)
    }
}

impl<'a> From<&'a UserId> for TargetId {
    fn from(id: &UserId) -> Self {
        Self(id.0)
    }
}

impl From<TargetId> for MessageId {
    fn from(id: TargetId) -> Self {
        Self(id.0)
    }
}

impl From<TargetId> for UserId {
    fn from(id: TargetId) -> Self {
        Self(id.0)
    }
}

#[cfg(test)]
mod tests {
    use serde_test::Token;

    use super::*;

    #[test]
    fn nested_options() {
        let value = CommandDataOption {
            name: "subcommand_group".into(),
            value: CommandDataOptionValue::SubCommandGroup(vec![CommandDataOption {
                name: "subcommand".into(),
                value: CommandDataOptionValue::SubCommand(vec![CommandDataOption {
                    name: "channel".into(),
                    value: CommandDataOptionValue::Channel(ChannelId::new(3)),
                }]),
            }]),
        };

        serde_test::assert_tokens(&value, &[
            Token::Struct {
                name: "CommandDataOption",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("subcommand_group"),
            Token::Str("type"),
            Token::U8(CommandOptionType::SubCommandGroup.into()),
            Token::Str("options"),
            Token::Seq {
                len: Some(1),
            },
            Token::Struct {
                name: "CommandDataOption",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("subcommand"),
            Token::Str("type"),
            Token::U8(CommandOptionType::SubCommand.into()),
            Token::Str("options"),
            Token::Seq {
                len: Some(1),
            },
            Token::Struct {
                name: "CommandDataOption",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("channel"),
            Token::Str("type"),
            Token::U8(CommandOptionType::Channel.into()),
            Token::Str("value"),
            Token::NewtypeStruct {
                name: "ChannelId",
            },
            Token::Str("3"),
            Token::StructEnd,
            Token::SeqEnd,
            Token::StructEnd,
            Token::SeqEnd,
            Token::StructEnd,
        ]);
    }

    #[test]
    fn mixed_options() {
        let value = vec![
            CommandDataOption {
                name: "boolean".into(),
                value: CommandDataOptionValue::Boolean(true),
            },
            CommandDataOption {
                name: "integer".into(),
                value: CommandDataOptionValue::Integer(1),
            },
            CommandDataOption {
                name: "number".into(),
                value: CommandDataOptionValue::Number(2.0),
            },
            CommandDataOption {
                name: "string".into(),
                value: CommandDataOptionValue::String("foobar".into()),
            },
            CommandDataOption {
                name: "empty_subcommand".into(),
                value: CommandDataOptionValue::SubCommand(vec![]),
            },
            CommandDataOption {
                name: "autocomplete".into(),
                value: CommandDataOptionValue::Autocomplete {
                    kind: CommandOptionType::Integer,
                    value: "not an integer".into(),
                },
            },
        ];

        serde_test::assert_tokens(&value, &[
            Token::Seq {
                len: Some(value.len()),
            },
            Token::Struct {
                name: "CommandDataOption",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("boolean"),
            Token::Str("type"),
            Token::U8(CommandOptionType::Boolean.into()),
            Token::Str("value"),
            Token::Bool(true),
            Token::StructEnd,
            Token::Struct {
                name: "CommandDataOption",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("integer"),
            Token::Str("type"),
            Token::U8(CommandOptionType::Integer.into()),
            Token::Str("value"),
            Token::I64(1),
            Token::StructEnd,
            Token::Struct {
                name: "CommandDataOption",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("number"),
            Token::Str("type"),
            Token::U8(CommandOptionType::Number.into()),
            Token::Str("value"),
            Token::F64(2.0),
            Token::StructEnd,
            Token::Struct {
                name: "CommandDataOption",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("string"),
            Token::Str("type"),
            Token::U8(CommandOptionType::String.into()),
            Token::Str("value"),
            Token::Str("foobar"),
            Token::StructEnd,
            Token::Struct {
                name: "CommandDataOption",
                len: 2,
            },
            Token::Str("name"),
            Token::Str("empty_subcommand"),
            Token::Str("type"),
            Token::U8(CommandOptionType::SubCommand.into()),
            Token::Str("options"),
            Token::Seq {
                len: Some(0),
            },
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "CommandDataOption",
                len: 4,
            },
            Token::Str("name"),
            Token::Str("autocomplete"),
            Token::Str("type"),
            Token::U8(CommandOptionType::Integer.into()),
            Token::Str("value"),
            Token::Str("not an integer"),
            Token::Str("focused"),
            Token::Bool(true),
            Token::StructEnd,
            Token::SeqEnd,
        ]);
    }
}
