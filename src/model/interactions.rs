//! Interactions information-related models.

use super::prelude::*;
use crate::internal::prelude::*;

use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde_json::{Value, Number};

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
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub member: Member,
    pub token: String,
    pub version: u8,
}

impl<'de> Deserialize<'de> for Interaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("guild_id")
            .and_then(|x| x.as_str())
            .and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(member) = map.get_mut("member").and_then(|x| x.as_object_mut()) {
                member
                    .insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
            }
        }

        let id = map.remove("id")
            .ok_or_else(|| DeError::custom("expected id"))
            .and_then(InteractionId::deserialize)
            .map_err(DeError::custom)?;

        let kind = map.remove("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(InteractionType::deserialize)
            .map_err(DeError::custom)?;

        let data = match map.remove("data") {
            Some(v) => serde_json::from_value::<Option<ApplicationCommandInteractionData>>(v)
                .map_err(DeError::custom)?,
            None => None,
        };

        let guild_id = map.remove("guild_id")
            .ok_or_else(|| DeError::custom("expected guild_id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;

        let channel_id = map.remove("channel_id")
            .ok_or_else(|| DeError::custom("expected channel_id"))
            .and_then(ChannelId::deserialize)
            .map_err(DeError::custom)?;

        let member = map.remove("member")
            .ok_or_else(|| DeError::custom("expected member"))
            .and_then(Member::deserialize)
            .map_err(DeError::custom)?;

        let token = map.remove("token")
            .ok_or_else(|| DeError::custom("expected token"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        let version = map.remove("version")
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
    ApplicationCommand,
});

/// The command data payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandInteractionData {
    pub id: CommandId,
    pub name: String,
    #[serde(default)]
    pub options: Vec<ApplicationCommandInteractionDataOption>,
}

/// A set of a parameter and a value from the user.
///
/// All options have names and an option can either be a parameter and input `value` or it can denote a sub-command or group, in which case it will contain a
/// top-level key and another vector of `options`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandInteractionDataOption {
    pub name: String,
    pub value: Option<ApplicationCommandOptionType>,
    #[serde(default)]
    pub options: Vec<ApplicationCommandInteractionDataOption>,
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

/// The type of an application command option.
#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize)]
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

/// The only valid values a user can pick in a command option.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandOptionChoice {
    pub name: String,
    pub value: Value,
}
