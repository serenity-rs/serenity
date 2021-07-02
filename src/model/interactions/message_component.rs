use std::convert::TryFrom;

use serde::de::Error as DeError;
use serde::{Serialize, Serializer};

use super::prelude::*;
use crate::builder::{
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    EditInteractionResponse,
};
use crate::http::Http;
use crate::model::interactions::InteractionType;
use crate::utils;

/// An interaction triggered by a message component.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct MessageComponentInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The type of interaction.
    #[serde(rename = "type")]
    pub kind: InteractionType,
    /// The data of the interaction which was triggered.
    pub data: MessageComponentInteractionData,
    /// The message this interaction was triggered by, if
    /// it is a component.
    pub message: InteractionMessage,
    /// The guild Id this interaction was sent from, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// The channel Id this interaction was sent from.
    pub channel_id: ChannelId,
    /// The `member` data for the invoking user.
    ///
    /// **Note**: It is only present if the interaction is triggered in a guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Member>,
    /// The `user` object for the invoking user.
    pub user: User,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Always `1`.
    pub version: u8,
}

impl MessageComponentInteraction {
    /// Gets the interaction response.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if there is no interaction response.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    pub async fn get_interaction_response(&self, http: impl AsRef<Http>) -> Result<Message> {
        http.as_ref().get_original_interaction_response(&self.token).await
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
    /// `application_id` will usually be the bot's [`UserId`], except in cases of bots being very old.
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

        http.as_ref().edit_original_interaction_response(&self.token, &Value::Object(map)).await
    }

    /// Deletes the initial interaction response.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was already deleted.
    pub async fn delete_original_interaction_response(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_original_interaction_response(&self.token).await
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
        f: F,
    ) -> Result<Message>
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

        http.as_ref().create_followup_message(&self.token, &Value::Object(map)).await
    }

    /// Edits a followup response to the response sent.
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
    pub async fn edit_followup_message<'a, F, M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
        f: F,
    ) -> Result<Message>
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

        http.as_ref()
            .edit_followup_message(&self.token, message_id.into().into(), &Value::Object(map))
            .await
    }

    /// Deletes a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was already deleted.
    pub async fn delete_followup_message<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<()> {
        http.as_ref().delete_followup_message(&self.token, message_id.into().into()).await
    }
}

impl<'de> Deserialize<'de> for MessageComponentInteraction {
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

        let application_id = map
            .remove("application_id")
            .ok_or_else(|| DeError::custom("expected application id"))
            .and_then(ApplicationId::deserialize)
            .map_err(DeError::custom)?;

        let kind = map
            .remove("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(InteractionType::deserialize)
            .map_err(DeError::custom)?;

        let data = map
            .remove("data")
            .ok_or_else(|| DeError::custom("expected data"))
            .and_then(MessageComponentInteractionData::deserialize)
            .map_err(DeError::custom)?;

        let guild_id = match map.contains_key("guild_id") {
            true => Some(
                map.remove("guild_id")
                    .ok_or_else(|| DeError::custom("expected guild_id"))
                    .and_then(GuildId::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let channel_id = map
            .remove("channel_id")
            .ok_or_else(|| DeError::custom("expected channel_id"))
            .and_then(ChannelId::deserialize)
            .map_err(DeError::custom)?;

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
            true => map
                .remove("user")
                .ok_or_else(|| DeError::custom("expected user"))
                .and_then(User::deserialize)
                .map_err(DeError::custom)?,
            false => member.as_ref().expect("expected user or member").user.clone(),
        };

        let message = {
            let message = map
                .remove("message")
                .ok_or_else(|| DeError::custom("expected message"))
                .and_then(JsonMap::deserialize)
                .map_err(DeError::custom)?;

            let partial = !message.contains_key("author");

            let value: Value = message.into();

            if partial {
                InteractionMessage::Ephemeral(
                    EphemeralMessage::deserialize(value).map_err(DeError::custom)?,
                )
            } else {
                InteractionMessage::Regular(Message::deserialize(value).map_err(DeError::custom)?)
            }
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
            application_id,
            kind,
            data,
            message,
            guild_id,
            channel_id,
            member,
            user,
            token,
            version,
        })
    }
}

/// A message component interaction data, provided by [`MessageComponentInteraction::data`]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageComponentInteractionData {
    /// The custom id of the component.
    pub custom_id: String,
    /// The type of the component.
    pub component_type: ComponentType,
    /// The given values of the [`SelectMenu`]s
    #[serde(default)]
    pub values: Vec<String>,
}

// A component.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Component {
    ActionRow(ActionRow),
    Button(Button),
    SelectMenu(SelectMenu),
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let kind = map
            .get("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(ComponentType::deserialize)
            .map_err(DeError::custom)?;

        match kind {
            ComponentType::ActionRow => serde_json::from_value::<ActionRow>(Value::Object(map))
                .map(Component::ActionRow)
                .map_err(DeError::custom),
            ComponentType::Button => serde_json::from_value::<Button>(Value::Object(map))
                .map(Component::Button)
                .map_err(DeError::custom),
            ComponentType::SelectMenu => serde_json::from_value::<SelectMenu>(Value::Object(map))
                .map(Component::SelectMenu)
                .map_err(DeError::custom),
            ComponentType::Unknown => Err(DeError::custom("Unknown component type")),
        }
    }
}

impl Serialize for Component {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Component::ActionRow(c) => ActionRow::serialize(c, serializer),
            Component::Button(c) => Button::serialize(c, serializer),
            Component::SelectMenu(c) => SelectMenu::serialize(c, serializer),
        }
    }
}

impl From<ActionRow> for Component {
    fn from(component: ActionRow) -> Self {
        Component::ActionRow(component)
    }
}

impl From<Button> for Component {
    fn from(component: Button) -> Self {
        Component::Button(component)
    }
}

impl From<SelectMenu> for Component {
    fn from(component: SelectMenu) -> Self {
        Component::SelectMenu(component)
    }
}

/// The type of a component
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum ComponentType {
    ActionRow = 1,
    Button = 2,
    SelectMenu = 3,
    Unknown = !0,
}

enum_number!(ComponentType {
    ActionRow,
    Button,
    SelectMenu
});

/// An action row.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionRow {
    /// The type of component this ActionRow is.
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The components of this ActionRow.
    #[serde(default)]
    pub components: Vec<ActionRowComponent>,
}

// A component which can be inside of an [`ActionRow`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ActionRowComponent {
    Button(Button),
    SelectMenu(SelectMenu),
}

impl<'de> Deserialize<'de> for ActionRowComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let kind = map
            .get("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(ComponentType::deserialize)
            .map_err(DeError::custom)?;

        match kind {
            ComponentType::Button => serde_json::from_value::<Button>(Value::Object(map))
                .map(ActionRowComponent::Button)
                .map_err(DeError::custom),
            ComponentType::SelectMenu => serde_json::from_value::<SelectMenu>(Value::Object(map))
                .map(ActionRowComponent::SelectMenu)
                .map_err(DeError::custom),
            _ => Err(DeError::custom("Unknown component type")),
        }
    }
}

impl Serialize for ActionRowComponent {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ActionRowComponent::Button(c) => Button::serialize(c, serializer),
            ActionRowComponent::SelectMenu(c) => SelectMenu::serialize(c, serializer),
        }
    }
}

impl From<ActionRowComponent> for Component {
    fn from(component: ActionRowComponent) -> Self {
        match component {
            ActionRowComponent::Button(b) => Component::Button(b),
            ActionRowComponent::SelectMenu(s) => Component::SelectMenu(s),
        }
    }
}

impl TryFrom<Component> for ActionRowComponent {
    type Error = Error;

    fn try_from(value: Component) -> Result<Self> {
        match value {
            Component::ActionRow(_) => Err(Error::Model(ModelError::InvalidComponentType)),
            Component::Button(b) => Ok(ActionRowComponent::Button(b)),
            Component::SelectMenu(s) => Ok(ActionRowComponent::SelectMenu(s)),
        }
    }
}

impl From<Button> for ActionRowComponent {
    fn from(component: Button) -> Self {
        ActionRowComponent::Button(component)
    }
}

impl From<SelectMenu> for ActionRowComponent {
    fn from(component: SelectMenu) -> Self {
        ActionRowComponent::SelectMenu(component)
    }
}

/// A button component.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Button {
    /// The component type, it will always be [`ComponentType::Button`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The button style.
    pub style: ButtonStyle,
    /// The text which appears on the button.
    pub label: Option<String>,
    /// The emoji of this button, if there is one.
    pub emoji: Option<ReactionType>,
    /// An identifier defined by the developer for the button.
    pub custom_id: Option<String>,
    /// The url of the button, if there is one.
    pub url: Option<String>,
    /// Whether the button is disabled.
    #[serde(default)]
    pub disabled: bool,
}

/// The style of a button.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum ButtonStyle {
    Primary = 1,
    Secondary = 2,
    Success = 3,
    Danger = 4,
    Link = 5,
    Unknown = !0,
}

enum_number!(ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
    Link
});

/// A select menu component.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectMenu {
    /// The component type, it will always be [`ComponentType::SelectMenu`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The placeholder shown when nothing is selected.
    pub placeholder: Option<String>,
    /// An identifier defined by the developer for the select menu.
    pub custom_id: Option<String>,
    /// The minimum number of selections allowed.
    pub min_values: Option<u64>,
    /// The maximum number of selections allowed.
    pub max_values: Option<u64>,
    /// The options of this select menu.
    #[serde(default)]
    pub options: Vec<SelectMenuOption>,
}

/// A select menu component options.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectMenuOption {
    /// The text displayed on this option.
    pub label: String,
    /// The value to be sent for this option.
    pub value: String,
    /// The description shown for this option.
    pub description: Option<String>,
    /// The emoji displayed on this option.
    pub emoji: Option<ReactionType>,
    /// Render this option as the default selection.
    #[serde(default)]
    pub default: bool,
}

/// The [`MessageComponentInteraction::message`] field.
#[derive(Clone, Debug, Deserialize)]
pub enum InteractionMessage {
    Regular(Message),
    Ephemeral(EphemeralMessage),
}

impl InteractionMessage {
    /// Whether the message is ephemeral.
    pub fn is_ephemeral(&self) -> bool {
        matches!(self, InteractionMessage::Ephemeral(_))
    }

    /// Gets the message Id.
    pub fn id(&self) -> MessageId {
        match self {
            InteractionMessage::Regular(m) => m.id,
            InteractionMessage::Ephemeral(m) => m.id,
        }
    }

    /// Converts this to a regular message,
    /// if it is one.
    pub fn regular(self) -> Option<Message> {
        match self {
            InteractionMessage::Regular(m) => Some(m),
            InteractionMessage::Ephemeral(_) => None,
        }
    }

    /// Converts this to an ephemeral message,
    /// if it is one.
    pub fn ephemeral(self) -> Option<EphemeralMessage> {
        match self {
            InteractionMessage::Regular(_) => None,
            InteractionMessage::Ephemeral(m) => Some(m),
        }
    }
}

impl Serialize for InteractionMessage {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            InteractionMessage::Regular(c) => Message::serialize(c, serializer),
            InteractionMessage::Ephemeral(c) => EphemeralMessage::serialize(c, serializer),
        }
    }
}

/// An ephemeral message given in an interaction.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EphemeralMessage {
    /// The message flags.
    pub flags: MessageFlags,
    /// The message Id.
    pub id: MessageId,
}
