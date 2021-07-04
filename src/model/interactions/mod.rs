pub mod application_command;
pub mod message_component;
pub mod ping;

use application_command::ApplicationCommandInteraction;
use bitflags::__impl_bitflags;
use message_component::MessageComponentInteraction;
use ping::PingInteraction;
use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};
use serde_json::Value;

use super::prelude::*;
use crate::internal::prelude::*;

#[derive(Clone, Debug)]
pub enum Interaction {
    Ping(PingInteraction),
    ApplicationCommand(ApplicationCommandInteraction),
    MessageComponent(MessageComponentInteraction),
}

impl Interaction {
    /// Gets the interaction Id.
    pub fn id(&self) -> InteractionId {
        match self {
            Interaction::Ping(i) => i.id,
            Interaction::ApplicationCommand(i) => i.id,
            Interaction::MessageComponent(i) => i.id,
        }
    }

    /// Gets the interaction type
    pub fn kind(&self) -> InteractionType {
        match self {
            Interaction::Ping(_) => InteractionType::Ping,
            Interaction::ApplicationCommand(_) => InteractionType::ApplicationCommand,
            Interaction::MessageComponent(_) => InteractionType::MessageComponent,
        }
    }

    /// Gets the interaction application Id
    pub fn application_id(&self) -> ApplicationId {
        match self {
            Interaction::Ping(i) => i.application_id,
            Interaction::ApplicationCommand(i) => i.application_id,
            Interaction::MessageComponent(i) => i.application_id,
        }
    }

    /// Gets the interaction token.
    pub fn token(&self) -> &str {
        match self {
            Interaction::Ping(ref i) => i.token.as_str(),
            Interaction::ApplicationCommand(i) => i.token.as_str(),
            Interaction::MessageComponent(i) => i.token.as_str(),
        }
    }

    /// Converts this to a [`PingInteraction`]
    pub fn ping(self) -> Option<PingInteraction> {
        match self {
            Interaction::Ping(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to an [`ApplicationCommandInteraction`]
    pub fn application_command(self) -> Option<ApplicationCommandInteraction> {
        match self {
            Interaction::ApplicationCommand(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`MessageComponentInteraction`]
    pub fn message_component(self) -> Option<MessageComponentInteraction> {
        match self {
            Interaction::MessageComponent(i) => Some(i),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for Interaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let kind = map
            .get("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(InteractionType::deserialize)
            .map_err(DeError::custom)?;

        match kind {
            InteractionType::Ping => serde_json::from_value::<PingInteraction>(Value::Object(map))
                .map(Interaction::Ping)
                .map_err(DeError::custom),
            InteractionType::ApplicationCommand => {
                serde_json::from_value::<ApplicationCommandInteraction>(Value::Object(map))
                    .map(Interaction::ApplicationCommand)
                    .map_err(DeError::custom)
            },
            InteractionType::MessageComponent => {
                serde_json::from_value::<MessageComponentInteraction>(Value::Object(map))
                    .map(Interaction::MessageComponent)
                    .map_err(DeError::custom)
            },
            InteractionType::Unknown => Err(DeError::custom("Unknown interaction type")),
        }
    }
}

impl Serialize for Interaction {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Interaction::Ping(i) => PingInteraction::serialize(i, serializer),
            Interaction::ApplicationCommand(i) => {
                ApplicationCommandInteraction::serialize(i, serializer)
            },
            Interaction::MessageComponent(i) => {
                MessageComponentInteraction::serialize(i, serializer)
            },
        }
    }
}

/// The type of an Interaction.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
    Unknown = !0,
}

enum_number!(InteractionType {
    Ping,
    MessageComponent,
    ApplicationCommand
});

/// The flags for an interaction response.
#[derive(Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InteractionApplicationCommandCallbackDataFlags {
    bits: u64,
}

__impl_bitflags! {
    InteractionApplicationCommandCallbackDataFlags: u64 {
        /// Interaction message will only be visible to sender and will
        /// be quickly deleted.
        EPHEMERAL = 0b0000_0000_0000_0000_0000_0000_0100_0000;
    }
}

/// Sent when a [`Message`] is a response to an [`Interaction`].
///
/// [`Message`]: crate::model::channel::Message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageInteraction {
    /// The id of the interaction.
    pub id: InteractionId,
    /// The type of the interaction.
    #[serde(rename = "type")]
    pub kind: InteractionType,
    /// The name of the [`ApplicationCommand`].
    ///
    /// [`ApplicationCommand`]: crate::model::interactions::application_command::ApplicationCommand
    pub name: String,
    /// The user who invoked the interaction.
    pub user: User,
}

/// The available responses types for an interaction response.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum InteractionResponseType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    DeferredChannelMessageWithSource = 5,
    DeferredUpdateMessage = 6,
    UpdateMessage = 7,
}
