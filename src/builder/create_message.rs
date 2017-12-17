use super::CreateEmbed;
use model::ReactionType;
use internal::prelude::*;
use std::fmt::Display;

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
/// use serenity::model::ChannelId;
///
/// let channel_id = ChannelId(7);
///
/// let _ = channel_id.send_message(|m| m
///     .content("test")
///     .tts(true)
///     .embed(|e| e
///         .title("This is an embed")
///         .description("With a description")));
/// ```
///
/// [`ChannelId::say`]: ../model/struct.ChannelId.html#method.say
/// [`ChannelId::send_message`]: ../model/struct.ChannelId.html#method.send_message
/// [`content`]: #method.content
/// [`embed`]: #method.embed
/// [`http::send_message`]: ../http/fn.send_message.html
#[derive(Clone, Debug)]
pub struct CreateMessage(pub Map<String, Value>, pub Option<Vec<ReactionType>>);

impl CreateMessage {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    pub fn content<D: Display>(mut self, content: D) -> Self {
        self.0
            .insert("content".to_string(), Value::String(format!("{}", content)));

        CreateMessage(self.0, self.1)
    }

    /// Set an embed for the message.
    pub fn embed<F>(mut self, f: F) -> Self
        where F: FnOnce(CreateEmbed) -> CreateEmbed {
        let embed = Value::Object(f(CreateEmbed::default()).0);

        self.0.insert("embed".to_string(), embed);

        CreateMessage(self.0, self.1)
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(mut self, tts: bool) -> Self {
        self.0.insert("tts".to_string(), Value::Bool(tts));

        CreateMessage(self.0, self.1)
    }

    /// Adds a list of reactions to create after the message's sent.
    pub fn reactions<R: Into<ReactionType>, It: IntoIterator<Item=R>>(mut self, reactions: It) -> Self {
        self.1 = Some(reactions.into_iter().map(|r| r.into()).collect());

        CreateMessage(self.0, self.1)
    }
}

impl Default for CreateMessage {
    /// Creates a map for sending a [`Message`], setting [`tts`] to `false` by
    /// default.
    ///
    /// [`Message`]: ../model/struct.Message.html
    /// [`tts`]: #method.tts
    fn default() -> CreateMessage {
        let mut map = Map::default();
        map.insert("tts".to_string(), Value::Bool(false));

        CreateMessage(map, None)
    }
}
