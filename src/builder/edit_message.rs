use std::borrow::Cow;

#[cfg(feature = "http")]
use super::Builder;
use super::{
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateEmbed,
    EditAttachments,
};
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to specify the fields to edit in an existing message.
///
/// # Examples
///
/// Editing the content of a [`Message`] to `"hello"`:
///
/// ```rust,no_run
/// # use serenity::builder::EditMessage;
/// # use serenity::model::channel::Message;
/// # use serenity::model::id::ChannelId;
/// # use serenity::http::CacheHttp;
///
/// # async fn example(ctx: impl CacheHttp, mut message: Message) -> Result<(), Box<dyn std::error::Error>> {
/// let builder = EditMessage::new().content("hello");
/// message.edit(ctx, builder).await?;
/// # Ok(())
/// # }
/// ```
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#edit-message)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditMessage<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Cow<'a, [CreateEmbed<'a>]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Cow<'a, [CreateActionRow<'a>]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<EditAttachments<'a>>,
}

impl<'a> EditMessage<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "http")]
    fn check_length(&self) -> Result<(), ModelError> {
        super::check_lengths(self.content.as_deref(), self.embeds.as_deref(), 0)
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    pub fn content(mut self, content: impl Into<Cow<'a, str>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Add an embed for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::embed()`] to replace existing
    /// embeds.
    pub fn add_embed(mut self, embed: CreateEmbed<'a>) -> Self {
        self.embeds.get_or_insert_with(Cow::default).to_mut().push(embed);
        self
    }

    /// Add multiple embeds for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::embeds()`] to replace existing
    /// embeds.
    pub fn add_embeds(mut self, embeds: impl IntoIterator<Item = CreateEmbed<'a>>) -> Self {
        self.embeds.get_or_insert_with(Cow::default).to_mut().extend(embeds);
        self
    }

    /// Set an embed for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embed()`] to keep existing
    /// embeds.
    pub fn embed(self, embed: CreateEmbed<'a>) -> Self {
        self.embeds(vec![embed])
    }

    /// Set multiple embeds for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embeds()`] to keep existing
    /// embeds.
    pub fn embeds(mut self, embeds: impl Into<Cow<'a, [CreateEmbed<'a>]>>) -> Self {
        self.embeds = Some(embeds.into());
        self
    }

    /// Suppress or unsuppress embeds in the message, this includes those generated by Discord
    /// themselves.
    ///
    /// If this is sent directly after posting the message, there is a small chance Discord hasn't
    /// yet fully parsed the contained links and generated the embeds, so this embed suppression
    /// request has no effect. To mitigate this, you can defer the embed suppression until the
    /// embeds have loaded:
    ///
    /// ```rust,no_run
    /// # use serenity::all::*;
    /// # #[cfg(feature = "collector")]
    /// # async fn test(ctx: &Context, channel_id: ChannelId) -> Result<(), Error> {
    /// use std::time::Duration;
    ///
    /// use futures::StreamExt;
    ///
    /// let mut msg = channel_id.say(ctx, "<link that spawns an embed>").await?;
    ///
    /// // When the embed appears, a MessageUpdate event is sent and we suppress the embed.
    /// // No MessageUpdate event is sent if the message contains no embeddable link or if the link
    /// // has been posted before and is still cached in Discord's servers (in which case the
    /// // embed appears immediately), no MessageUpdate event is sent. To not wait forever in those
    /// // cases, a timeout of 2000ms was added.
    /// let msg_id = msg.id;
    /// let mut message_updates = serenity::collector::collect(&ctx.shard, move |ev| match ev {
    ///     Event::MessageUpdate(x) if x.id == msg_id => Some(()),
    ///     _ => None,
    /// });
    /// let _ = tokio::time::timeout(Duration::from_millis(2000), message_updates.next()).await;
    /// msg.edit(&ctx, EditMessage::new().suppress_embeds(true)).await?;
    /// # Ok(()) }
    /// ```
    pub fn suppress_embeds(mut self, suppress: bool) -> Self {
        // At time of writing, only `SUPPRESS_EMBEDS` can be set/unset when editing messages. See
        // for details: https://discord.com/developers/docs/resources/channel#edit-message-jsonform-params
        let flags =
            suppress.then_some(MessageFlags::SUPPRESS_EMBEDS).unwrap_or_else(MessageFlags::empty);

        self.flags = Some(flags);
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions<'a>) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Sets the components of this message.
    pub fn components(mut self, components: impl Into<Cow<'a, [CreateActionRow<'a>]>>) -> Self {
        self.components = Some(components.into());
        self
    }
    super::button_and_select_menu_convenience_methods!(self.components);

    /// Sets the flags for the message.
    pub fn flags(mut self, flags: MessageFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Sets attachments, see [`EditAttachments`] for more details.
    pub fn attachments(mut self, attachments: EditAttachments<'a>) -> Self {
        self.attachments = Some(attachments);
        self
    }

    /// Adds a new attachment to the message.
    ///
    /// Resets existing attachments. See the documentation for [`EditAttachments`] for details.
    pub fn new_attachment(mut self, attachment: CreateAttachment<'a>) -> Self {
        let attachments = self.attachments.get_or_insert_with(Default::default);
        self.attachments = Some(std::mem::take(attachments).add(attachment));
        self
    }

    /// Shorthand for [`EditAttachments::keep`].
    pub fn keep_existing_attachment(mut self, id: AttachmentId) -> Self {
        let attachments = self.attachments.get_or_insert_with(Default::default);
        self.attachments = Some(std::mem::take(attachments).keep(id));
        self
    }

    /// Shorthand for [`EditAttachments::remove`].
    pub fn remove_existing_attachment(mut self, id: AttachmentId) -> Self {
        if let Some(attachments) = self.attachments {
            self.attachments = Some(attachments.remove(id));
        }
        self
    }

    /// Shorthand for calling [`Self::attachments`] with [`EditAttachments::new`].
    pub fn remove_all_attachments(mut self) -> Self {
        self.attachments = Some(EditAttachments::new());
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for EditMessage<'_> {
    type Context<'ctx> = (ChannelId, MessageId);
    type Built = Message;

    /// Edits a message in the channel.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 code points.
    ///
    /// **Note**: Requires that the current user be the author of the message. Other users can only
    /// call [`Self::suppress_embeds`], but additionally require the [Manage Messages] permission
    /// to do so.
    ///
    /// **Note**: If any embeds or attachments are set, they will overwrite the existing contents
    /// of the message, deleting existing embeds and attachments. Preserving them requires calling
    /// [`Self::keep_existing_attachment`] in the case of attachments. In the case of embeds,
    /// duplicate copies of the existing embeds must be sent. Luckily, [`CreateEmbed`] implements
    /// [`From<Embed>`], so one can simply call `embed.into()`.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::TooLarge`] if the message contents are over the above limits.
    ///
    /// Returns [`Error::Http`] if the user lacks permission, as well as if invalid data is given.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    /// [`From<Embed>`]: CreateEmbed#impl-From<Embed>
    async fn execute(
        mut self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        self.check_length()?;

        let files = self.attachments.as_mut().map_or(Vec::new(), |a| a.take_files());

        cache_http.http().edit_message(ctx.0, ctx.1, &self, files).await
    }
}
