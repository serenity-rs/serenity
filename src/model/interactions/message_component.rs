use serde::de::Error as DeError;
use serde::Serialize;

use super::prelude::*;
#[cfg(feature = "http")]
use crate::builder::{
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    EditInteractionResponse,
};
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::json;
use crate::json::prelude::*;
use crate::model::interactions::InteractionType;

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
    pub message: Message,
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
    /// The guild's preferred locale.
    pub guild_locale: Option<String>,
    /// The selected language of the invoking user.
    pub locale: String,
}

#[cfg(feature = "http")]
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
    /// [`Error::Model`]: crate::error::Error::Model
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn create_interaction_response<'a, F>(
        &self,
        http: impl AsRef<Http>,
        f: F,
    ) -> Result<()>
    where
        for<'b> F:
            FnOnce(&'b mut CreateInteractionResponse<'a>) -> &'b mut CreateInteractionResponse<'a>,
    {
        let mut interaction_response = CreateInteractionResponse::default();
        f(&mut interaction_response);

        let map = json::hashmap_to_json_map(interaction_response.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        if interaction_response.1.is_empty() {
            http.as_ref()
                .create_interaction_response(self.id.0, &self.token, &Value::from(map))
                .await
        } else {
            http.as_ref()
                .create_interaction_response_with_files(
                    self.id.0,
                    &self.token,
                    &Value::from(map),
                    interaction_response.1,
                )
                .await
        }
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

        let map = json::hashmap_to_json_map(interaction_response.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        http.as_ref().edit_original_interaction_response(&self.token, &Value::from(map)).await
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

        let map = json::hashmap_to_json_map(interaction_response.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        http.as_ref().create_followup_message(&self.token, &Value::from(map)).await
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

        let map = json::hashmap_to_json_map(interaction_response.0);

        Message::check_content_length(&map)?;
        Message::check_embed_length(&map)?;

        http.as_ref()
            .edit_followup_message(&self.token, message_id.into().into(), &Value::from(map))
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

    /// Gets a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error.
    /// Such as if the response was deleted.
    pub async fn get_followup_message<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<Message> {
        http.as_ref().get_followup_message(&self.token, message_id.into().into()).await
    }

    /// Helper function to defer an interaction
    ///
    /// # Errors
    ///
    /// May also return an [`Error::Http`] if the API returns an error,
    /// or an [`Error::Json`] if there is an error in deserializing the
    /// API response.
    ///
    /// # Errors
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn defer(&self, http: impl AsRef<Http>) -> Result<()> {
        self.create_interaction_response(http, |f| {
            f.kind(InteractionResponseType::DeferredUpdateMessage)
        })
        .await
    }
}

impl<'de> Deserialize<'de> for MessageComponentInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("guild_id").and_then(Value::as_str).and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(member) = map.get_mut("member").and_then(Value::as_object_mut) {
                member.insert("guild_id".to_string(), from_number(guild_id));
            }

            if let Some(data) = map.get_mut("data") {
                if let Some(resolved) = data.get_mut("resolved") {
                    if let Some(roles) = resolved.get_mut("roles") {
                        if let Some(values) = roles.as_object_mut() {
                            for value in values.values_mut() {
                                value
                                    .as_object_mut()
                                    .expect("couldn't deserialize message component")
                                    .insert(
                                        "guild_id".to_string(),
                                        Value::from(guild_id.to_string()),
                                    );
                            }
                        }
                    }

                    if let Some(channels) = resolved.get_mut("channels") {
                        if let Some(values) = channels.as_object_mut() {
                            for value in values.values_mut() {
                                value
                                    .as_object_mut()
                                    .expect(
                                        "couldn't deserialize the message component interaction",
                                    )
                                    .insert(
                                        "guild_id".to_string(),
                                        Value::from(guild_id.to_string()),
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

        let guild_id = map
            .remove("guild_id")
            .map(GuildId::deserialize)
            .transpose()
            .map_err(DeError::custom)?;

        let channel_id = map
            .remove("channel_id")
            .ok_or_else(|| DeError::custom("expected channel_id"))
            .and_then(ChannelId::deserialize)
            .map_err(DeError::custom)?;

        let member =
            map.remove("member").map(Member::deserialize).transpose().map_err(DeError::custom)?;

        let user =
            map.remove("user").map(User::deserialize).transpose().map_err(DeError::custom)?;

        let user = user
            .or_else(|| member.as_ref().map(|m| m.user.clone()))
            .ok_or_else(|| DeError::custom("expected user or member"))?;

        let message = map
            .remove("message")
            .ok_or_else(|| DeError::custom("expected message"))
            .and_then(Message::deserialize)
            .map_err(DeError::custom)?;

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

        let guild_locale = map
            .remove("guild_locale")
            .map(String::deserialize)
            .transpose()
            .map_err(DeError::custom)?;

        let locale = map
            .remove("locale")
            .ok_or_else(|| DeError::custom("expected locale"))
            .and_then(String::deserialize)
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
            guild_locale,
            locale,
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

use crate::model::application::component;

/// The type of a component
#[deprecated(note = "use `model::application::component::ComponentType`")]
pub type ComponentType = component::ComponentType;

/// An action row.
#[deprecated(note = "use `model::application::component::ActionRow`")]
pub type ActionRow = component::ActionRow;

// A component which can be inside of an [`ActionRow`].
#[deprecated(note = "use `model::application::component::ActionRowComponent`")]
pub type ActionRowComponent = component::ActionRowComponent;

/// A button component.
#[deprecated(note = "use `model::application::component::Button`")]
pub type Button = component::Button;

/// The style of a button.
#[deprecated(note = "use `model::application::component::ButtonStyle`")]
pub type ButtonStyle = component::ButtonStyle;

/// A select menu component.
#[deprecated(note = "use `model::application::component::SelectMenu`")]
pub type SelectMenu = component::SelectMenu;

/// A select menu component options.
#[deprecated(note = "use `model::application::component::SelectMenuOption`")]
pub type SelectMenuOption = component::SelectMenuOption;

/// An input text component for modal interactions
#[deprecated(note = "use `model::application::component::InputText`")]
pub type InputText = component::InputText;

/// The style of the input text
#[deprecated(note = "use `model::application::component::InputTextStyle`")]
pub type InputTextStyle = component::InputTextStyle;
