use std::borrow::Cow;

#[cfg(feature = "http")]
use super::Builder;
use super::{
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateEmbed,
    EditAttachments,
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
pub struct EditInteractionResponse<'a>(EditWebhookMessage<'a>);

impl<'a> EditInteractionResponse<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(self, content: impl Into<Cow<'a, str>>) -> Self {
        Self(self.0.content(content))
    }

    /// Adds an embed for the message.
    ///
    /// Embeds from the original message are reset when adding new embeds and must be re-added.
    pub fn add_embed(self, embed: CreateEmbed<'a>) -> Self {
        Self(self.0.add_embed(embed))
    }

    /// Adds multiple embeds to the message.
    ///
    /// Embeds from the original message are reset when adding new embeds and must be re-added.
    pub fn add_embeds(self, embeds: impl IntoIterator<Item = CreateEmbed<'a>>) -> Self {
        Self(self.0.add_embeds(embeds))
    }

    /// Sets a single embed to include in the message
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embed`]
    /// instead.
    pub fn embed(self, embed: CreateEmbed<'a>) -> Self {
        Self(self.0.embed(embed))
    }

    /// Sets the embeds for the message.
    ///
    /// **Note**: You can only have up to 10 embeds per message.
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embeds`]
    /// instead.
    pub fn embeds(self, embeds: impl Into<Cow<'a, [CreateEmbed<'a>]>>) -> Self {
        Self(self.0.embeds(embeds))
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(self, allowed_mentions: CreateAllowedMentions<'a>) -> Self {
        Self(self.0.allowed_mentions(allowed_mentions))
    }

    /// Sets the components of this message.
    pub fn components(self, components: impl Into<Cow<'a, [CreateActionRow<'a>]>>) -> Self {
        Self(self.0.components(components))
    }
    super::button_and_select_menu_convenience_methods!(self.0.components);

    /// Sets attachments, see [`EditAttachments`] for more details.
    pub fn attachments(self, attachments: EditAttachments<'a>) -> Self {
        Self(self.0.attachments(attachments))
    }

    /// Adds a new attachment to the message.
    ///
    /// Resets existing attachments. See the documentation for [`EditAttachments`] for details.
    pub fn new_attachment(self, attachment: CreateAttachment<'a>) -> Self {
        Self(self.0.new_attachment(attachment))
    }

    /// Shorthand for [`EditAttachments::keep`].
    pub fn keep_existing_attachment(self, id: AttachmentId) -> Self {
        Self(self.0.keep_existing_attachment(id))
    }

    /// Shorthand for calling [`Self::attachments`] with [`EditAttachments::new`].
    pub fn clear_attachments(self) -> Self {
        Self(self.0.clear_attachments())
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for EditInteractionResponse<'_> {
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

        let files = self.0.attachments.as_mut().map_or(Vec::new(), |a| a.take_files());

        cache_http.http().edit_original_interaction_response(ctx, &self, files).await
    }
}
