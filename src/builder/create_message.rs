use internal::prelude::*;
use http::AttachmentType;
use model::channel::ReactionType;
use std::fmt::Display;
use super::CreateEmbed;
use utils::{self, VecMap};

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
///
/// let channel_id = ChannelId(7);
///
/// let _ = channel_id.send_message(|mut m| {
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
pub struct CreateMessage<'a>(pub VecMap<&'static str, Value>, pub Option<Vec<ReactionType>>, pub Vec<AttachmentType<'a>>);

impl<'a> CreateMessage<'a> {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    pub fn content<D: Display>(&mut self, content: D) {
        self.0.insert("content", Value::String(content.to_string()));
    }

    /// Set an embed for the message.
    pub fn embed<F: FnOnce(CreateEmbed) -> CreateEmbed>(&mut self, f: F) {
        let map = utils::vecmap_to_json_map(f(CreateEmbed::default()).0);
        let embed = Value::Object(map);

        self.0.insert("embed", embed);
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(&mut self, tts: bool) {
        self.0.insert("tts", Value::Bool(tts));
    }

    /// Adds a list of reactions to create after the message's sent.
    pub fn reactions<R: Into<ReactionType>, It: IntoIterator<Item=R>>(&mut self, reactions: It) {
        self.1 = Some(reactions.into_iter().map(|r| r.into()).collect());
    }

    /// Appends a file to the message.
    pub fn add_file<T: Into<AttachmentType<'a>>>(&mut self, file: T) {
        self.2.push(file.into());
    }

    /// Appends a list of files to the message.
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item=T>>(&mut self, files: It) {
        self.2.extend(files.into_iter().map(|f| f.into()));
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call `add_file` or `add_files` instead.
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item=T>>(&mut self, files: It) {
        self.2 = files.into_iter().map(|f| f.into()).collect();
    }
}

impl<'a> Default for CreateMessage<'a> {
    /// Creates a map for sending a [`Message`], setting [`tts`] to `false` by
    /// default.
    ///
    /// [`Message`]: ../model/channel/struct.Message.html
    /// [`tts`]: #method.tts
    fn default() -> CreateMessage<'a> {
        let mut map = VecMap::new();
        map.insert("tts", Value::Bool(false));

        CreateMessage(map, None, Vec::new())
    }
}
