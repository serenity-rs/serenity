//! Interactions information-related models.

use bitflags::__impl_bitflags;
use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde_json::{Map, Number, Value};

use super::prelude::*;
use crate::builder::{
    CreateInteraction,
    CreateInteractionPermissions,
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    CreateInteractions,
    EditInteractionResponse,
};
use crate::http::Http;
use crate::internal::prelude::*;
use crate::utils;

/// Information about an interaction.
///
/// An interaction is sent when a user invokes a slash command and is the same
/// for slash commands and other future interaction types.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct Interaction {
    pub id: InteractionId,
    #[serde(rename = "type")]
    pub kind: InteractionType,
    pub data: Option<ApplicationCommandInteractionData>,
    pub guild_id: Option<GuildId>,
    pub channel_id: Option<ChannelId>,
    pub member: Option<Member>,
    pub user: Option<User>,
    pub token: String,
    pub version: u8,
}

impl<'de> Deserialize<'de> for Interaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("guild_id").and_then(|x| x.as_str()).and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(member) = map.get_mut("member").and_then(|x| x.as_object_mut()) {
                member.insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
            }

            if let Some(data) = map.get_mut("data") {
                if let Some(resolved) = data.get_mut("resolved") {
                    if let Some(roles) = resolved.get_mut("roles") {
                        if let Some(values) = roles.as_object_mut() {
                            for value in values.values_mut() {
                                value.as_object_mut().unwrap().insert(
                                    "guild_id".to_string(),
                                    Value::String(guild_id.to_string()),
                                );
                            }
                        }
                    }

                    if let Some(channels) = resolved.get_mut("channels") {
                        if let Some(values) = channels.as_object_mut() {
                            for value in values.values_mut() {
                                value.as_object_mut().unwrap().insert(
                                    "guild_id".to_string(),
                                    Value::String(guild_id.to_string()),
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

        let kind = map
            .remove("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(InteractionType::deserialize)
            .map_err(DeError::custom)?;

        let data = match map.remove("data") {
            Some(v) => serde_json::from_value::<Option<ApplicationCommandInteractionData>>(v)
                .map_err(DeError::custom)?,
            None => None,
        };

        let guild_id = match map.contains_key("guild_id") {
            true => Some(
                map.remove("guild_id")
                    .ok_or_else(|| DeError::custom("expected guild_id"))
                    .and_then(GuildId::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let channel_id = match map.contains_key("channel_id") {
            true => Some(
                map.remove("channel_id")
                    .ok_or_else(|| DeError::custom("expected channel_id"))
                    .and_then(ChannelId::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let member = match map.contains_key("member") {
            true => Some(
                map.remove("member")
                    .ok_or_else(|| DeError::custom("expected member"))
                    .and_then(Member::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let user = match map.contains_key("user") {
            true => Some(
                map.remove("user")
                    .ok_or_else(|| DeError::custom("expected user"))
                    .and_then(User::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

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

        Ok(Self {
            id,
            kind,
            data,
            guild_id,
            channel_id,
            member,
            user,
            token,
            version,
        })
    }
}

/// The type of an Interaction
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
}

enum_number!(InteractionType {
    Ping,
    ApplicationCommand
});

/// The command data payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandInteractionData {
    pub id: CommandId,
    pub name: String,
    #[serde(default)]
    pub options: Vec<ApplicationCommandInteractionDataOption>,
    pub resolved: Option<ApplicationCommandInteractionDataResolved>,
}

/// The resolved data of a command data interaction payload.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandInteractionDataResolved {
    pub users: Option<HashMap<UserId, User>>,
    pub members: Option<HashMap<UserId, PartialMember>>,
    pub roles: Option<HashMap<RoleId, Role>>,
    pub channels: Option<HashMap<ChannelId, PartialChannel>>,
}

impl<'de> Deserialize<'de> for ApplicationCommandInteractionDataResolved {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        println!("{:?}", map);

        let members = match map.contains_key("members") {
            true => Some(
                map.remove("members")
                    .ok_or_else(|| DeError::custom("expected members"))
                    .and_then(deserialize_partial_members_map)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let users = match map.contains_key("users") {
            true => Some(
                map.remove("users")
                    .ok_or_else(|| DeError::custom("expected users"))
                    .and_then(deserialize_users)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let roles = match map.contains_key("roles") {
            true => Some(
                map.remove("roles")
                    .ok_or_else(|| DeError::custom("expected roles"))
                    .and_then(deserialize_roles_map)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let channels = match map.contains_key("channels") {
            true => Some(
                map.remove("channels")
                    .ok_or_else(|| DeError::custom("expected chanels"))
                    .and_then(deserialize_channels_map)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        Ok(Self {
            users,
            members,
            roles,
            channels,
        })
    }
}

/// A set of a parameter and a value from the user.
///
/// All options have names and an option can either be a parameter and input `value` or it can denote a sub-command or group, in which case it will contain a
/// top-level key and another vector of `options`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandInteractionDataOption {
    pub name: String,
    pub value: Option<Value>,
    // pub kind: ApplicationCommandOptionType,
    #[serde(default)]
    pub options: Vec<ApplicationCommandInteractionDataOption>,
}

fn default_permission_value() -> bool {
    true
}

/// The base command model that belongs to an application.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommand {
    pub id: CommandId,
    pub application_id: ApplicationId,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub options: Vec<ApplicationCommandOption>,
    #[serde(default = "self::default_permission_value")]
    pub default_permission: bool,
}

/// The parameters for a command.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandOption {
    #[serde(rename = "type")]
    pub kind: ApplicationCommandOptionType,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub choices: Vec<ApplicationCommandOptionChoice>,
    #[serde(default)]
    pub options: Vec<ApplicationCommandOption>,
}

/// An application command permission.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandPermission {
    pub id: InteractionId,
    pub application_id: ApplicationId,
    pub guild_id: GuildId,
    pub permissions: Vec<ApplicationCommandPermissionData>,
}

/// The permissions data.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandPermissionData {
    #[serde(rename = "type")]
    pub kind: ApplicationCommandPermissionType,
    pub id: CommandPermissionId,
    pub permission: bool,
}

/// The type of an application command option.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum ApplicationCommandOptionType {
    SubCommand = 1,
    SubCommandGroup = 2,
    String = 3,
    Integer = 4,
    Boolean = 5,
    User = 6,
    Channel = 7,
    Role = 8,
}

enum_number!(ApplicationCommandOptionType {
    SubCommand,
    SubCommandGroup,
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
});

/// The type of an application command option.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum ApplicationCommandPermissionType {
    Role = 1,
    User = 2,
}

enum_number!(ApplicationCommandPermissionType {
    Role,
    User
});

/// The only valid values a user can pick in a command option.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandOptionChoice {
    pub name: String,
    pub value: Value,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum InteractionResponseType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    DeferredChannelMessageWithSource = 5,
}

#[derive(Clone, Serialize)]
#[non_exhaustive]
pub struct InteractionApplicationCommandCallbackDataFlags {
    bits: u64,
}

__impl_bitflags! {
    InteractionApplicationCommandCallbackDataFlags: u64 {
        /// Interaction message will only be visible to sender
        EPHEMERAL = 0b0000_0000_0000_0000_0000_0000_0100_0000;
    }
}

/// Sent when a [`Message`] is a response to an [`Interaction`].
///
/// [`Message`]: crate::model::channel::Message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageInteraction {
    pub id: InteractionId,
    #[serde(rename = "type")]
    pub kind: InteractionType,
    pub name: String,
    pub user: User,
}

impl Interaction {
    /// Creates a global [`ApplicationCommand`],
    /// overriding an existing one with the same name if it exists.
    ///
    /// When a created `ApplicationCommand` is used, the [`InteractionCreate`] event will be emitted.
    ///
    /// **Note**: Global commands may take up to an hour to become available.
    ///
    /// As such, it is recommended that guild application commands be used for testing purposes.
    ///
    /// # Examples
    ///
    /// Create a simple ping command
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http = Arc::new(Http::default());
    /// use serenity::model::{interactions::Interaction, id::ApplicationId};
    ///
    /// let _ = Interaction::create_global_application_command(&http, |a| {
    ///    a.name("ping")
    ///     .description("A simple ping command")
    /// })
    /// .await;
    /// # }
    /// ```
    ///
    /// Create a command that echoes what is inserted
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() {
    /// # let http = Arc::new(Http::default());
    /// use serenity::model::{
    /// interactions::{Interaction, ApplicationCommandOptionType},
    /// id::ApplicationId
    /// };
    ///
    /// let _ = Interaction::create_global_application_command(&http, |a| {
    ///    a.name("echo")
    ///     .description("What is said is echoed")
    ///     .create_interaction_option(|o| {
    ///         o.name("to_say")
    ///          .description("What will be echoed")
    ///          .kind(ApplicationCommandOptionType::String)
    ///          .required(true)
    ///     })
    /// })
    /// .await;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// May return an [`Error::Http`] if the `ApplicationCommand` is illformed,
    /// such as if more than 10 `choices` are set. See the [API Docs] for further details.
    ///
    /// Can also return an [`Error::Json`] if there is an error in deserializing
    /// the response.
    ///
    /// [`ApplicationCommand`]: crate::model::interactions::ApplicationCommand
    /// [`InteractionCreate`]: crate::client::EventHandler::interaction_create
    /// [API Docs]: https://discord.com/developers/docs/interactions/slash-commands
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn create_global_application_command<F>(
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<ApplicationCommand>
    where
        F: FnOnce(&mut CreateInteraction) -> &mut CreateInteraction,
    {
        let map = Interaction::build_interaction(f);
        http.as_ref().create_global_application_command(&Value::Object(map)).await
    }

    /// Same as [`create_global_application_command`] but allows
    /// to create more than one global command per call.
    ///
    /// [`create_global_application_command`]: Self::create_global_application_command
    pub async fn create_global_application_commands<F>(
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<Vec<ApplicationCommand>>
    where
        F: FnOnce(&mut CreateInteractions) -> &mut CreateInteractions,
    {
        let mut array = CreateInteractions::default();

        f(&mut array);

        http.as_ref().create_global_application_commands(&Value::Array(array.0)).await
    }

    /// Edits a global command by its Id.
    pub async fn edit_global_application_command<F>(
        http: impl AsRef<Http>,
        command_id: CommandId,
        f: F,
    ) -> Result<ApplicationCommand>
    where
        F: FnOnce(&mut CreateInteraction) -> &mut CreateInteraction,
    {
        let map = Interaction::build_interaction(f);
        http.as_ref().edit_global_application_command(command_id.into(), &Value::Object(map)).await
    }

    /// Gets all global commands.
    pub async fn get_global_application_commands(
        http: impl AsRef<Http>,
    ) -> Result<Vec<ApplicationCommand>> {
        http.as_ref().get_global_application_commands().await
    }

    /// Gets a global command by its Id.
    pub async fn get_global_application_command(
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<ApplicationCommand> {
        http.as_ref().get_global_application_command(command_id.into()).await
    }

    /// Deletes a global command by its Id.
    pub async fn delete_global_application_command(
        http: impl AsRef<Http>,
        command_id: CommandId,
    ) -> Result<()> {
        http.as_ref().delete_global_application_command(command_id.into()).await
    }

    #[inline]
    pub(crate) fn build_interaction<F>(f: F) -> Map<String, Value>
    where
        F: FnOnce(&mut CreateInteraction) -> &mut CreateInteraction,
    {
        let mut create_interaction = CreateInteraction::default();
        f(&mut create_interaction);
        utils::hashmap_to_json_map(create_interaction.0)
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
    /// # Errors
    ///
    /// [`Error::Model`]: crate::error::Error::Model
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn create_interaction_response<F>(&self, http: impl AsRef<Http>, f: F) -> Result<()>
    where
        F: FnOnce(&mut CreateInteractionResponse) -> &mut CreateInteractionResponse,
    {
        let mut interaction_response = CreateInteractionResponse::default();
        f(&mut interaction_response);

        let map = utils::hashmap_to_json_map(interaction_response.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        http.as_ref().create_interaction_response(self.id.0, &self.token, &Value::Object(map)).await
    }

    /// Edits the initial interaction response.
    ///
    /// `application_id` will usually be the bot's `[UserId]`, except in cases of bots being very old.
    ///
    /// Refer to Discord's docs for Edit Webhook Message for field information.
    ///
    /// **Note**:   Message contents must be under 2000 unicode code points, does not work on ephemeral messages.
    ///
    /// [`UserId`]: crate::model::id::UserId
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
        application_id: u64,
        f: F,
    ) -> Result<Message>
    where
        F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse,
    {
        let mut interaction_response = EditInteractionResponse::default();
        f(&mut interaction_response);

        let map = utils::hashmap_to_json_map(interaction_response.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        http.as_ref()
            .edit_original_interaction_response(application_id, &self.token, &Value::Object(map))
            .await
    }

    /// Deletes the initial interaction response.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was already deleted.
    pub async fn delete_original_interaction_response(
        &self,
        http: impl AsRef<Http>,
        application_id: u64,
    ) -> Result<()> {
        http.as_ref().delete_original_interaction_response(application_id, &self.token).await
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
        application_id: u64,
        wait: bool,
        f: F,
    ) -> Result<Option<Message>>
    where
        for<'b> F: FnOnce(
            &'b mut CreateInteractionResponseFollowup<'a>,
        ) -> &'b mut CreateInteractionResponseFollowup<'a>,
    {
        let mut interaction_response = CreateInteractionResponseFollowup::default();
        f(&mut interaction_response);

        let map = utils::hashmap_to_json_map(interaction_response.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        http.as_ref().create_followup_message(application_id, &self.token, wait, &map).await
    }
}
