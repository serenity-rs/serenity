#[cfg(feature = "http")]
use super::Builder;
use super::{
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateEmbed,
    EditWebhookMessage,
};
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#edit-original-interaction-response)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditInteractionResponse(EditWebhookMessage);

impl EditInteractionResponse {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(self, content: impl Into<String>) -> Self {
        Self(self.0.content(content))
    }

    /// Adds an embed for the message.
    ///
    /// Embeds from the original message are reset when adding new embeds and must be re-added.
    pub fn add_embed(self, embed: CreateEmbed) -> Self {
        Self(self.0.add_embed(embed))
    }

    /// Adds multiple embeds to the message.
    ///
    /// Embeds from the original message are reset when adding new embeds and must be re-added.
    pub fn add_embeds(self, embeds: Vec<CreateEmbed>) -> Self {
        Self(self.0.add_embeds(embeds))
    }

    /// Sets a single embed to include in the message
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embed`]
    /// instead.
    pub fn embed(self, embed: CreateEmbed) -> Self {
        Self(self.0.embed(embed))
    }

    /// Sets the embeds for the message.
    ///
    /// **Note**: You can only have up to 10 embeds per message.
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embeds`]
    /// instead.
    pub fn embeds(self, embeds: Vec<CreateEmbed>) -> Self {
        Self(self.0.embeds(embeds))
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(self, allowed_mentions: CreateAllowedMentions) -> Self {
        Self(self.0.allowed_mentions(allowed_mentions))
    }

    /// Sets the components of this message.
    pub fn components(self, components: Vec<CreateActionRow>) -> Self {
        Self(self.0.components(components))
    }
    super::button_and_select_menu_convenience_methods!(self.0.components);

    /// Add a new attachment for the message.
    ///
    /// This can be called multiple times.
    ///
    /// If this is called one or more times, existing attachments will reset. To keep them, provide
    /// their IDs to [`Self::keep_existing_attachment`].
    pub fn new_attachment(self, attachment: CreateAttachment) -> Self {
        Self(self.0.new_attachment(attachment))
    }

    /// Keeps an existing attachment by id.
    ///
    /// To be used after [`Self::new_attachment`] or [`Self::clear_existing_attachments`].
    pub fn keep_existing_attachment(self, id: AttachmentId) -> Self {
        Self(self.0.keep_existing_attachment(id))
    }

    /// Clears existing attachments.
    ///
    /// In combination with [`Self::keep_existing_attachment`], this can be used to selectively
    /// keep only some existing attachments.
    pub fn clear_existing_attachments(self) -> Self {
        Self(self.0.clear_existing_attachments())
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for EditInteractionResponse {
    type Context<'ctx> = &'ctx str;
    type Built = Message;

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
    async fn execute(
        mut self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        self.0.check_length()?;
        let files = std::mem::take(&mut self.0.files);
        cache_http.http().edit_original_interaction_response(ctx, &self, files).await
    }
}
