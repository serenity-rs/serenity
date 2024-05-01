use std::borrow::Cow;

use super::create_poll::Ready;
use super::{
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateEmbed,
    CreatePoll,
    EditAttachments,
};
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to specify the contents of an send message request, primarily meant for use
/// through [`ChannelId::send_message`].
///
/// There are three situations where different field requirements are present:
///
/// 1. When sending a message without embeds or stickers, [`Self::content`] is the only required
///    field that is required to be set.
/// 2. When sending an [`Self::embed`], no other field is required.
/// 3. When sending stickers with [`Self::sticker_id`] or other sticker methods, no other field is
///    required.
///
/// Note that if you only need to send the content of a message, without specifying other fields,
/// then [`ChannelId::say`] may be a more preferable option.
///
/// # Examples
///
/// Sending a message with a content of `"test"` and applying text-to-speech:
///
/// ```rust,no_run
/// use serenity::builder::{CreateEmbed, CreateMessage};
/// use serenity::model::id::ChannelId;
/// # use serenity::http::Http;
/// # use std::sync::Arc;
/// #
/// # async fn run() {
/// # let http: Arc<Http> = unimplemented!();
/// # let channel_id = ChannelId::new(7);
/// let embed = CreateEmbed::new().title("This is an embed").description("With a description");
/// let builder = CreateMessage::new().content("test").tts(true).embed(embed);
/// let _ = channel_id.send_message(&http, builder).await;
/// # }
/// ```
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#create-message)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateMessage<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nonce: Option<Nonce>,
    tts: bool,
    embeds: Cow<'a, [CreateEmbed<'a>]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_reference: Option<MessageReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Cow<'a, [CreateActionRow<'a>]>>,
    sticker_ids: Cow<'a, [StickerId]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
    pub(crate) attachments: EditAttachments<'a>,
    enforce_nonce: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    poll: Option<CreatePoll<'a, Ready>>,

    // The following fields are handled separately.
    #[serde(skip)]
    reactions: Cow<'a, [ReactionType]>,
}

impl<'a> CreateMessage<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "http")]
    fn check_length(&self) -> Result<(), ModelError> {
        super::check_lengths(self.content.as_deref(), Some(&self.embeds), self.sticker_ids.len())
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
        self.embeds.to_mut().push(embed);
        self
    }

    /// Add multiple embeds for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::embeds()`] to replace existing
    /// embeds.
    pub fn add_embeds(mut self, embeds: impl IntoIterator<Item = CreateEmbed<'a>>) -> Self {
        self.embeds.to_mut().extend(embeds);
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
        self.embeds = embeds.into();
        self
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(mut self, tts: bool) -> Self {
        self.tts = tts;
        self
    }

    /// Adds a list of reactions to create after the message's sent.
    pub fn reactions(mut self, reactions: impl Into<Cow<'a, [ReactionType]>>) -> Self {
        self.reactions = reactions.into();
        self
    }

    /// Appends a file to the message.
    ///
    /// **Note**: Requires the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    pub fn add_file(mut self, file: CreateAttachment<'a>) -> Self {
        self.attachments = self.attachments.add(file);
        self
    }

    /// Appends a list of files to the message.
    ///
    /// **Note**: Requires the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    pub fn add_files(mut self, files: impl IntoIterator<Item = CreateAttachment<'a>>) -> Self {
        for file in files {
            self.attachments = self.attachments.add(file);
        }
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list. To append files, call
    /// [`Self::add_file`] or [`Self::add_files`] instead.
    ///
    /// **Note**: Requires the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    pub fn files(mut self, files: impl IntoIterator<Item = CreateAttachment<'a>>) -> Self {
        self.attachments = EditAttachments::new();
        self.add_files(files)
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions<'a>) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Set the message this reply or forward is referring to.
    pub fn reference_message(mut self, reference: impl Into<MessageReference>) -> Self {
        self.message_reference = Some(reference.into());
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

    /// Sets a single sticker ID to include in the message.
    ///
    /// **Note**: This will replace all existing stickers. Use [`Self::add_sticker_id()`] to keep
    /// existing stickers.
    pub fn sticker_id(self, sticker_id: StickerId) -> Self {
        self.sticker_ids(vec![sticker_id])
    }

    /// Sets a list of sticker IDs to include in the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will replace all existing stickers. Use [`Self::add_sticker_id()`] or
    /// [`Self::add_sticker_ids()`] to keep existing stickers.
    pub fn sticker_ids(mut self, sticker_ids: impl Into<Cow<'a, [StickerId]>>) -> Self {
        self.sticker_ids = sticker_ids.into();
        self
    }

    /// Add a sticker ID for the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will keep all existing stickers. Use [`Self::sticker_id()`] to replace
    /// existing sticker.
    pub fn add_sticker_id(mut self, sticker_id: StickerId) -> Self {
        self.sticker_ids.to_mut().push(sticker_id);
        self
    }

    /// Add multiple sticker IDs for the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will keep all existing stickers. Use [`Self::sticker_ids()`] to replace
    /// existing stickers.
    pub fn add_sticker_ids(mut self, sticker_ids: impl IntoIterator<Item = StickerId>) -> Self {
        self.sticker_ids.to_mut().extend(sticker_ids);
        self
    }

    /// Can be used to verify a message was sent (up to 25 characters). Value will appear in
    /// [`Message::nonce`]
    ///
    /// See [`Self::enforce_nonce`] if you would like discord to perform de-duplication.
    pub fn nonce(mut self, nonce: Nonce) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// If true and [`Self::nonce`] is provided, it will be checked for uniqueness in the past few
    /// minutes. If another message was created by the same author with the same nonce, that
    /// message will be returned and no new message will be created.
    pub fn enforce_nonce(mut self, enforce_nonce: bool) -> Self {
        self.enforce_nonce = enforce_nonce;
        self
    }

    /// Sets the [`Poll`] for this message.
    pub fn poll(mut self, poll: CreatePoll<'a, Ready>) -> Self {
        self.poll = Some(poll);
        self
    }

    /// Send a message to the channel.
    ///
    /// **Note**: Requires the [Send Messages] permission. Additionally, attaching files requires
    /// the [Attach Files] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::TooLarge`] if the message contents are over the above limits.
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    /// [Attach Files]: Permissions::ATTACH_FILES
    #[cfg(feature = "http")]
    pub async fn execute(
        mut self,
        http: &Http,
        channel_id: ChannelId,
        guild_id: Option<GuildId>,
    ) -> Result<Message> {
        self.check_length()?;

        let files = self.attachments.take_files();
        if self.allowed_mentions.is_none() {
            self.allowed_mentions.clone_from(&http.default_allowed_mentions);
        }

        let mut message = http.send_message(channel_id, files, &self).await?;

        for reaction in self.reactions.iter() {
            http.create_reaction(channel_id, message.id, reaction).await?;
        }

        // HTTP sent Messages don't have guild_id set, so we fill it in ourselves by best effort
        if message.guild_id.is_none() {
            // If we were called from GuildChannel, we can fill in the GuildId ourselves.
            message.guild_id = guild_id;
        }

        Ok(message)
    }
}
