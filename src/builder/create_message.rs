use std::collections::HashMap;
#[cfg(not(feature = "model"))]
use std::marker::PhantomData;

use super::{CreateAllowedMentions, CreateEmbed};
use crate::builder::CreateComponents;
use crate::internal::prelude::*;
use crate::json::{self, from_number, to_value};
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
#[derive(Clone, Debug)]
pub struct CreateMessage<'a>(
    pub HashMap<&'static str, Value>,
    pub Option<Vec<ReactionType>>,
    #[cfg(feature = "model")] pub Vec<AttachmentType<'a>>,
    #[cfg(not(feature = "model"))] PhantomData<&'a ()>,
);

impl<'a> CreateMessage<'a> {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content<D: ToString>(&mut self, content: D) -> &mut Self {
        self._content(content.to_string())
    }

    fn _content(&mut self, content: String) -> &mut Self {
        self.0.insert("content", Value::from(content));
        self
    }

    fn _add_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        let map = json::hashmap_to_json_map(embed.0);
        let embed = Value::from(map);

        let embeds = self.0.entry("embeds").or_insert_with(|| Value::from(Vec::<Value>::new()));
        let embeds_array = embeds.as_array_mut().expect("Embeds must be an array");

        embeds_array.push(embed);

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
        for embed in embeds {
            self._add_embed(embed);
        }

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
        self.0.insert("embeds", Value::from(Vec::<Value>::new()));
        self._add_embed(embed)
    }

    /// Set an embed for the message.
    ///
    /// Equivalent to [`Self::embed()`].
    ///
    /// **Note**: This will replace all existing embeds.
    /// Use [`Self::add_embed()`] to add an additional embed.
    pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        self.0.insert("embeds", Value::from(Vec::<Value>::new()));
        self._add_embed(embed)
    }

    /// Set multiple embeds for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embeds()`] to keep existing
    /// embeds.
    pub fn set_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        self.0.insert("embeds", Value::from(Vec::<Value>::new()));
        for embed in embeds {
            self._add_embed(embed);
        }

        self
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.0.insert("tts", Value::from(tts));
        self
    }

    /// Adds a list of reactions to create after the message's sent.
    #[inline]
    pub fn reactions<R: Into<ReactionType>, It: IntoIterator<Item = R>>(
        &mut self,
        reactions: It,
    ) -> &mut Self {
        self._reactions(reactions.into_iter().map(Into::into).collect());
        self
    }

    fn _reactions(&mut self, reactions: Vec<ReactionType>) {
        self.1 = Some(reactions);
    }

    /// Appends a file to the message.
    #[cfg(feature = "model")]
    pub fn add_file<T: Into<AttachmentType<'a>>>(&mut self, file: T) -> &mut Self {
        self.2.push(file.into());
        self
    }

    /// Appends a list of files to the message.
    #[cfg(feature = "model")]
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.2.extend(files.into_iter().map(Into::into));
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
        self.2 = files.into_iter().map(Into::into).collect();
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);
        let map = json::hashmap_to_json_map(allowed_mentions.0);
        let allowed_mentions = Value::from(map);

        self.0.insert("allowed_mentions", allowed_mentions);
        self
    }

    /// Set the reference message this message is a reply to.
    #[allow(clippy::unwrap_used)] // allowing unwrap here because serializing MessageReference should never error
    pub fn reference_message(&mut self, reference: impl Into<MessageReference>) -> &mut Self {
        self.0.insert("message_reference", to_value(reference.into()).unwrap());
        self
    }

    /// Creates components for this message.
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.0.insert("components", Value::from(components.0));
        self
    }

    /// Sets the components of this message.
    pub fn set_components(&mut self, components: CreateComponents) -> &mut Self {
        self.0.insert("components", Value::from(components.0));
        self
    }

    /// Sets the flags for the message.
    pub fn flags(&mut self, flags: MessageFlags) -> &mut Self {
        self.0.insert("flags", from_number(flags.bits()));
        self
    }

    /// Sets a single sticker ID to include in the message.
    ///
    /// **Note**: This will replace all existing stickers. Use
    /// [`Self::add_sticker_id()`] to add an additional sticker.
    pub fn sticker_id(&mut self, sticker_id: impl Into<StickerId>) -> &mut Self {
        self.0.insert("sticker_ids", Value::from(Vec::<Value>::new()));
        self.add_sticker_id(sticker_id)
    }

    /// Add a sticker ID for the message.
    ///
    /// **Note**: There can be a maximum of 3 stickers in a message.
    ///
    /// **Note**: This will keep all existing stickers. Use
    /// [`Self::set_sticker_ids()`] to replace existing stickers.
    pub fn add_sticker_id(&mut self, sticker_id: impl Into<StickerId>) -> &mut Self {
        let sticker_ids =
            self.0.entry("sticker_ids").or_insert_with(|| Value::from(Vec::<Value>::new()));
        let sticker_ids_array = sticker_ids.as_array_mut().expect("Sticker_ids must be an array");

        sticker_ids_array.push(Value::from(sticker_id.into().0));

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
        self.0.insert("sticker_ids", Value::from(Vec::<Value>::new()));
        self.add_sticker_ids(sticker_ids)
    }
}

impl<'a> Default for CreateMessage<'a> {
    /// Creates a map for sending a [`Message`], setting [`Self::tts`] to `false` by
    /// default.
    ///
    /// [`Message`]: crate::model::channel::Message
    fn default() -> CreateMessage<'a> {
        let mut map = HashMap::new();
        map.insert("tts", Value::from(false));

        // Necessary because the type of the third field is different without model feature
        #[allow(clippy::default_trait_access)]
        CreateMessage(map, None, Default::default())
    }
}
