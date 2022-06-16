#[cfg(not(feature = "model"))]
use std::marker::PhantomData;

use super::{CreateAllowedMentions, CreateEmbed};
use crate::builder::CreateComponents;
#[cfg(feature = "model")]
use crate::model::channel::AttachmentType;
use crate::model::channel::{MessageFlags, MessageReference, ReactionType};
use crate::model::id::StickerId;

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
/// use serenity::model::id::ChannelId;
/// # use serenity::http::Http;
/// # use std::sync::Arc;
/// #
/// # let http = Arc::new(Http::new("token"));
///
/// let channel_id = ChannelId(7);
///
/// let _ = channel_id.send_message(&http, |m| {
///     m.content("test")
///         .tts(true)
///         .embed(|e| e.title("This is an embed").description("With a description"))
/// });
/// ```
///
/// [`ChannelId::say`]: crate::model::id::ChannelId::say
/// [`ChannelId::send_message`]: crate::model::id::ChannelId::send_message
/// [`Http::send_message`]: crate::http::client::Http::send_message
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateMessage<'a> {
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

    // Following fields are not sent to discord, and are
    // instead handled seperately.
    #[serde(skip)]
    #[cfg(feature = "model")]
    pub(crate) files: Vec<AttachmentType<'a>>,
    #[cfg(not(feature = "model"))]
    pub(crate) files: PhantomData<&'a ()>,

    #[serde(skip)]
    pub(crate) reactions: Vec<ReactionType>,
}

impl<'a> CreateMessage<'a> {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = Some(content.into());
        self
    }

    fn _add_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        self.embeds.push(embed);
        self
    }

    /// Add an embed for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::set_embed()`] to replace existing
    /// embeds.
    pub fn add_embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self._add_embed(embed)
    }

    /// Add multiple embeds for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::set_embeds()`] to replace existing
    /// embeds.
    pub fn add_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        self.embeds.extend(embeds);
        self
    }

    /// Set an embed for the message.
    ///
    /// Equivalent to [`Self::set_embed()`].
    ///
    /// **Note**: This will replace all existing embeds. Use
    /// [`Self::add_embed()`] to add an additional embed.
    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);

        self.set_embed(embed)
    }

    /// Set an embed for the message.
    ///
    /// Equivalent to [`Self::embed()`].
    ///
    /// **Note**: This will replace all existing embeds.
    /// Use [`Self::add_embed()`] to add an additional embed.
    pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        self.set_embeds(vec![embed])
    }

    /// Set multiple embeds for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embeds()`] to keep existing
    /// embeds.
    pub fn set_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        self.embeds = embeds;
        self
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.tts = tts;
        self
    }

    /// Adds a list of reactions to create after the message's sent.
    #[inline]
    pub fn reactions<R: Into<ReactionType>, It: IntoIterator<Item = R>>(
        &mut self,
        reactions: It,
    ) -> &mut Self {
        self.reactions = reactions.into_iter().map(Into::into).collect();
        self
    }

    /// Appends a file to the message.
    #[cfg(feature = "model")]
    pub fn add_file<T: Into<AttachmentType<'a>>>(&mut self, file: T) -> &mut Self {
        self.files.push(file.into());
        self
    }

    /// Appends a list of files to the message.
    #[cfg(feature = "model")]
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.files.extend(files.into_iter().map(Into::into));
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call [`Self::add_file`] or [`Self::add_files`] instead.
    #[cfg(feature = "model")]
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.files = files.into_iter().map(Into::into).collect();
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
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
    pub fn reference_message(&mut self, reference: impl Into<MessageReference>) -> &mut Self {
        self.message_reference = Some(reference.into());
        self
    }

    /// Creates components for this message.
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.set_components(components)
    }

    /// Sets the components of this message.
    pub fn set_components(&mut self, components: CreateComponents) -> &mut Self {
        self.components = Some(components);
        self
    }

    /// Sets the flags for the message.
    pub fn flags(&mut self, flags: MessageFlags) -> &mut Self {
        self.flags = Some(flags);
        self
    }

    /// Sets a single sticker ID to include in the message.
    ///
    /// **Note**: This will replace all existing stickers. Use
    /// [`Self::add_sticker_id()`] to add an additional sticker.
    pub fn sticker_id(&mut self, sticker_id: impl Into<StickerId>) -> &mut Self {
        self.set_sticker_ids(vec![sticker_id.into()])
    }

    /// Add a sticker ID for the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will keep all existing stickers. Use
    /// [`Self::set_sticker_ids()`] to replace existing stickers.
    pub fn add_sticker_id(&mut self, sticker_id: impl Into<StickerId>) -> &mut Self {
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
        &mut self,
        sticker_ids: It,
    ) -> &mut Self {
        for sticker_id in sticker_ids {
            self.add_sticker_id(sticker_id);
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
        &mut self,
        sticker_ids: It,
    ) -> &mut Self {
        self.sticker_ids = sticker_ids.into_iter().map(Into::into).collect();
        self
    }
}
