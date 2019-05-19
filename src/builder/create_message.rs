use crate::internal::prelude::*;
use crate::http::AttachmentType;
use crate::model::channel::ReactionType;
use super::CreateEmbed;
use crate::utils;

use std::collections::HashMap;

/// A builder to specify the contents of an [`http::send_message`] request,
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
/// [`ChannelId::say`]: ../model/id/struct.ChannelId.html#method.say
/// [`ChannelId::send_message`]: ../model/id/struct.ChannelId.html#method.send_message
/// [`content`]: #method.content
/// [`embed`]: #method.embed
/// [`http::send_message`]: ../http/fn.send_message.html
#[derive(Clone, Debug)]
pub struct CreateMessage<'a>(pub HashMap<&'static str, Value>, pub Option<Vec<ReactionType>>, pub Vec<AttachmentType<'a>>);

impl<'a> CreateMessage<'a> {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content<D: ToString>(&mut self, content: D) -> &mut Self {
        self.0.insert("content", Value::String(content.to_string()));
        self
    }

    fn _content(&mut self, content: String) {
        self.0.insert("content", Value::String(content));
    }

    /// Set an embed for the message.
    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        let map = utils::hashmap_to_json_map(embed.0);
        let embed = Value::Object(map);

        self.0.insert("embed", embed);
        self
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.0.insert("tts", Value::Bool(tts));
        self
    }

    /// Adds a list of reactions to create after the message's sent.
    #[inline]
    pub fn reactions<R: Into<ReactionType>, It: IntoIterator<Item=R>>(&mut self, reactions: It) -> &mut Self {
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
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item=T>>(&mut self, files: It) -> &mut Self {
        self.2.extend(files.into_iter().map(|f| f.into()));
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call `add_file` or `add_files` instead.
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item=T>>(&mut self, files: It) -> &mut Self {
        self.2 = files.into_iter().map(|f| f.into()).collect();
        self
    }
}

impl<'a> Default for CreateMessage<'a> {
    /// Creates a map for sending a [`Message`], setting [`tts`] to `false` by
    /// default.
    ///
    /// [`Message`]: ../model/channel/struct.Message.html
    /// [`tts`]: #method.tts
    fn default() -> CreateMessage<'a> {
        let mut map = HashMap::new();
        map.insert("tts", Value::Bool(false));

        CreateMessage(map, None, Vec::new())
    }
}
