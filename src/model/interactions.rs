//! Interactions information-related models.

use super::prelude::*;
use crate::internal::prelude::*;

/// Information about an interaction.
///
/// An interaction is sent when a user invokes a slash command and is the same
/// for slash commands and other future interaction types.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub version: i32
}

/// The type of an Interaction
#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[non_exhaustive]
#[repr(u8)]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
}

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
