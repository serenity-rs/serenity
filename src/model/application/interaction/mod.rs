pub mod application_command;
pub mod message_component;
pub mod modal;
pub mod ping;

use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};

use self::application_command::CommandInteraction;
use self::message_component::ComponentInteraction;
use self::modal::ModalInteraction;
use self::ping::PingInteraction;
use crate::internal::prelude::*;
use crate::json::from_value;
use crate::model::guild::PartialMember;
use crate::model::id::{ApplicationId, GuildId, InteractionId};
use crate::model::user::User;
use crate::model::utils::deserialize_val;
use crate::model::Permissions;

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object)
#[derive(Clone, Debug)]
pub enum Interaction {
    Ping(PingInteraction),
    Command(CommandInteraction),
    Autocomplete(CommandInteraction),
    Component(ComponentInteraction),
    Modal(ModalInteraction),
}

impl Interaction {
    /// Gets the interaction Id.
    #[must_use]
    pub fn id(&self) -> InteractionId {
        match self {
            Self::Ping(i) => i.id,
            Self::Command(i) | Self::Autocomplete(i) => i.id,
            Self::Component(i) => i.id,
            Self::Modal(i) => i.id,
        }
    }

    /// Gets the interaction type
    #[must_use]
    pub fn kind(&self) -> InteractionType {
        match self {
            Self::Ping(_) => InteractionType::Ping,
            Self::Command(_) => InteractionType::Command,
            Self::Component(_) => InteractionType::Component,
            Self::Autocomplete(_) => InteractionType::Autocomplete,
            Self::Modal(_) => InteractionType::Modal,
        }
    }

    /// Permissions the app or bot has within the channel the interaction was sent from.
    #[must_use]
    pub fn app_permissions(&self) -> Option<Permissions> {
        match self {
            Self::Ping(_) => None,
            Self::Command(i) | Self::Autocomplete(i) => i.app_permissions,
            Self::Component(i) => i.app_permissions,
            Self::Modal(i) => i.app_permissions,
        }
    }

    /// Gets the interaction application Id
    #[must_use]
    pub fn application_id(&self) -> ApplicationId {
        match self {
            Self::Ping(i) => i.application_id,
            Self::Command(i) | Self::Autocomplete(i) => i.application_id,
            Self::Component(i) => i.application_id,
            Self::Modal(i) => i.application_id,
        }
    }

    /// Gets the interaction token.
    #[must_use]
    pub fn token(&self) -> &str {
        match self {
            Self::Ping(i) => i.token.as_str(),
            Self::Command(i) | Self::Autocomplete(i) => i.token.as_str(),
            Self::Component(i) => i.token.as_str(),
            Self::Modal(i) => i.token.as_str(),
        }
    }

    /// Gets the invoked guild locale.
    #[must_use]
    pub fn guild_locale(&self) -> Option<&str> {
        match self {
            Self::Ping(i) => i.guild_locale.as_deref(),
            Self::Command(i) | Self::Autocomplete(i) => i.guild_locale.as_deref(),
            Self::Component(i) => i.guild_locale.as_deref(),
            Self::Modal(i) => i.guild_locale.as_deref(),
        }
    }

    /// Converts this to a [`PingInteraction`]
    #[must_use]
    pub fn ping(self) -> Option<PingInteraction> {
        match self {
            Self::Ping(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to an [`CommandInteraction`]
    #[must_use]
    pub fn application_command(self) -> Option<CommandInteraction> {
        match self {
            Self::Command(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ComponentInteraction`]
    #[must_use]
    pub fn message_component(self) -> Option<ComponentInteraction> {
        match self {
            Self::Component(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`CommandInteraction`]
    #[must_use]
    pub fn autocomplete(self) -> Option<CommandInteraction> {
        match self {
            Self::Autocomplete(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ModalInteraction`]
    #[must_use]
    pub fn modal_submit(self) -> Option<ModalInteraction> {
        match self {
            Self::Modal(i) => Some(i),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for Interaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let raw_kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?.clone();
        let value = Value::from(map);

        match deserialize_val(raw_kind)? {
            InteractionType::Command => from_value(value).map(Interaction::Command),
            InteractionType::Component => from_value(value).map(Interaction::Component),
            InteractionType::Autocomplete => from_value(value).map(Interaction::Autocomplete),
            InteractionType::Modal => from_value(value).map(Interaction::Modal),
            InteractionType::Ping => from_value(value).map(Interaction::Ping),
            InteractionType::Unknown(_) => return Err(DeError::custom("Unknown interaction type")),
        }
        .map_err(DeError::custom)
    }
}

impl Serialize for Interaction {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Ping(i) => i.serialize(serializer),
            Self::Command(i) | Self::Autocomplete(i) => i.serialize(serializer),
            Self::Component(i) => i.serialize(serializer),
            Self::Modal(i) => i.serialize(serializer),
        }
    }
}

enum_number! {
    /// The type of an Interaction.
    ///
    /// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-type).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum InteractionType {
        Ping = 1,
        Command = 2,
        Component = 3,
        Autocomplete = 4,
        Modal = 5,
        _ => Unknown(u8),
    }
}

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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
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
    /// The member who invoked the interaction in the guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<PartialMember>,
}

fn add_guild_id_to_resolved(map: &mut JsonMap, guild_id: GuildId) {
    if let Some(member) = map.get_mut("member").and_then(Value::as_object_mut) {
        member.insert("guild_id".to_string(), guild_id.get().into());
    }

    if let Some(data) = map.get_mut("data") {
        if let Some(resolved) = data.get_mut("resolved") {
            if let Some(roles) = resolved.get_mut("roles") {
                if let Some(values) = roles.as_object_mut() {
                    for value in values.values_mut() {
                        if let Some(role) = value.as_object_mut() {
                            role.insert("guild_id".to_string(), guild_id.get().into());
                        };
                    }
                }
            }
        }
    }
}
