//! Interactions information-related models.

use super::prelude::*;
use crate::internal::prelude::*;
use crate::http::Http;
use crate::builder::{CreateInteractionResponse, CreateInteractionResponseFollowup, EditInteractionResponse, CreateInteraction};
use crate::utils;

use bitflags::__impl_bitflags;
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
    pub value: Option<Value>,
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
    Acknowledge = 2,
    ChannelMessage = 3,
    ChannelMessageWithSource = 4,
    AcknowledgeWithSource = 5
}

#[derive(Clone, Serialize)]
#[non_exhaustive]
pub struct InteractionApplicationCommandCallbackDataFlags {
    bits: u64
}

__impl_bitflags! {
    InteractionApplicationCommandCallbackDataFlags: u64 {
        /// Interaction message will only be visible to sender
        EPHEMERAL = 0b0000_0000_0000_0000_0000_0000_0100_0000;
    }
}

impl Interaction {
    /// Creates a response to the interaction received.
    ///
    /// Note: Message contents must be under 2000 unicode code points.
    pub async fn create_interaction_response<F>(&self, http: impl AsRef<Http>, f: F) 
        where for <'b> F: FnOnce(&'b mut CreateInteractionResponse) -> &'b mut CreateInteractionResponse {
            let mut interaction_response = CreateInteractionResponse::default();
            let interaction_response = f(&mut interaction_response);

            let map = utils::hashmap_to_json_map(interaction_response.0.clone());

            Message::check_content_length(&map).unwrap();
            Message::check_embed_length(&map).unwrap();    

            http.as_ref().create_interaction_response(self.id.0, &self.token, &Value::Object(map)).await.unwrap();
        }

    /// Edits the initial interaction response.
    /// 
    /// Refer to Discord's docs for Edit Webhook Message for field information.
    /// 
    /// Note:   Message contents must be under 2000 unicode code points, does not work on ephemeral messages.
    pub async fn edit_original_interaction_response<F>(&self, http: impl AsRef<Http>, application_id: u64, f: F) 
        where for <'b> F: FnOnce(&'b mut EditInteractionResponse) -> &'b mut EditInteractionResponse {
            let mut interaction_response = EditInteractionResponse::default();
            let interaction_response = f(&mut interaction_response);

            let map = utils::hashmap_to_json_map(interaction_response.0.clone());

            Message::check_content_length(&map).unwrap();
            Message::check_embed_length(&map).unwrap();    

            http.as_ref().edit_original_interaction_response(application_id, &self.token, &Value::Object(map)).await.unwrap();
        }

    /// Deletes the initial interaction response.
    pub async fn delete_original_interaction_response(&self, http: impl AsRef<Http>, application_id: u64) {
        http.as_ref().delete_original_interaction_response(application_id, &self.token).await.unwrap();
    }

    /// Creates a followup response to the response sent.
    ///
    /// Note: Message contents must be under 2000 unicode code points.
    pub async fn create_followup_message<'a, F>(&self, http: impl AsRef<Http>, application_id: u64, wait: bool, f: F) 
        where for <'b> F: FnOnce(&'b mut CreateInteractionResponseFollowup<'a>) -> &'b mut CreateInteractionResponseFollowup<'a> {
            let mut interaction_response = CreateInteractionResponseFollowup::default();
            let interaction_response = f(&mut interaction_response);

            let map = utils::hashmap_to_json_map(interaction_response.0.clone());

            Message::check_content_length(&map).unwrap();
            Message::check_embed_length(&map).unwrap();    

            http.as_ref().create_followup_message(application_id, &self.token, wait, &map).await.unwrap();
        }

    
}