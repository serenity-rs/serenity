use std::collections::HashMap;

use super::CreateAllowedMentions;
use super::CreateEmbed;
use crate::http::AttachmentType;
use crate::internal::prelude::*;
use crate::json::to_value;
use crate::model::channel::{MessageReference, ReactionType};
use crate::utils;

/// A builder to specify the contents of an [`Http::send_message`] request,
/// primarily meant for use through [`ChannelId::send_message`].
///
/// There are two situations where different field requirements are present:
///
/// 1. When sending an [`embed`], no other field is required;
/// 2. Otherwise, [`content`] is the only required field that is required to be
/// set.
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
/// # let http = Arc::new(Http::default());
///
/// let channel_id = ChannelId(7);
///
/// let _ = channel_id.send_message(&http, |m| {
///     m.content("test");
///     m.tts(true);
///
///     m.embed(|mut e| {
///         e.title("This is an embed");
///         e.description("With a description");
///
///         e
///     });
///
///     m
/// });
/// ```
///
/// [`ChannelId::say`]: crate::model::id::ChannelId::say
/// [`ChannelId::send_message`]: crate::model::id::ChannelId::send_message
/// [`content`]: Self::content
/// [`embed`]: Self::embed
/// [`Http::send_message`]: crate::http::client::Http::send_message
#[derive(Clone, Debug)]
pub struct CreateMessage<'a>(
    pub HashMap<&'static str, Value>,
    pub Option<Vec<ReactionType>>,
    pub Vec<AttachmentType<'a>>,
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
        self.0.insert("content", Value::String(content));
        self
    }

    /// Create an embed for the message.
    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self.set_embed(embed)
    }

    /// Set an embed for the message.
    pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        let map = utils::hashmap_to_json_map(embed.0);
        let embed = Value::from(map);

        self.0.insert("embed", embed);
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
    pub fn add_file<T: Into<AttachmentType<'a>>>(&mut self, file: T) -> &mut Self {
        self.2.push(file.into());
        self
    }

    /// Appends a list of files to the message.
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.2.extend(files.into_iter().map(|f| f.into()));
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call `add_file` or `add_files` instead.
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.2 = files.into_iter().map(|f| f.into()).collect();
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);
        let map = utils::hashmap_to_json_map(allowed_mentions.0);
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
}

impl<'a> Default for CreateMessage<'a> {
    /// Creates a map for sending a [`Message`], setting [`tts`] to `false` by
    /// default.
    ///
    /// [`Message`]: crate::model::channel::Message
    /// [`tts`]: Self::tts
    fn default() -> CreateMessage<'a> {
        let mut map = HashMap::new();
        map.insert("tts", Value::from(false));

        CreateMessage(map, None, Vec::new())
    }
}
