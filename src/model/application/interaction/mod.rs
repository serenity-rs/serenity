pub mod application_command;
pub mod autocomplete;
pub mod message_component;
pub mod modal;
pub mod ping;

use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};

use self::application_command::ApplicationCommandInteraction;
use self::autocomplete::AutocompleteInteraction;
use self::message_component::MessageComponentInteraction;
use self::modal::ModalSubmitInteraction;
use self::ping::PingInteraction;
use crate::json::{from_value, JsonMap, Value};
use crate::model::guild::PartialMember;
use crate::model::id::{ApplicationId, InteractionId};
use crate::model::user::User;

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object)
#[derive(Clone, Debug)]
pub enum Interaction {
    Ping(PingInteraction),
    ApplicationCommand(ApplicationCommandInteraction),
    MessageComponent(MessageComponentInteraction),
    Autocomplete(AutocompleteInteraction),
    ModalSubmit(ModalSubmitInteraction),
}

impl Interaction {
    /// Gets the interaction Id.
    #[must_use]
    pub fn id(&self) -> InteractionId {
        match self {
            Interaction::Ping(i) => i.id,
            Interaction::ApplicationCommand(i) => i.id,
            Interaction::MessageComponent(i) => i.id,
            Interaction::Autocomplete(i) => i.id,
            Interaction::ModalSubmit(i) => i.id,
        }
    }

    /// Gets the interaction type
    #[must_use]
    pub fn kind(&self) -> InteractionType {
        match self {
            Interaction::Ping(_) => InteractionType::Ping,
            Interaction::ApplicationCommand(_) => InteractionType::ApplicationCommand,
            Interaction::MessageComponent(_) => InteractionType::MessageComponent,
            Interaction::Autocomplete(_) => InteractionType::Autocomplete,
            Interaction::ModalSubmit(_) => InteractionType::ModalSubmit,
        }
    }

    /// Gets the interaction application Id
    #[must_use]
    pub fn application_id(&self) -> ApplicationId {
        match self {
            Interaction::Ping(i) => i.application_id,
            Interaction::ApplicationCommand(i) => i.application_id,
            Interaction::MessageComponent(i) => i.application_id,
            Interaction::Autocomplete(i) => i.application_id,
            Interaction::ModalSubmit(i) => i.application_id,
        }
    }

    /// Gets the interaction token.
    #[must_use]
    pub fn token(&self) -> &str {
        match self {
            Interaction::Ping(ref i) => i.token.as_str(),
            Interaction::ApplicationCommand(i) => i.token.as_str(),
            Interaction::MessageComponent(i) => i.token.as_str(),
            Interaction::Autocomplete(i) => i.token.as_str(),
            Interaction::ModalSubmit(i) => i.token.as_str(),
        }
    }

    /// Gets the invoked guild locale.
    #[must_use]
    pub fn guild_locale(&self) -> Option<&str> {
        match self {
            Interaction::Ping(i) => i.guild_locale.as_deref(),
            Interaction::ApplicationCommand(i) => i.guild_locale.as_deref(),
            Interaction::MessageComponent(i) => i.guild_locale.as_deref(),
            Interaction::Autocomplete(i) => i.guild_locale.as_deref(),
            Interaction::ModalSubmit(i) => i.guild_locale.as_deref(),
        }
    }

    /// Converts this to a [`PingInteraction`]
    #[must_use]
    pub fn ping(self) -> Option<PingInteraction> {
        match self {
            Interaction::Ping(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to an [`ApplicationCommandInteraction`]
    #[must_use]
    pub fn application_command(self) -> Option<ApplicationCommandInteraction> {
        match self {
            Interaction::ApplicationCommand(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`MessageComponentInteraction`]
    #[must_use]
    pub fn message_component(self) -> Option<MessageComponentInteraction> {
        match self {
            Interaction::MessageComponent(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`AutocompleteInteraction`]
    #[must_use]
    pub fn autocomplete(self) -> Option<AutocompleteInteraction> {
        match self {
            Interaction::Autocomplete(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ModalSubmitInteraction`]
    #[must_use]
    pub fn modal_submit(self) -> Option<ModalSubmitInteraction> {
        match self {
            Interaction::ModalSubmit(i) => Some(i),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for Interaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let kind = map
            .get("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(InteractionType::deserialize)
            .map_err(DeError::custom)?;

        match kind {
            InteractionType::Ping => from_value::<PingInteraction>(Value::from(map))
                .map(Interaction::Ping)
                .map_err(DeError::custom),
            InteractionType::ApplicationCommand => {
                from_value::<ApplicationCommandInteraction>(Value::from(map))
                    .map(Interaction::ApplicationCommand)
                    .map_err(DeError::custom)
            },
            InteractionType::MessageComponent => {
                from_value::<MessageComponentInteraction>(Value::from(map))
                    .map(Interaction::MessageComponent)
                    .map_err(DeError::custom)
            },
            InteractionType::Autocomplete => {
                from_value::<AutocompleteInteraction>(Value::from(map))
                    .map(Interaction::Autocomplete)
                    .map_err(DeError::custom)
            },
            InteractionType::ModalSubmit => from_value::<ModalSubmitInteraction>(Value::from(map))
                .map(Interaction::ModalSubmit)
                .map_err(DeError::custom),
            InteractionType::Unknown => Err(DeError::custom("Unknown interaction type")),
        }
    }
}

impl Serialize for Interaction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Interaction::Ping(i) => PingInteraction::serialize(i, serializer),
            Interaction::ApplicationCommand(i) => {
                ApplicationCommandInteraction::serialize(i, serializer)
            },
            Interaction::MessageComponent(i) => {
                MessageComponentInteraction::serialize(i, serializer)
            },
            Interaction::Autocomplete(i) => AutocompleteInteraction::serialize(i, serializer),
            Interaction::ModalSubmit(i) => ModalSubmitInteraction::serialize(i, serializer),
        }
    }
}

/// The type of an Interaction.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-type).
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
    Autocomplete = 4,
    ModalSubmit = 5,
    Unknown = !0,
}

enum_number!(InteractionType {
    Ping,
    MessageComponent,
    ApplicationCommand,
    Autocomplete,
    ModalSubmit
});

bitflags! {
    /// The flags for an interaction response message.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#message-object-message-flags)
    /// ([only some are valid in this context](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-messages))
    #[derive(Default)]
    pub struct MessageFlags: u64 {
        /// Do not include any embeds when serializing this message.
        const SUPPRESS_EMBEDS = 1 << 2;
        /// Interaction message will only be visible to sender and will
        /// be quickly deleted.
        const EPHEMERAL = 1 << 6;
    }
}

/// Sent when a [`Message`] is a response to an [`Interaction`].
///
/// [`Message`]: crate::model::channel::Message
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#message-interaction-object).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageInteraction {
    /// The id of the interaction.
    pub id: InteractionId,
    /// The type of the interaction.
    #[serde(rename = "type")]
    pub kind: InteractionType,
    /// The name of the [`Command`].
    ///
    /// [`Command`]: crate::model::application::command::Command
    pub name: String,
    /// The user who invoked the interaction.
    pub user: User,
    /// If in a guild, the member who invoked the interaction.
    pub member: Option<PartialMember>,
}

/// The available responses types for an interaction response.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-interaction-callback-type).
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum InteractionResponseType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    DeferredChannelMessageWithSource = 5,
    DeferredUpdateMessage = 6,
    UpdateMessage = 7,
    Autocomplete = 8,
    Modal = 9,
}
