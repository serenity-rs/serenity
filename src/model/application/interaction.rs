use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};
use serde_json::from_value;

#[cfg(feature = "unstable_discord_api")]
use super::InstallationContext;
use super::{CommandInteraction, ComponentInteraction, ModalInteraction, PingInteraction};
use crate::internal::prelude::*;
use crate::model::guild::PartialMember;
use crate::model::id::{ApplicationId, InteractionId};
#[cfg(feature = "unstable_discord_api")]
use crate::model::id::{GuildId, MessageId, UserId};
use crate::model::monetization::Entitlement;
use crate::model::user::User;
use crate::model::utils::deserialize_val;
#[cfg(feature = "unstable_discord_api")]
use crate::model::utils::StrOrInt;
use crate::model::Permissions;

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
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
            Self::Ping(_) => None,
            Self::Command(i) | Self::Autocomplete(i) => i.guild_locale.as_deref(),
            Self::Component(i) => i.guild_locale.as_deref(),
            Self::Modal(i) => i.guild_locale.as_deref(),
        }
    }

    /// For monetized applications, gets the invoking user's granted entitlements.
    #[must_use]
    pub fn entitlements(&self) -> Option<&[Entitlement]> {
        match self {
            Self::Ping(_) => None,
            Self::Command(i) | Self::Autocomplete(i) => Some(&i.entitlements),
            Self::Component(i) => Some(&i.entitlements),
            Self::Modal(i) => Some(&i.entitlements),
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

    /// Converts this to a [`PingInteraction`]
    #[must_use]
    pub fn as_ping(&self) -> Option<&PingInteraction> {
        match self {
            Self::Ping(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`PingInteraction`]
    #[must_use]
    pub fn into_ping(self) -> Option<PingInteraction> {
        self.ping()
    }

    /// Converts this to an [`CommandInteraction`]
    #[must_use]
    pub fn command(self) -> Option<CommandInteraction> {
        match self {
            Self::Command(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to an [`CommandInteraction`]
    #[must_use]
    pub fn as_command(&self) -> Option<&CommandInteraction> {
        match self {
            Self::Command(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to an [`CommandInteraction`]
    #[must_use]
    pub fn into_command(self) -> Option<CommandInteraction> {
        self.command()
    }

    /// Converts this to a [`ComponentInteraction`]
    #[must_use]
    pub fn message_component(self) -> Option<ComponentInteraction> {
        match self {
            Self::Component(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ComponentInteraction`]
    #[must_use]
    pub fn as_message_component(&self) -> Option<&ComponentInteraction> {
        match self {
            Self::Component(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ComponentInteraction`]
    #[must_use]
    pub fn into_message_component(self) -> Option<ComponentInteraction> {
        self.message_component()
    }

    /// Converts this to a [`CommandInteraction`]
    #[must_use]
    pub fn autocomplete(self) -> Option<CommandInteraction> {
        match self {
            Self::Autocomplete(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`CommandInteraction`]
    #[must_use]
    pub fn as_autocomplete(&self) -> Option<&CommandInteraction> {
        match self {
            Self::Autocomplete(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`CommandInteraction`]
    #[must_use]
    pub fn into_autocomplete(self) -> Option<CommandInteraction> {
        self.autocomplete()
    }

    /// Converts this to a [`ModalInteraction`]
    #[must_use]
    pub fn modal_submit(self) -> Option<ModalInteraction> {
        match self {
            Self::Modal(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ModalInteraction`]
    #[must_use]
    pub fn as_modal_submit(&self) -> Option<&ModalInteraction> {
        match self {
            Self::Modal(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ModalInteraction`]
    #[must_use]
    pub fn into_modal_submit(self) -> Option<ModalInteraction> {
        self.modal_submit()
    }
}

// Manual impl needed to emulate integer enum tags
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
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
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
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct InteractionResponseFlags: u64 {
        /// Do not include any embeds when serializing this message.
        const SUPPRESS_EMBEDS = 1 << 2;
        /// Interaction message will only be visible to sender and will
        /// be quickly deleted.
        const EPHEMERAL = 1 << 6;
        /// Does not trigger push notifications or desktop notifications.
        const SUPPRESS_NOTIFICATIONS = 1 << 12;
    }
}

/// A cleaned up enum for determining the authorizing owner for an [`Interaction`].
///
/// [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-authorizing-integration-owners-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[cfg(feature = "unstable_discord_api")]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AuthorizingIntegrationOwner {
    /// The [`Application`] was installed to a guild, containing the id if invoked in said guild.
    ///
    /// [`Application`]: super::CurrentApplicationInfo
    GuildInstall(Option<GuildId>),
    /// The [`Application`] was installed to a user, containing the id of said user.
    ///
    /// [`Application`]: super::CurrentApplicationInfo
    UserInstall(UserId),
    Unknown(InstallationContext),
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[cfg(feature = "unstable_discord_api")]
#[derive(Clone, Debug, Default)]
#[repr(transparent)]
pub struct AuthorizingIntegrationOwners(Vec<AuthorizingIntegrationOwner>);

#[cfg(feature = "unstable_discord_api")]
impl<'de> serde::Deserialize<'de> for AuthorizingIntegrationOwners {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = AuthorizingIntegrationOwners;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a hashmap containing keys of InstallationContext and values based on those keys")
            }

            fn visit_map<A>(self, mut map: A) -> StdResult<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut out = Vec::new();
                while let Some(key_str) = map.next_key::<serde_cow::CowStr<'_>>()? {
                    let key_int = key_str.0.parse::<u8>().map_err(serde::de::Error::custom)?;
                    let value = match InstallationContext::from(key_int) {
                        InstallationContext::Guild => {
                            // GuildId here can be `0`, which signals the command is guild installed
                            // but invoked in a DM, we have to do this fun deserialisation dance.
                            let id_str = map.next_value::<StrOrInt<'_>>()?;
                            let id_int = id_str.parse().map_err(A::Error::custom)?;
                            let id = if id_int == 0 {
                                None
                            } else if id_int == u64::MAX {
                                return Err(serde::de::Error::custom("GuildId cannot be u64::MAX"));
                            } else {
                                Some(GuildId::new(id_int))
                            };

                            AuthorizingIntegrationOwner::GuildInstall(id)
                        },
                        InstallationContext::User => {
                            AuthorizingIntegrationOwner::UserInstall(map.next_value()?)
                        },
                        key => AuthorizingIntegrationOwner::Unknown(key),
                    };

                    out.push(value);
                }

                Ok(AuthorizingIntegrationOwners(out))
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

#[cfg(feature = "unstable_discord_api")]
impl serde::Serialize for AuthorizingIntegrationOwners {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        use serde::ser::SerializeMap;

        let mut serializer = serializer.serialize_map(Some(self.0.len()))?;
        for value in &self.0 {
            match value {
                AuthorizingIntegrationOwner::GuildInstall(inner) => {
                    serializer.serialize_entry(&InstallationContext::Guild, &inner)
                },
                AuthorizingIntegrationOwner::UserInstall(inner) => {
                    serializer.serialize_entry(&InstallationContext::User, &inner)
                },
                AuthorizingIntegrationOwner::Unknown(inner) => {
                    serializer.serialize_entry(&inner, &())
                },
            }?;
        }

        serializer.end()
    }
}

/// Sent when a [`Message`] is a response to an [`Interaction`].
///
/// [`Message`]: crate::model::channel::Message
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#message-interaction-object).
#[cfg_attr(
    all(not(ignore_serenity_deprecated), feature = "unstable_discord_api"),
    deprecated = "Use Message::interaction_metadata"
)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
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
    /// [`Command`]: crate::model::application::Command
    pub name: FixedString<u8>,
    /// The user who invoked the interaction.
    pub user: User,
    /// The member who invoked the interaction in the guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<PartialMember>,
}

/// Metadata about the interaction, including the source of the interaction relevant server and
/// user IDs.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable_discord_api")]
pub struct MessageInteractionMetadata {
    /// The ID of the interaction
    pub id: InteractionId,
    /// The type of interaction
    #[serde(rename = "type")]
    pub kind: InteractionType,
    /// The ID of the user who triggered the interaction
    pub user: User,
    /// The IDs for installation context(s) related to an interaction.
    #[serde(default)]
    pub authorizing_integration_owners: AuthorizingIntegrationOwners,
    /// The ID of the original response message, present only on follow-up messages.
    pub original_response_message_id: Option<MessageId>,
    /// ID of the message that contained interactive component, present only on messages created
    /// from component interactions.
    pub interacted_message_id: Option<MessageId>,
    /// Metadata for the interaction that was used to open the modal, present only on modal submit
    /// interactions
    pub triggering_interaction_metadata: Option<Box<MessageInteractionMetadata>>,
}
