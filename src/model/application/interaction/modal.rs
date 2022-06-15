use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::Serialize;

#[cfg(feature = "model")]
use crate::builder::{
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    EditInteractionResponse,
};
#[cfg(feature = "model")]
use crate::http::Http;
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::json;
use crate::model::application::component::ActionRow;
#[cfg(feature = "http")]
use crate::model::application::interaction::InteractionResponseType;
use crate::model::channel::Message;
use crate::model::guild::Member;
#[cfg(feature = "http")]
use crate::model::id::MessageId;
use crate::model::id::{ApplicationId, ChannelId, GuildId, InteractionId};
use crate::model::user::User;
use crate::model::utils::{remove_from_map, remove_from_map_opt};

/// An interaction triggered by a modal submit.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct ModalSubmitInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The data of the interaction which was triggered.
    pub data: ModalSubmitInteractionData,
    /// The message this interaction was triggered by
    /// **Note**: Does not exist if the modal interaction originates from
    /// an application command interaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
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

#[cfg(feature = "model")]
impl ModalSubmitInteraction {
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

        let http = http.as_ref();
        let files = interaction_response
            .data
            .as_mut()
            .map_or_else(Vec::new, |d| std::mem::take(&mut d.files));

        if files.is_empty() {
            http.create_interaction_response(self.id.0, &self.token, &interaction_response).await
        } else {
            http.create_interaction_response_with_files(
                self.id.0,
                &self.token,
                &interaction_response,
                files,
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
    /// **Note**:   Message contents must be under 2000 unicode code points.
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

        http.as_ref().create_followup_message(&self.token, &interaction_response).await
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

        http.as_ref()
            .edit_followup_message(&self.token, message_id.into().into(), &interaction_response)
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

impl<'de> Deserialize<'de> for ModalSubmitInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let member = remove_from_map_opt::<Member, _>(&mut map, "member")?;
        let user = remove_from_map_opt(&mut map, "user")?
            .or_else(|| member.as_ref().map(|m| m.user.clone()))
            .ok_or_else(|| DeError::custom("expected user or member"))?;

        Ok(Self {
            member,
            user,
            id: remove_from_map(&mut map, "id")?,
            guild_id: remove_from_map_opt(&mut map, "guild_id")?,
            application_id: remove_from_map(&mut map, "application_id")?,
            data: remove_from_map(&mut map, "data")?,
            message: remove_from_map(&mut map, "message")?,
            channel_id: remove_from_map(&mut map, "channel_id")?,
            token: remove_from_map(&mut map, "token")?,
            version: remove_from_map(&mut map, "version")?,
            guild_locale: remove_from_map_opt(&mut map, "guild_locale")?,
            locale: remove_from_map(&mut map, "locale")?,
        })
    }
}

/// A modal submit interaction data, provided by [`ModalSubmitInteraction::data`]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ModalSubmitInteractionData {
    /// The custom id of the modal
    pub custom_id: String,
    /// The components.
    pub components: Vec<ActionRow>,
}
