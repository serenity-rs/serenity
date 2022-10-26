mod command;
pub use command::*;
use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};

use super::component::{ActionRow, ComponentType};
#[cfg(feature = "model")]
use crate::builder::{
    CreateAutocompleteResponse,
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    EditInteractionResponse,
};
#[cfg(feature = "model")]
use crate::http::Http;
use crate::internal::prelude::*;
use crate::model::channel::Message;
use crate::model::guild::{Member, PartialMember};
use crate::model::id::{ApplicationId, ChannelId, GuildId, InteractionId};
use crate::model::user::User;
use crate::model::utils::{deserialize_val, remove_from_map, remove_from_map_opt};
use crate::model::Permissions;

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object)
#[derive(Clone, Debug, Serialize)]
pub struct Interaction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The guild Id this interaction was sent from, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// The channel Id this interaction was sent from.
    pub channel_id: Option<ChannelId>,
    /// The `member` data for the invoking user.
    ///
    /// **Note**: It is only present if the interaction is triggered in a guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Box<Member>>,
    /// The `user` object for the invoking user.
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
    /// Type of this interaction and type-specific data.
    #[serde(flatten)]
    pub kind: InteractionKind,
}

#[cfg(feature = "model")]
impl Interaction {
    /// Gets the interaction response.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if there is no interaction response.
    pub async fn get_response(&self, http: impl AsRef<Http>) -> Result<Message> {
        http.as_ref().get_original_interaction_response(&self.token).await
    }

    /// Gets a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was deleted.
    pub async fn get_followup<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<Message> {
        http.as_ref().get_followup_message(&self.token, message_id.into()).await
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
        http: impl AsRef<Http>,
        builder: CreateInteractionResponse,
    ) -> Result<()> {
        builder.execute(http, self.id, &self.token).await
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
        http: impl AsRef<Http>,
        builder: CreateInteractionResponseFollowup,
    ) -> Result<Message> {
        builder.execute(http, None, &self.token).await
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
        http: impl AsRef<Http>,
        builder: EditInteractionResponse,
    ) -> Result<Message> {
        builder.execute(http, &self.token).await
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
        http: impl AsRef<Http>,
        message_id: impl Into<MessageId>,
        builder: CreateInteractionResponseFollowup,
    ) -> Result<Message> {
        builder.execute(http, Some(message_id.into()), &self.token).await
    }

    /// Deletes the initial interaction response.
    ///
    /// Does not work on ephemeral messages.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was already deleted.
    pub async fn delete_response(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_original_interaction_response(&self.token).await
    }

    /// Deletes a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was already deleted.
    pub async fn delete_followup<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<()> {
        http.as_ref().delete_followup_message(&self.token, message_id.into()).await
    }

    /// Converts this to an [`CommandInteraction`]
    #[must_use]
    pub fn command(self) -> Option<CommandData> {
        match self.kind {
            InteractionKind::Command(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to an [`CommandInteraction`]
    #[must_use]
    pub fn as_command(&self) -> Option<&CommandData> {
        match &self.kind {
            InteractionKind::Command(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to an [`CommandInteraction`]
    #[must_use]
    pub fn into_command(self) -> Option<CommandData> {
        self.command()
    }

    /// Converts this to a [`ComponentInteraction`]
    #[must_use]
    pub fn message_component(self) -> Option<ComponentData> {
        match self.kind {
            InteractionKind::Component(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ComponentInteraction`]
    #[must_use]
    pub fn as_message_component(&self) -> Option<&ComponentData> {
        match &self.kind {
            InteractionKind::Component(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ComponentInteraction`]
    #[must_use]
    pub fn into_message_component(self) -> Option<ComponentData> {
        self.message_component()
    }

    /// Converts this to a [`CommandInteraction`]
    #[must_use]
    pub fn autocomplete(self) -> Option<CommandData> {
        match self.kind {
            InteractionKind::Autocomplete(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`CommandInteraction`]
    #[must_use]
    pub fn as_autocomplete(&self) -> Option<&CommandData> {
        match &self.kind {
            InteractionKind::Autocomplete(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`CommandInteraction`]
    #[must_use]
    pub fn into_autocomplete(self) -> Option<CommandData> {
        self.autocomplete()
    }

    /// Converts this to a [`ModalInteraction`]
    #[must_use]
    pub fn modal_submit(self) -> Option<ModalData> {
        match self.kind {
            InteractionKind::Modal(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ModalInteraction`]
    #[must_use]
    pub fn as_modal_submit(&self) -> Option<&ModalData> {
        match &self.kind {
            InteractionKind::Modal(i) => Some(i),
            _ => None,
        }
    }

    /// Converts this to a [`ModalInteraction`]
    #[must_use]
    pub fn into_modal_submit(self) -> Option<ModalData> {
        self.modal_submit()
    }
}

impl<'de> Deserialize<'de> for Interaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = remove_from_map_opt::<GuildId, _>(&mut map, "guild_id")?;
        if let Some(guild_id) = guild_id {
            add_guild_id_to_resolved(&mut map, guild_id);
        }

        let member = remove_from_map_opt::<Box<Member>, _>(&mut map, "member")?;
        let user = remove_from_map_opt(&mut map, "user")?
            .or_else(|| member.as_ref().map(|m| m.user.clone()))
            .ok_or_else(|| DeError::custom("expected user or member"))?;

        Ok(Self {
            guild_id,
            member,
            user,
            id: remove_from_map(&mut map, "id")?,
            application_id: remove_from_map(&mut map, "application_id")?,
            channel_id: remove_from_map(&mut map, "channel_id")?,
            token: remove_from_map(&mut map, "token")?,
            version: remove_from_map(&mut map, "version")?,
            app_permissions: remove_from_map_opt(&mut map, "app_permissions")?,
            locale: remove_from_map(&mut map, "locale")?,
            guild_locale: remove_from_map_opt(&mut map, "guild_locale")?,
            kind: match remove_from_map(&mut map, "type")? {
                InteractionType::Ping => InteractionKind::Ping,
                InteractionType::Command => {
                    InteractionKind::Command(remove_from_map(&mut map, "data")?)
                },
                InteractionType::Component => {
                    let mut data: JsonMap = remove_from_map(&mut map, "data")?;
                    data.insert("message".into(), remove_from_map(&mut map, "message")?);
                    InteractionKind::Component(deserialize_val(data.into())?)
                },
                InteractionType::Autocomplete => {
                    InteractionKind::Autocomplete(remove_from_map(&mut map, "data")?)
                },
                InteractionType::Modal => {
                    InteractionKind::Modal(remove_from_map(&mut map, "data")?)
                },
                InteractionType::Unknown(x) => InteractionKind::Unknown(x),
            },
        })
    }
}

/// Contains all type-specific data of an [`Interaction`].
#[derive(Clone, Debug)]
pub enum InteractionKind {
    Ping,
    Command(CommandData),
    Component(ComponentData),
    Autocomplete(CommandData),
    Modal(ModalData),
    Unknown(u8),
}

macro_rules! getter_function {
    ($fn_name:ident, $variant_name:ident, $type_name:ident) => {
        #[doc = concat!("Converts this to a [`", stringify!($type_name), "`].")]
        #[must_use]
        pub fn $fn_name(self) -> Option<$type_name> {
            match self {
                Self::$variant_name(i) => Some(i),
                _ => None,
            }
        }
    };
}

impl InteractionKind {
    getter_function!(command, Command, CommandData);
    getter_function!(component, Component, ComponentData);
    getter_function!(autocomplete, Autocomplete, CommandData);
    getter_function!(modal, Modal, ModalData);
}

impl Serialize for InteractionKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        use serde::ser::Error as _;

        #[derive(Serialize)]
        struct Json {
            r#type: InteractionType,
            data: Option<serde_json::Value>,
        }

        match self {
            Self::Ping => Json {
                r#type: InteractionType::Ping,
                data: None,
            },
            Self::Command(x) => Json {
                r#type: InteractionType::Command,
                data: Some(serde_json::to_value(x).map_err(S::Error::custom)?),
            },
            Self::Component(x) => Json {
                r#type: InteractionType::Component,
                data: Some(serde_json::to_value(x).map_err(S::Error::custom)?),
            },
            Self::Autocomplete(x) => Json {
                r#type: InteractionType::Autocomplete,
                data: Some(serde_json::to_value(x).map_err(S::Error::custom)?),
            },
            Self::Modal(x) => Json {
                r#type: InteractionType::Modal,
                data: Some(serde_json::to_value(x).map_err(S::Error::custom)?),
            },
            Self::Unknown(x) => Json {
                r#type: InteractionType::Unknown(*x),
                data: Some(serde_json::to_value(*x).map_err(S::Error::custom)?),
            },
        }
        .serialize(serializer)
    }
}

/// A message component interaction data, provided by [`InteractionKind::Component`]
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ComponentData {
    /// The message this interaction was triggered by.
    pub message: Box<Message>,
    /// The custom id of the component.
    pub custom_id: String,
    /// The type of the component.
    pub component_type: ComponentType,
    /// The given values of the [`SelectMenu`]s
    ///
    /// [`SelectMenu`]: crate::model::application::component::SelectMenu
    #[serde(default)]
    pub values: Vec<String>,
}

/// A modal submit interaction data, provided by [`InteractionKind::Modal`]
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ModalData {
    /// The custom id of the modal
    pub custom_id: String,
    /// The components.
    pub components: Vec<ActionRow>,
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
