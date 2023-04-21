use std::collections::HashMap;

use serde::de::{Deserializer, Error as DeError};
use serde::ser::{Error as _, Serializer};
use serde::{Deserialize, Serialize};

#[cfg(feature = "model")]
use crate::builder::{
    Builder,
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
    EditInteractionResponse,
};
#[cfg(feature = "collector")]
use crate::builder::{CreateQuickModal, QuickModalResponse};
#[cfg(feature = "collector")]
use crate::client::Context;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::internal::prelude::*;
use crate::model::application::{CommandOptionType, CommandType};
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
use crate::model::Permissions;

/// An interaction when a user invokes a slash command.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(remote = "Self")]
#[non_exhaustive]
pub struct CommandInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The data of the interaction which was triggered.
    pub data: CommandData,
    /// The guild Id this interaction was sent from, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// Channel that the interaction was sent from.
    pub channel: Option<PartialChannel>,
    /// The channel Id this interaction was sent from.
    pub channel_id: ChannelId,
    /// The `member` data for the invoking user.
    ///
    /// **Note**: It is only present if the interaction is triggered in a guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Box<Member>>,
    /// The `user` object for the invoking user.
    #[serde(default)]
    pub user: User,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Always `1`.
    pub version: u8,
    /// Permissions the app or bot has within the channel the interaction was sent from.
    pub app_permissions: Option<Permissions>,
    /// The selected language of the invoking user.
    pub locale: String,
    /// The guild's preferred locale.
    pub guild_locale: Option<String>,
}

#[cfg(feature = "model")]
impl CommandInteraction {
    /// Gets the interaction response.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if there is no interaction response.
    pub async fn get_response(&self, http: impl AsRef<Http>) -> Result<Message> {
        http.as_ref().get_original_interaction_response(&self.token).await
    }

    /// Creates a response to the interaction received.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long. May also return an
    /// [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is an error in
    /// deserializing the API response.
    pub async fn create_response(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInteractionResponse,
    ) -> Result<()> {
        builder.execute(cache_http, (self.id, &self.token)).await
    }

    /// Edits the initial interaction response. Does not work for ephemeral messages.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long. May also return an
    /// [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is an error in
    /// deserializing the API response.
    pub async fn edit_response(
        &self,
        cache_http: impl CacheHttp,
        builder: EditInteractionResponse,
    ) -> Result<Message> {
        builder.execute(cache_http, &self.token).await
    }

    /// Deletes the initial interaction response.
    ///
    /// Does not work on ephemeral messages.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error. Such as if the response was already
    /// deleted.
    pub async fn delete_response(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_original_interaction_response(&self.token).await
    }

    /// Creates a followup response to the response sent.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the content is too long. May also return [`Error::Http`] if the
    /// API returns an error, or [`Error::Json`] if there is an error in deserializing the
    /// response.
    pub async fn create_followup(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInteractionResponseFollowup,
    ) -> Result<Message> {
        builder.execute(cache_http, (None, &self.token)).await
    }

    /// Edits a followup response to the response sent.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the content is too long. May also return [`Error::Http`] if the
    /// API returns an error, or [`Error::Json`] if there is an error in deserializing the
    /// response.
    pub async fn edit_followup(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
        builder: CreateInteractionResponseFollowup,
    ) -> Result<Message> {
        builder.execute(cache_http, (Some(message_id.into()), &self.token)).await
    }

    /// Deletes a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error. Such as if the response was already
    /// deleted.
    pub async fn delete_followup<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<()> {
        http.as_ref().delete_followup_message(&self.token, message_id.into()).await
    }

    /// Gets a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error. Such as if the response was
    /// deleted.
    pub async fn get_followup<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<Message> {
        http.as_ref().get_followup_message(&self.token, message_id.into()).await
    }

    /// Helper function to defer an interaction.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is
    /// an error in deserializing the API response.
    pub async fn defer(&self, cache_http: impl CacheHttp) -> Result<()> {
        let builder = CreateInteractionResponse::Defer(CreateInteractionResponseMessage::default());
        self.create_response(cache_http, builder).await
    }

    /// Helper function to defer an interaction ephemerally
    ///
    /// # Errors
    ///
    /// May also return an [`Error::Http`] if the API returns an error, or an [`Error::Json`] if
    /// there is an error in deserializing the API response.
    pub async fn defer_ephemeral(&self, cache_http: impl CacheHttp) -> Result<()> {
        let builder = CreateInteractionResponse::Defer(
            CreateInteractionResponseMessage::new().ephemeral(true),
        );
        self.create_response(cache_http, builder).await
    }

    /// See [`CreateQuickModal`].
    ///
    /// # Errors
    ///
    /// See [`CreateQuickModal::execute()`].
    #[cfg(feature = "collector")]
    pub async fn quick_modal(
        &self,
        ctx: &Context,
        builder: CreateQuickModal,
    ) -> Result<Option<QuickModalResponse>> {
        builder.execute(ctx, (self.id, &self.token, &ctx.shard)).await
    }
}

// Manual impl needed to insert guild_id into resolved Role's
impl<'de> Deserialize<'de> for CommandInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        // calls #[serde(remote)]-generated inherent method
        let mut interaction = Self::deserialize(deserializer)?;
        if let Some(guild_id) = interaction.guild_id {
            if let Some(member) = &mut interaction.member {
                member.guild_id = guild_id;
                // If `member` is present, `user` wasn't sent and is still filled with default data
                interaction.user = member.user.clone();
            }
            interaction.data.resolved.roles.values_mut().for_each(|r| r.guild_id = guild_id);
        }
        Ok(interaction)
    }
}

impl Serialize for CommandInteraction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        // calls #[serde(remote)]-generated inherent method
        Self::serialize(self, serializer)
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
    /// The parameters and the given values. The converted objects from the given options.
    #[serde(default)]
    pub resolved: CommandDataResolved,
    #[serde(default)]
    pub options: Vec<CommandDataOption>,
    /// The Id of the guild the command is registered to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// The targeted user or message, if the triggered application command type is [`User`] or
    /// [`Message`].
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

    /// Returns the resolved options from `CommandData::options` and [`CommandData::resolved`].
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

        resolve_options(&self.options, &self.resolved)
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

/// The resolved data of a command data interaction payload. It contains the objects of
/// [`CommandDataOption`]s.
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
/// All options have names and an option can either be a parameter and input `value` or it can
/// denote a sub-command or group, in which case it will contain a top-level key and another vector
/// of `options`.
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

#[derive(Deserialize, Serialize)]
struct RawCommandDataOption {
    name: String,
    #[serde(rename = "type")]
    kind: CommandOptionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Vec<RawCommandDataOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    focused: Option<bool>,
}

fn option_from_raw<E: DeError>(raw: RawCommandDataOption) -> StdResult<CommandDataOption, E> {
    macro_rules! value {
        () => {
            serde_json::from_value(raw.value.ok_or_else(|| DeError::missing_field("value"))?)
                .map_err(DeError::custom)?
        };
    }

    let value = match raw.kind {
        _ if raw.focused == Some(true) => CommandDataOptionValue::Autocomplete {
            kind: raw.kind,
            value: value!(),
        },
        CommandOptionType::Boolean => CommandDataOptionValue::Boolean(value!()),
        CommandOptionType::Integer => CommandDataOptionValue::Integer(value!()),
        CommandOptionType::Number => CommandDataOptionValue::Number(value!()),
        CommandOptionType::String => CommandDataOptionValue::String(value!()),
        CommandOptionType::SubCommand => {
            let options = raw.options.ok_or_else(|| DeError::missing_field("options"))?;
            let options = options.into_iter().map(option_from_raw).collect::<StdResult<_, E>>()?;
            CommandDataOptionValue::SubCommand(options)
        },
        CommandOptionType::SubCommandGroup => {
            let options = raw.options.ok_or_else(|| DeError::missing_field("options"))?;
            let options = options.into_iter().map(option_from_raw).collect::<StdResult<_, E>>()?;
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
        name: raw.name,
        value,
    })
}

fn option_to_raw(option: &CommandDataOption) -> StdResult<RawCommandDataOption, serde_json::Error> {
    let mut raw = RawCommandDataOption {
        name: option.name.clone(),
        kind: option.kind(),
        value: None,
        options: None,
        focused: None,
    };

    match &option.value {
        CommandDataOptionValue::Autocomplete {
            kind: _,
            value,
        } => {
            raw.value = Some(serde_json::to_value(value)?);
            raw.focused = Some(true);
        },
        CommandDataOptionValue::Boolean(v) => raw.value = Some(serde_json::to_value(v)?),
        CommandDataOptionValue::Integer(v) => raw.value = Some(serde_json::to_value(v)?),
        CommandDataOptionValue::Number(v) => raw.value = Some(serde_json::to_value(v)?),
        CommandDataOptionValue::String(v) => raw.value = Some(serde_json::to_value(v)?),
        CommandDataOptionValue::SubCommand(o) | CommandDataOptionValue::SubCommandGroup(o) => {
            raw.options =
                Some(o.iter().map(option_to_raw).collect::<StdResult<_, serde_json::Error>>()?);
        },
        CommandDataOptionValue::Attachment(v) => raw.value = Some(serde_json::to_value(v)?),
        CommandDataOptionValue::Channel(v) => raw.value = Some(serde_json::to_value(v)?),
        CommandDataOptionValue::Mentionable(v) => raw.value = Some(serde_json::to_value(v)?),
        CommandDataOptionValue::Role(v) => raw.value = Some(serde_json::to_value(v)?),
        CommandDataOptionValue::User(v) => raw.value = Some(serde_json::to_value(v)?),
        CommandDataOptionValue::Unknown(_) => {},
    }

    Ok(raw)
}

// Manual impl needed to emulate integer enum tags
impl<'de> Deserialize<'de> for CommandDataOption {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        option_from_raw(RawCommandDataOption::deserialize(deserializer)?)
    }
}

impl Serialize for CommandDataOption {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        option_to_raw(self).map_err(S::Error::custom)?.serialize(serializer)
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
    use super::*;
    use crate::json::{assert_json, json};

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

        assert_json(
            &value,
            json!({
                "name": "subcommand_group",
                "type": 2,
                "options": [{
                    "name": "subcommand",
                    "type": 1,
                    "options": [{"name": "channel", "type": 7, "value": "3"}],
                }]
            }),
        );
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

        assert_json(
            &value,
            json!([
                {"name": "boolean", "type": 5, "value": true},
                {"name": "integer", "type": 4, "value": 1},
                {"name": "number", "type": 10, "value": 2.0},
                {"name": "string", "type": 3, "value": "foobar"},
                {"name": "empty_subcommand", "type": 1, "options": []},
                {"name": "autocomplete", "type": 4, "value": "not an integer", "focused": true},
            ]),
        );
    }
}
