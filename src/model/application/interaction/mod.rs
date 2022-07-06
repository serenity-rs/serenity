pub mod application_command;
pub mod message_component;
pub mod modal;
pub mod ping;

use std::fmt;

use serde::de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, MapAccess};
use serde_value::DeserializerError;

use self::application_command::CommandData;
use self::message_component::MessageComponentInteractionData;
use self::modal::ModalSubmitInteractionData;
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
use crate::model::guild::PartialMember;
#[cfg(feature = "model")]
use crate::model::id::MessageId;
use crate::model::id::{ApplicationId, ChannelId, GuildId, InteractionId};
use crate::model::user::User;
use crate::model::Permissions;

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object)
#[derive(Clone, Debug, Serialize)]
pub struct Interaction {
    /// ID of the interaction.
    pub id: InteractionId,
    /// ID of the application this interaction is for.
    pub application_id: ApplicationId,
    #[serde(rename = "type")]
    pub kind: InteractionType,
    pub data: Option<InteractionData>,
    /// The guild ID this interaction was sent from, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<ChannelId>,
    /// The `member` data for the invoking user.
    ///
    /// **Note**: It is only present if the interaction is triggered in a guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<PartialMember>,
    /// The `user` object for the invoking user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Always `1`.
    pub version: u8,
    /// The message this interaction was triggered by
    ///
    /// **Note**: Does not exist if the modal interaction originates from
    /// an application command interaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    /// Permissions the app or bot has within the channel the interaction was sent from.
    pub app_permissions: Option<Permissions>,
    /// The selected language of the invoking user.
    pub locale: String,
    /// The guild's preferred locale.
    pub guild_locale: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum InteractionData {
    ApplicationCommand(CommandData),
    Autocomplete(CommandData),
    MessageComponent(MessageComponentInteractionData),
    ModalSubmit(ModalSubmitInteractionData),
}

impl<'de> Deserialize<'de> for Interaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            ApplicationId,
            Type,
            Data,
            GuildId,
            ChannelId,
            Member,
            User,
            Token,
            Version,
            Message,
            AppPermissions,
            Locale,
            GuildLocale,
            Unknown(String),
        }

        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Interaction;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("expecting interaction object")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> StdResult<Self::Value, A::Error> {
                let mut id = None;
                let mut application_id = None;
                let mut kind = None;
                let mut data: Option<serde_value::Value> = None;
                let mut guild_id = None;
                let mut channel_id = None;
                let mut member = None;
                let mut user = None;
                let mut token = None;
                let mut version = None;
                let mut message = None;
                let mut app_permissions = None;
                let mut locale = None;
                let mut guild_locale = None;

                macro_rules! next_value {
                    ($field:ident, $name:literal) => {
                        if $field.is_some() {
                            return Err(DeError::duplicate_field($name));
                        }
                        $field = Some(map.next_value()?);
                    };
                }

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            next_value!(id, "id");
                        },
                        Field::ApplicationId => {
                            next_value!(application_id, "application_id");
                        },
                        Field::Type => {
                            next_value!(kind, "type");
                        },
                        Field::Data => {
                            next_value!(data, "data");
                        },
                        Field::GuildId => {
                            next_value!(guild_id, "guild_id");
                        },
                        Field::ChannelId => {
                            next_value!(channel_id, "channel_id");
                        },
                        Field::Member => {
                            next_value!(member, "member");
                        },
                        Field::User => {
                            next_value!(user, "user");
                        },
                        Field::Token => {
                            next_value!(token, "token");
                        },
                        Field::Version => {
                            next_value!(version, "version");
                        },
                        Field::Message => {
                            next_value!(message, "message");
                        },
                        Field::AppPermissions => {
                            next_value!(app_permissions, "app_permissions");
                        },
                        Field::Locale => {
                            next_value!(locale, "locale");
                        },
                        Field::GuildLocale => {
                            next_value!(guild_locale, "guild_locale");
                        },
                        Field::Unknown(_) => {
                            map.next_value::<IgnoredAny>()?;
                        },
                    }
                }

                let id = id.ok_or_else(|| DeError::missing_field("id"))?;
                let application_id =
                    application_id.ok_or_else(|| DeError::missing_field("application_id"))?;
                let kind = kind.ok_or_else(|| DeError::missing_field("type"))?;

                macro_rules! data {
                    () => {
                        data.ok_or_else(|| DeError::missing_field("data"))?
                            .deserialize_into()
                            .map_err(DeserializerError::into_error)?
                    };
                }
                let data = match kind {
                    InteractionType::ApplicationCommand => {
                        Some(InteractionData::ApplicationCommand(data!()))
                    },
                    InteractionType::Autocomplete => Some(InteractionData::Autocomplete(data!())),
                    InteractionType::MessageComponent => {
                        Some(InteractionData::MessageComponent(data!()))
                    },
                    InteractionType::ModalSubmit => Some(InteractionData::ModalSubmit(data!())),
                    _ => None,
                };

                let token = token.ok_or_else(|| DeError::missing_field("token"))?;
                let version = version.ok_or_else(|| DeError::missing_field("version"))?;
                let locale = locale.ok_or_else(|| DeError::missing_field("locale"))?;

                Ok(Self::Value {
                    id,
                    application_id,
                    kind,
                    data,
                    guild_id,
                    channel_id,
                    member,
                    user,
                    token,
                    version,
                    message,
                    app_permissions,
                    locale,
                    guild_locale,
                })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl Interaction {
    /// Gets the interaction Id.
    #[deprecated(note = "use the `id` field")]
    #[must_use]
    pub fn id(&self) -> InteractionId {
        self.id
    }

    /// Gets the interaction type
    #[deprecated(note = "use the `kind` field")]
    #[must_use]
    pub fn kind(&self) -> InteractionType {
        self.kind
    }

    /// Gets the interaction application Id
    #[deprecated(note = "use the `application_id` field")]
    #[must_use]
    pub fn application_id(&self) -> ApplicationId {
        self.application_id
    }

    /// Gets the interaction token.
    #[deprecated(note = "use the `token` field")]
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Gets the invoked guild locale.
    #[deprecated(note = "use the `guild_locale` field")]
    #[must_use]
    pub fn guild_locale(&self) -> Option<&str> {
        self.guild_locale.as_deref()
    }

    /// Gets a [`CommandData`] from this.
    #[must_use]
    pub fn application_command(&self) -> Option<&CommandData> {
        match &self.data {
            Some(InteractionData::ApplicationCommand(data)) => Some(data),
            _ => None,
        }
    }

    /// Gets a [`CommandData`] from this.
    #[must_use]
    pub fn autocomplete(&self) -> Option<&CommandData> {
        match &self.data {
            Some(InteractionData::ApplicationCommand(data)) => Some(data),
            _ => None,
        }
    }

    /// Gets a [`MessageComponentInteractionData`] from this.
    #[must_use]
    pub fn message_component(&self) -> Option<&MessageComponentInteractionData> {
        match &self.data {
            Some(InteractionData::MessageComponent(data)) => Some(data),
            _ => None,
        }
    }

    /// Gets a [`ModalSubmitInteractionData`] from this.
    #[must_use]
    pub fn modal_submit(&self) -> Option<&ModalSubmitInteractionData> {
        match &self.data {
            Some(InteractionData::ModalSubmit(data)) => Some(data),
            _ => None,
        }
    }
}

/// General interaction response methods.
#[cfg(feature = "model")]
impl Interaction {
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
        self._create_interaction_response(http.as_ref(), interaction_response).await
    }

    async fn _create_interaction_response<'a>(
        &self,
        http: &Http,
        mut interaction_response: CreateInteractionResponse<'a>,
    ) -> Result<()> {
        let files = interaction_response
            .data
            .as_mut()
            .map_or_else(Vec::new, |d| std::mem::take(&mut d.files));

        if files.is_empty() {
            http.create_interaction_response(self.id.get(), &self.token, &interaction_response)
                .await
        } else {
            http.create_interaction_response_with_files(
                self.id.get(),
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
    /// **Note**:   Message contents must be under 2000 unicode code points, does not work on ephemeral messages.
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

        http.as_ref().edit_original_interaction_response(&self.token, &interaction_response).await
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
        self._create_followup_message(http.as_ref(), interaction_response).await
    }

    async fn _create_followup_message<'a>(
        &self,
        http: &Http,
        mut interaction_response: CreateInteractionResponseFollowup<'a>,
    ) -> Result<Message> {
        let files = std::mem::take(&mut interaction_response.files);

        if files.is_empty() {
            http.create_followup_message(&self.token, &interaction_response).await
        } else {
            http.create_followup_message_with_files(&self.token, &interaction_response, files).await
        }
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
        let mut builder = CreateInteractionResponseFollowup::default();
        f(&mut builder);

        let http = http.as_ref();
        let message_id = message_id.into().into();
        let files = std::mem::take(&mut builder.files);

        if files.is_empty() {
            http.edit_followup_message(&self.token, message_id, &builder).await
        } else {
            http.edit_followup_message_and_attachments(&self.token, message_id, &builder, files)
                .await
        }
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
    /// [`Error::Http`]: crate::error::Error::Http
    /// [`Error::Json`]: crate::error::Error::Json
    pub async fn defer(&self, http: impl AsRef<Http>) -> Result<()> {
        self.create_interaction_response(http, |f| {
            f.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    }
}

/// Autocomplete interaction response methods.
#[cfg(feature = "model")]
impl Interaction {
    /// Creates a response to an autocomplete interaction.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    pub async fn create_autocomplete_response<F>(&self, http: impl AsRef<Http>, f: F) -> Result<()>
    where
        F: FnOnce(&mut CreateAutocompleteResponse) -> &mut CreateAutocompleteResponse,
    {
        #[derive(Serialize)]
        struct AutocompleteResponse {
            data: CreateAutocompleteResponse,
            #[serde(rename = "type")]
            kind: InteractionResponseType,
        }

        let mut response = CreateAutocompleteResponse::default();
        f(&mut response);

        let map = AutocompleteResponse {
            data: response,
            kind: InteractionResponseType::Autocomplete,
        };

        http.as_ref().create_interaction_response(self.id.get(), &self.token, &map).await
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
        ApplicationCommand = 2,
        MessageComponent = 3,
        Autocomplete = 4,
        ModalSubmit = 5,
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

/// The available responses types for an interaction response.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-interaction-callback-type).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
