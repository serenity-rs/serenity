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
use crate::internal::prelude::*;
use crate::json::{from_number, from_value};
use crate::model::id::{ApplicationId, GuildId, InteractionId};
use crate::model::user::User;
use crate::model::utils::deserialize_val;

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
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let map = value.as_object().ok_or_else(|| DeError::custom("expected JsonMap"))?;

        let raw_kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?;
        match deserialize_val(raw_kind.clone())? {
            InteractionType::ApplicationCommand => {
                from_value(value).map(Interaction::ApplicationCommand)
            },
            InteractionType::MessageComponent => {
                from_value(value).map(Interaction::MessageComponent)
            },
            InteractionType::Autocomplete => from_value(value).map(Interaction::Autocomplete),
            InteractionType::ModalSubmit => from_value(value).map(Interaction::ModalSubmit),
            InteractionType::Ping => from_value(value).map(Interaction::Ping),
            InteractionType::Unknown => return Err(DeError::custom("Unknown interaction type")),
        }
        .map_err(DeError::custom)
    }
}

impl Serialize for Interaction {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Interaction::Ping(i) => i.serialize(serializer),
            Interaction::ApplicationCommand(i) => i.serialize(serializer),
            Interaction::MessageComponent(i) => i.serialize(serializer),
            Interaction::Autocomplete(i) => i.serialize(serializer),
            Interaction::ModalSubmit(i) => i.serialize(serializer),
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
    Autocomplete = 8,
    Modal = 9,
}

impl serde::Serialize for InteractionResponseType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

fn add_guild_id_to_resolved(map: &mut JsonMap, guild_id: GuildId) {
    if let Some(member) = map.get_mut("member").and_then(Value::as_object_mut) {
        member.insert("guild_id".to_string(), from_number(guild_id.0));
    }

    if let Some(data) = map.get_mut("data") {
        if let Some(resolved) = data.get_mut("resolved") {
            if let Some(roles) = resolved.get_mut("roles") {
                if let Some(values) = roles.as_object_mut() {
                    for value in values.values_mut() {
                        if let Some(role) = value.as_object_mut() {
                            role.insert("guild_id".to_string(), from_number(guild_id.0));
                        };
                    }
                }
            }
        }
    }
}
