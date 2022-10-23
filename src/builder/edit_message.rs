use super::{
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateEmbed,
    ExistingAttachment,
};
#[cfg(feature = "http")]
use crate::constants;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;
#[cfg(feature = "http")]
use crate::utils::check_overflow;

/// A builder to specify the fields to edit in an existing message.
///
/// # Examples
///
/// Editing the content of a [`Message`] to `"hello"`:
///
/// ```rust,no_run
/// # use serenity::builder::EditMessage;
/// # use serenity::model::id::{ChannelId, MessageId};
/// # #[cfg(feature = "client")]
/// # use serenity::client::Context;
/// # #[cfg(feature = "framework")]
/// # use serenity::framework::standard::{CommandResult, macros::command};
/// #
/// # #[cfg(all(feature = "model", feature = "utils", feature = "framework"))]
/// # #[command]
/// # async fn example(ctx: &Context) -> CommandResult {
/// # let mut message = ChannelId::new(7).message(&ctx, MessageId::new(8)).await?;
/// let builder = EditMessage::new().content("hello");
/// message.edit(ctx, builder).await?;
/// # Ok(())
/// # }
/// ```
///
/// [`Message`]: crate::model::channel::Message
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<CreateEmbed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<CreateActionRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<ExistingAttachment>>,

    #[serde(skip)]
    files: Vec<CreateAttachment>,
}

impl EditMessage {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Edits a message in the channel.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 code points.
    ///
    /// **Note**: Requires that the current user be the author of the message. Other users can
    /// only call [`Self::suppress_embeds`], but additionally require the [Manage Messages]
    /// permission to do so.
    ///
    /// **Note**: If any embeds or attachments are set, they will overwrite the existing contents
    /// of the message, deleting existing embeds and attachments. Preserving them requires calling
    /// [`Self::add_existing_attachment`] in the case of attachments. In the case of embeds,
    /// duplicate copies of the existing embeds must be sent. Luckily, [`CreateEmbed`] implements
    /// [`From<Embed>`], so one can simply call `embed.into()`.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the message contents are over the above limits.
    ///
    /// Returns [`Error::Http`] if the user lacks permission, as well as if invalid data is given.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    /// [`From<Embed>`]: CreateEmbed#impl-From<Embed>
    #[cfg(feature = "http")]
    pub async fn execute(
        mut self,
        http: impl AsRef<Http>,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<Message> {
        self.check_length()?;
        let files = std::mem::take(&mut self.files);
        http.as_ref().edit_message(channel_id, message_id, &self, files).await
    }

    #[cfg(feature = "http")]
    fn check_length(&self) -> Result<()> {
        if let Some(content) = &self.content {
            check_overflow(content.chars().count(), constants::MESSAGE_CODE_LIMIT)
                .map_err(|overflow| Error::Model(ModelError::MessageTooLong(overflow)))?;
        }

        if let Some(embeds) = &self.embeds {
            check_overflow(embeds.len(), constants::EMBED_MAX_COUNT)
                .map_err(|_| Error::Model(ModelError::EmbedAmount))?;
            for embed in embeds {
                embed.check_length()?;
            }
        }

        Ok(())
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Add an embed for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::embed()`] to replace existing
    /// embeds.
    pub fn add_embed(mut self, embed: CreateEmbed) -> Self {
        self.embeds.get_or_insert_with(Vec::new).push(embed);
        self
    }

    /// Add multiple embeds for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::embeds()`] to replace existing
    /// embeds.
    pub fn add_embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds.get_or_insert_with(Vec::new).extend(embeds);
        self
    }

    /// Set an embed for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embed()`] to keep existing
    /// embeds.
    pub fn embed(self, embed: CreateEmbed) -> Self {
        self.embeds(vec![embed])
    }

    /// Set multiple embeds for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embeds()`] to keep existing
    /// embeds.
    pub fn embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = Some(embeds);
        self
    }

    /// Suppress or unsuppress embeds in the message, this includes those generated by Discord
    /// themselves.
    pub fn suppress_embeds(mut self, suppress: bool) -> Self {
        // At time of writing, only `SUPPRESS_EMBEDS` can be set/unset when editing messages. See
        // for details: https://discord.com/developers/docs/resources/channel#edit-message-jsonform-params
        let flags =
            suppress.then_some(MessageFlags::SUPPRESS_EMBEDS).unwrap_or_else(MessageFlags::empty);

        self.flags = Some(flags);
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Sets the components of this message.
    pub fn components(mut self, components: Vec<CreateActionRow>) -> Self {
        self.components = Some(components);
        self
    }
    super::button_and_select_menu_convenience_methods!();

    /// Sets the flags for the message.
    pub fn flags(mut self, flags: MessageFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Add a new attachment for the message.
    ///
    /// This can be called multiple times.
    pub fn attachment(mut self, attachment: CreateAttachment) -> Self {
        self.files.push(attachment);
        self
    }

    /// Add an existing attachment by id.
    pub fn add_existing_attachment(mut self, id: AttachmentId) -> Self {
        self.attachments.get_or_insert_with(Vec::new).push(ExistingAttachment {
            id,
        });
        self
    }

    /// Remove an existing attachment by id.
    pub fn remove_existing_attachment(mut self, id: AttachmentId) -> Self {
        if let Some(attachments) = &mut self.attachments {
            if let Some(attachment_index) = attachments.iter().position(|a| a.id == id) {
                attachments.remove(attachment_index);
            };
        }
        self
    }
}
