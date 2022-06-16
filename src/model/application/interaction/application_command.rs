use std::collections::HashMap;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};

#[cfg(feature = "http")]
use crate::builder::{
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    EditInteractionResponse,
};
#[cfg(feature = "http")]
use crate::http::Http;
use crate::internal::prelude::*;
use crate::model::application::command::{CommandOptionType, CommandType};
use crate::model::application::interaction::add_guild_id_to_resolved;
#[cfg(feature = "http")]
use crate::model::application::interaction::InteractionResponseType;
use crate::model::channel::{Attachment, Message, PartialChannel};
use crate::model::guild::{Member, PartialMember, Role};
use crate::model::id::{
    ApplicationId,
    AttachmentId,
    ChannelId,
    CommandId,
    GuildId,
    InteractionId,
    MessageId,
    RoleId,
    TargetId,
    UserId,
};
use crate::model::user::User;
use crate::model::utils::{
    deserialize_options_with_resolved,
    remove_from_map,
    remove_from_map_opt,
};

/// An interaction when a user invokes a slash command.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object).
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct ApplicationCommandInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The data of the interaction which was triggered.
    pub data: CommandData,
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
    /// The selected language of the invoking user.
    pub locale: String,
    /// The guild's preferred locale.
    pub guild_locale: Option<String>,
}

#[cfg(feature = "http")]
impl ApplicationCommandInteraction {
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

impl<'de> Deserialize<'de> for ApplicationCommandInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let guild_id = remove_from_map_opt::<GuildId, _>(&mut map, "guild_id")?;

        if let Some(guild_id) = guild_id {
            add_guild_id_to_resolved(&mut map, guild_id);
        }

        let member = remove_from_map_opt::<Member, _>(&mut map, "member")?;
        let user = remove_from_map_opt(&mut map, "user")?
            .or_else(|| member.as_ref().map(|m| m.user.clone()))
            .ok_or_else(|| DeError::custom("expected user or member"))?;

        Ok(Self {
            guild_id,
            member,
            user,
            id: remove_from_map(&mut map, "id")?,
            application_id: remove_from_map(&mut map, "application_id")?,
            data: remove_from_map(&mut map, "data")?,
            channel_id: remove_from_map(&mut map, "channel_id")?,
            token: remove_from_map(&mut map, "token")?,
            version: remove_from_map(&mut map, "version")?,
            locale: remove_from_map(&mut map, "locale")?,
            guild_locale: remove_from_map_opt(&mut map, "guild_locale")?,
        })
    }
}

/// The command data payload.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data-structure).
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct CommandData {
    /// The Id of the invoked command.
    pub id: CommandId,
    /// The name of the invoked command.
    pub name: String,
    /// The application command type of the triggered application command.
    #[serde(rename = "type")]
    pub kind: CommandType,
    /// The parameters and the given values.
    /// The converted objects from the given options.
    #[serde(default)]
    pub resolved: CommandDataResolved,
    #[serde(default)]
    pub options: Vec<CommandDataOption>,
    /// The Id of the guild the command is registered to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// The targeted user or message, if the triggered application command type
    /// is [`User`] or [`Message`].
    ///
    /// Its object data can be found in the [`resolved`] field.
    ///
    /// [`resolved`]: Self::resolved
    /// [`User`]: CommandType::User
    /// [`Message`]: CommandType::Message
    pub target_id: Option<TargetId>,
}

impl CommandData {
    /// The target resolved data of [`target_id`]
    ///
    /// [`target_id`]: Self::target_id
    #[must_use]
    pub fn target(&self) -> Option<ResolvedTarget<'_>> {
        match (self.kind, self.target_id) {
            (CommandType::User, Some(id)) => {
                let user_id = id.to_user_id();

                let user = self.resolved.users.get(&user_id)?;
                let member = self.resolved.members.get(&user_id);

                Some(ResolvedTarget::User(user, member))
            },
            (CommandType::Message, Some(id)) => {
                let message_id = id.to_message_id();
                let message = self.resolved.messages.get(&message_id)?;

                Some(ResolvedTarget::Message(message))
            },
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for CommandData {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let resolved = remove_from_map_opt(&mut map, "resolved")?.unwrap_or_default();

        let options = map
            .remove("options")
            .map(|deserializer| deserialize_options_with_resolved(deserializer, &resolved))
            .transpose()
            .map_err(DeError::custom)?
            .unwrap_or_default();

        Ok(Self {
            options,
            resolved,
            id: remove_from_map(&mut map, "id")?,
            name: remove_from_map(&mut map, "name")?,
            kind: remove_from_map(&mut map, "type")?,
            guild_id: remove_from_map_opt(&mut map, "guild_id")?.flatten(),
            target_id: remove_from_map_opt(&mut map, "target_id")?.flatten(),
        })
    }
}

/// The resolved value of a [`CommandData::target_id`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ResolvedTarget<'a> {
    User(&'a User, Option<&'a PartialMember>),
    Message(&'a Message),
}

/// The resolved data of a command data interaction payload.
/// It contains the objects of [`CommandDataOption`]s.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-resolved-data-structure).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandDataResolved {
    /// The resolved users.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub users: HashMap<UserId, User>,
    /// The resolved partial members.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub members: HashMap<UserId, PartialMember>,
    /// The resolved roles.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub roles: HashMap<RoleId, Role>,
    /// The resolved partial channels.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub channels: HashMap<ChannelId, PartialChannel>,
    /// The resolved messages.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub messages: HashMap<MessageId, Message>,
    /// The resolved attachments.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attachments: HashMap<AttachmentId, Attachment>,
}

/// A set of a parameter and a value from the user.
///
/// All options have names and an option can either be a parameter and input `value` or it can denote a sub-command or group, in which case it will contain a
/// top-level key and another vector of `options`.
///
/// Their resolved objects can be found on [`CommandData::resolved`].
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-interaction-data-option-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandDataOption {
    /// The name of the parameter.
    pub name: String,
    /// The given value.
    pub value: Option<Value>,
    /// The value type.
    #[serde(rename = "type")]
    pub kind: CommandOptionType,
    /// The nested options.
    ///
    /// **Note**: It is only present if the option is
    /// a group or a subcommand.
    #[serde(default)]
    pub options: Vec<CommandDataOption>,
    /// The resolved object of the given `value`, if there is one.
    #[serde(default)]
    pub resolved: Option<CommandDataOptionValue>,
}

/// The resolved value of an [`CommandDataOption`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CommandDataOptionValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    User(User, Option<PartialMember>),
    Channel(PartialChannel),
    Role(Role),
    Number(f64),
    Attachment(Attachment),
}

impl TargetId {
    /// Converts this [`TargetId`] to [`UserId`].
    #[must_use]
    pub fn to_user_id(self) -> UserId {
        self.0.into()
    }

    /// Converts this [`TargetId`] to [`MessageId`].
    #[must_use]
    pub fn to_message_id(self) -> MessageId {
        self.0.into()
    }
}

impl From<MessageId> for TargetId {
    fn from(id: MessageId) -> Self {
        Self(id.0)
    }
}

impl<'a> From<&'a MessageId> for TargetId {
    fn from(id: &MessageId) -> Self {
        Self(id.0)
    }
}

impl From<UserId> for TargetId {
    fn from(id: UserId) -> Self {
        Self(id.0)
    }
}

impl<'a> From<&'a UserId> for TargetId {
    fn from(id: &UserId) -> Self {
        Self(id.0)
    }
}

impl From<TargetId> for MessageId {
    fn from(id: TargetId) -> Self {
        Self(id.0)
    }
}

impl From<TargetId> for UserId {
    fn from(id: TargetId) -> Self {
        Self(id.0)
    }
}
