use super::CreateEmbed;
use ::model::ReactionType;
use ::internal::prelude::*;

/// A builder to specify the contents of an [`http::send_message`] request,
/// primarily meant for use through [`Context::send_message`].
///
/// There are two situations where different field requirements are present:
///
/// 1. When sending an [`embed`], no other field is required;
/// 2. Otherwise, [`content`] is the only required field that is required to be
/// set.
///
/// Note that if you only need to send the content of a message, without
/// specifying other fields, then [`Context::say`] may be a more preferable
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
/// [`Context::say`]: ../client/struct.Context.html#method.say
/// [`Context::send_message`]: ../client/struct.Context.html#method.send_message
/// [`content`]: #method.content
/// [`embed`]: #method.embed
/// [`http::send_message`]: ../http/fn.send_message.html
#[derive(Clone, Debug)]
pub struct CreateMessage(pub Map<String, Value>, pub Option<Vec<ReactionType>>);

impl CreateMessage {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    pub fn content(mut self, content: &str) -> Self {
        self.0.insert("content".to_owned(), Value::String(content.to_owned()));

        CreateMessage(self.0, self.1)
    }

    /// Set an embed for the message.
    pub fn embed<F>(mut self, f: F) -> Self
        where F: FnOnce(CreateEmbed) -> CreateEmbed {
        let embed = Value::Object(f(CreateEmbed::default()).0);

        self.0.insert("embed".to_owned(), embed);

        CreateMessage(self.0, self.1)
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(mut self, tts: bool) -> Self {
        self.0.insert("tts".to_owned(), Value::Bool(tts));

        CreateMessage(self.0, self.1)
    }

    /// Adds a list of reactions to create after the message's sent.
    pub fn reactions<R: Into<ReactionType>>(mut self, reactions: Vec<R>) -> Self {
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
        map.insert("tts".to_owned(), Value::Bool(false));

        CreateMessage(map, None)
    }
}
