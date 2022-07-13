use super::{CreateAllowedMentions, CreateComponents, CreateEmbed};
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::prelude::*;

#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditInteractionResponse {
    embeds: Vec<CreateEmbed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<CreateComponents>,
}

impl EditInteractionResponse {
    /// Edits the initial interaction response. Does not work for ephemeral messages.
    ///
    /// The `application_id` used will usually be the bot's [`UserId`], except if the bot is very
    /// old.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long. May also return an
    /// [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is an error in
    /// deserializing the API response.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: impl AsRef<Http>, token: &str) -> Result<Message> {
        self.check_length()?;
        http.as_ref().edit_original_interaction_response(token, &self).await
    }

    #[cfg(feature = "http")]
    fn check_length(&self) -> Result<()> {
        if let Some(content) = &self.content {
            let length = content.chars().count();
            let max_length = crate::constants::MESSAGE_CODE_LIMIT;
            if length > max_length {
                return Err(Error::Model(ModelError::MessageTooLong(length - max_length)));
            }
        }

        if self.embeds.len() > crate::constants::EMBED_MAX_COUNT {
            return Err(Error::Model(ModelError::EmbedAmount));
        }
        for embed in &self.embeds {
            embed.check_length()?;
        }
        Ok(())
    }

    /// Sets the `InteractionApplicationCommandCallbackData` for the message.

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Adds an embed for the message.
    pub fn add_embed(mut self, embed: CreateEmbed) -> Self {
        self.embeds.push(embed);
        self
    }

    /// Adds multiple embeds to the message.
    pub fn add_embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds.extend(embeds);
        self
    }

    /// Sets a single embed to include in the message
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embed`]
    /// instead.
    pub fn embed(self, embed: CreateEmbed) -> Self {
        self.embeds(vec![embed])
    }

    /// Sets the embeds for the message.
    ///
    /// **Note**: You can only have up to 10 embeds per message.
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embeds`]
    /// instead.
    pub fn embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = embeds;
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Sets the components of this message.
    pub fn components<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.components = Some(components);
        self
    }
}
