use super::{CreateActionRow, CreateAllowedMentions, CreateAttachment, CreateEmbed};
#[cfg(feature = "http")]
use crate::constants;
#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;
#[cfg(feature = "http")]
use crate::utils::check_overflow;

/// A builder to specify the contents of an [`Http::send_message`] request, primarily meant for use
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
/// # let http = Arc::new(Http::new("token"));
///
/// let channel_id = ChannelId::new(7);
///
/// let embed = CreateEmbed::new().title("This is an embed").description("With a description");
/// let builder = CreateMessage::new().content("test").tts(true).embed(embed);
/// let _ = channel_id.send_message(&http, builder).await;
/// # }
/// ```
///
/// [`ChannelId::say`]: crate::model::id::ChannelId::say
/// [`ChannelId::send_message`]: crate::model::id::ChannelId::send_message
/// [`Http::send_message`]: crate::http::client::Http::send_message
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateMessage {
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
    components: Option<Vec<CreateActionRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,

    // The following fields are handled separately.
    #[serde(skip)]
    files: Vec<CreateAttachment>,
    #[serde(skip)]
    reactions: Vec<ReactionType>,
}

impl CreateMessage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Send a message to the channel.
    ///
    /// **Note**: Requires the [Send Messages] permission. Additionally, attaching files requires the
    /// [Attach Files] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the message contents are over the above limits.
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Send Messages]: Permissions::SEND_MESSAGES
    /// [Attach Files]: Permissions::ATTACH_FILES
    #[cfg(feature = "http")]
    #[inline]
    pub async fn execute(
        self,
        cache_http: impl CacheHttp,
        channel_id: ChannelId,
        #[cfg(feature = "cache")] guild_id: Option<GuildId>,
    ) -> Result<Message> {
        #[cfg(feature = "cache")]
        {
            let mut req = Permissions::SEND_MESSAGES;
            if !self.files.is_empty() {
                req |= Permissions::ATTACH_FILES;
            }
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(cache, channel_id, guild_id, req)?;
            }
        }

        #[cfg_attr(not(feature = "cache"), allow(unused_mut))]
        let mut message = self._execute(cache_http.http(), channel_id).await?;

        // HTTP sent Messages don't have guild_id set, so we fill it in ourselves by best effort
        #[cfg(feature = "cache")]
        if message.guild_id.is_none() {
            // Use either the passed in guild ID (e.g. if we were called from GuildChannel directly
            // we already know our guild ID), and otherwise find the guild ID in cache
            message.guild_id = guild_id
                .or_else(|| Some(cache_http.cache()?.guild_channel(message.channel_id)?.guild_id));
        }

        Ok(message)
    }

    #[cfg(feature = "http")]
    async fn _execute(mut self, http: &Http, channel_id: ChannelId) -> Result<Message> {
        self.check_length()?;
        let files = std::mem::take(&mut self.files);
        let message = http.send_message(channel_id, files, &self).await?;

        for reaction in self.reactions {
            channel_id.create_reaction(http, message.id, reaction).await?;
        }

        Ok(message)
    }

    #[cfg(feature = "http")]
    fn check_length(&self) -> Result<()> {
        if let Some(content) = &self.content {
            check_overflow(content.chars().count(), constants::MESSAGE_CODE_LIMIT)
                .map_err(|overflow| Error::Model(ModelError::MessageTooLong(overflow)))?;
        }

        check_overflow(self.embeds.len(), constants::EMBED_MAX_COUNT)
            .map_err(|_| Error::Model(ModelError::EmbedAmount))?;
        for embed in &self.embeds {
            embed.check_length()?;
        }

        check_overflow(self.sticker_ids.len(), constants::STICKER_MAX_COUNT)
            .map_err(|_| Error::Model(ModelError::StickerAmount))?;

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
    /// **Note**: Requires the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    pub fn add_file(mut self, file: CreateAttachment) -> Self {
        self.files.push(file);
        self
    }

    /// Appends a list of files to the message.
    ///
    /// **Note**: Requires the [Attach Files] permission.
    ///
    /// [Attach Files]: Permissions::ATTACH_FILES
    pub fn add_files(mut self, files: impl IntoIterator<Item = CreateAttachment>) -> Self {
        self.files.extend(files);
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
    pub fn files(mut self, files: impl IntoIterator<Item = CreateAttachment>) -> Self {
        self.files = files.into_iter().collect();
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Set the reference message this message is a reply to.
    #[allow(clippy::unwrap_used)] // allowing unwrap here because serializing MessageReference should never error
    pub fn reference_message(mut self, reference: impl Into<MessageReference>) -> Self {
        self.message_reference = Some(reference.into());
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

    /// Sets a single sticker ID to include in the message.
    ///
    /// **Note**: This will replace all existing stickers. Use [`Self::add_sticker_id()`] to keep
    /// existing stickers.
    pub fn sticker_id(self, sticker_id: impl Into<StickerId>) -> Self {
        self.sticker_ids(vec![sticker_id.into()])
    }

    /// Sets a list of sticker IDs to include in the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will replace all existing stickers. Use [`Self::add_sticker_id()`] or
    /// [`Self::add_sticker_ids()`] to keep existing stickers.
    pub fn sticker_ids<T: Into<StickerId>, It: IntoIterator<Item = T>>(
        mut self,
        sticker_ids: It,
    ) -> Self {
        self.sticker_ids = sticker_ids.into_iter().map(Into::into).collect();
        self
    }

    /// Add a sticker ID for the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will keep all existing stickers. Use [`Self::sticker_id()`] to replace
    /// existing sticker.
    pub fn add_sticker_id(mut self, sticker_id: impl Into<StickerId>) -> Self {
        self.sticker_ids.push(sticker_id.into());
        self
    }

    /// Add multiple sticker IDs for the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will keep all existing stickers. Use [`Self::sticker_ids()`] to replace
    /// existing stickers.
    pub fn add_sticker_ids<T: Into<StickerId>, It: IntoIterator<Item = T>>(
        mut self,
        sticker_ids: It,
    ) -> Self {
        for sticker_id in sticker_ids {
            self = self.add_sticker_id(sticker_id);
        }
        self
    }
}
