use super::{CreateAllowedMentions, CreateComponents, CreateEmbed};
#[cfg(feature = "http")]
use crate::constants;
#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to specify the contents of an [`Http::send_message`] request,
/// primarily meant for use through [`ChannelId::send_message`].
///
/// There are three situations where different field requirements are present:
///
/// 1. When sending a message without embeds or stickers, [`Self::content`] is
///    the only required field that is required to be set.
/// 2. When sending an [`Self::embed`], no other field is required.
/// 3. When sending stickers with [`Self::sticker_id`] or other sticker methods,
///    no other field is required.
///
/// Note that if you only need to send the content of a message, without
/// specifying other fields, then [`ChannelId::say`] may be a more preferable
/// option.
///
/// # Examples
///
/// Sending a message with a content of `"test"` and applying text-to-speech:
///
/// ```rust,no_run
/// use serenity::builder::CreateEmbed;
/// use serenity::model::id::ChannelId;
/// # use serenity::http::Http;
/// # use std::sync::Arc;
/// #
/// # pub async fn run() {
/// # let http = Arc::new(Http::new("token"));
///
/// let channel_id = ChannelId::new(7);
///
/// let embed = CreateEmbed::default().title("This is an embed").description("With a description");
/// let _ = channel_id.send_message().content("test").tts(true).embed(embed).execute(&http).await;
/// # }
/// ```
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateMessage<'a> {
    #[cfg(feature = "http")]
    #[serde(skip)]
    channel_id: ChannelId,
    #[cfg(all(feature = "http", feature = "cache"))]
    #[serde(skip)]
    guild_id: Option<GuildId>,

    tts: bool,
    embeds: Vec<CreateEmbed>,
    sticker_ids: Vec<StickerId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_reference: Option<MessageReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<CreateComponents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,

    // The following fields are handled separately.
    #[serde(skip)]
    files: Vec<AttachmentType<'a>>,
    #[serde(skip)]
    reactions: Vec<ReactionType>,
}

impl<'a> CreateMessage<'a> {
    pub fn new(
        #[cfg(feature = "http")] channel_id: ChannelId,
        #[cfg(all(feature = "http", feature = "cache"))] guild_id: Option<GuildId>,
    ) -> Self {
        Self {
            #[cfg(feature = "http")]
            channel_id,
            #[cfg(all(feature = "http", feature = "cache"))]
            guild_id,

            tts: false,
            embeds: Vec::new(),
            sticker_ids: Vec::new(),
            content: None,
            allowed_mentions: None,
            message_reference: None,
            components: None,
            flags: None,

            files: Vec::new(),
            reactions: Vec::new(),
        }
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
        self.embeds.push(embed);
        self
    }

    /// Add multiple embeds for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::embeds()`] to replace existing
    /// embeds.
    pub fn add_embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds.extend(embeds);
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
        self.embeds = embeds;
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
    #[inline]
    pub fn reactions<R: Into<ReactionType>, It: IntoIterator<Item = R>>(
        mut self,
        reactions: It,
    ) -> Self {
        self.reactions = reactions.into_iter().map(Into::into).collect();
        self
    }

    /// Appends a file to the message.
    ///
    /// **Note**: Requres the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    pub fn add_file<T: Into<AttachmentType<'a>>>(mut self, file: T) -> Self {
        self.files.push(file.into());
        self
    }

    /// Appends a list of files to the message.
    ///
    /// **Note**: Requres the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        mut self,
        files: It,
    ) -> Self {
        self.files.extend(files.into_iter().map(Into::into));
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call [`Self::add_file`] or [`Self::add_files`] instead.
    ///
    /// **Note**: Requres the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        mut self,
        files: It,
    ) -> Self {
        self.files = files.into_iter().map(Into::into).collect();
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);

        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Set the reference message this message is a reply to.
    #[allow(clippy::unwrap_used)] // allowing unwrap here because serializing MessageReference should never error
    pub fn reference_message(mut self, reference: impl Into<MessageReference>) -> Self {
        self.message_reference = Some(reference.into());
        self
    }

    /// Creates components for this message.
    pub fn components<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.set_components(components)
    }

    /// Sets the components of this message.
    pub fn set_components(mut self, components: CreateComponents) -> Self {
        self.components = Some(components);
        self
    }

    /// Sets the flags for the message.
    pub fn flags(mut self, flags: MessageFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Sets a single sticker ID to include in the message.
    ///
    /// **Note**: This will replace all existing stickers. Use
    /// [`Self::add_sticker_id()`] to add an additional sticker.
    pub fn sticker_id(self, sticker_id: impl Into<StickerId>) -> Self {
        self.set_sticker_ids(vec![sticker_id.into()])
    }

    /// Add a sticker ID for the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will keep all existing stickers. Use
    /// [`Self::set_sticker_ids()`] to replace existing stickers.
    pub fn add_sticker_id(mut self, sticker_id: impl Into<StickerId>) -> Self {
        self.sticker_ids.push(sticker_id.into());
        self
    }

    /// Add multiple sticker IDs for the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will keep all existing stickers. Use
    /// [`Self::set_sticker_ids()`] to replace existing stickers.
    pub fn add_sticker_ids<T: Into<StickerId>, It: IntoIterator<Item = T>>(
        mut self,
        sticker_ids: It,
    ) -> Self {
        for sticker_id in sticker_ids {
            self = self.add_sticker_id(sticker_id);
        }
        self
    }

    /// Sets a list of sticker IDs to include in the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will replace all existing stickers. Use
    /// [`Self::add_sticker_id()`] or [`Self::add_sticker_ids()`] to keep
    /// existing stickers.
    pub fn set_sticker_ids<T: Into<StickerId>, It: IntoIterator<Item = T>>(
        mut self,
        sticker_ids: It,
    ) -> Self {
        self.sticker_ids = sticker_ids.into_iter().map(Into::into).collect();
        self
    }

    /// Sends a message to the channel.
    ///
    /// **Note**: Requires the [Send Messages] permission. Adding files additionally requires the
    /// [Attach Files] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message is over the above
    /// limit, containing the number of unicode code points over the limit.
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have the required permissions.
    ///
    /// Otherwise, returns [`Error::Http`] if the current user lacks permission, as well as if any
    /// files sent are too large channel, or otherwise if any invalid data is sent.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    /// [Attach Files]: Permissions::ATTACH_FILES
    #[cfg(feature = "http")]
    pub async fn execute(self, cache_http: impl CacheHttp) -> Result<Message> {
        #[cfg(feature = "cache")]
        {
            let mut req = Permissions::SEND_MESSAGES;
            if !self.files.is_empty() {
                req |= Permissions::ATTACH_FILES;
            }
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(cache, self.channel_id, self.guild_id, req)?;
            }
        }

        self._execute(cache_http.http()).await
    }

    #[cfg(feature = "http")]
    async fn _execute(mut self, http: &Http) -> Result<Message> {
        self.check_lengths()?;
        let files = std::mem::take(&mut self.files);

        let message = if files.is_empty() {
            http.send_message(self.channel_id.into(), &self).await?
        } else {
            http.send_files(self.channel_id.into(), files, &self).await?
        };

        for reaction in self.reactions {
            self.channel_id.create_reaction(&http, message.id, reaction).await?;
        }

        Ok(message)
    }

    #[cfg(feature = "http")]
    pub(crate) fn check_lengths(&self) -> Result<()> {
        if let Some(ref content) = self.content {
            let length = content.chars().count();
            let max_length = constants::MESSAGE_CODE_LIMIT;
            if length > max_length {
                let overflow = length - max_length;
                return Err(Error::Model(ModelError::MessageTooLong(overflow)));
            }
        }

        if self.embeds.len() > constants::EMBED_MAX_COUNT {
            return Err(Error::Model(ModelError::EmbedAmount));
        }
        for embed in &self.embeds {
            embed.check_length()?;
        }

        if self.sticker_ids.len() > constants::STICKER_MAX_COUNT {
            return Err(Error::Model(ModelError::StickerAmount));
        }

        Ok(())
    }
}
