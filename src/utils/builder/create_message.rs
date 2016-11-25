use serde_json::Value;
use std::collections::BTreeMap;
use std::default::Default;
use super::CreateEmbed;

/// A builder to specify the contents of an [`rest::create_message`] request,
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
/// ```rust,ignore
/// // assuming you are in a context
/// context.send_message(message.channel_id, |m|
///     .content("test")
///     .tts(true));
/// ```
///
/// [`Context::say`]: ../../client/struct.Context.html#method.say
/// [`Context::send_message`]: ../../client/struct.Context.html#method.send_message
/// [`content`]: #method.content
/// [`embed`]: #method.embed
/// [`rest::create_message`]: ../../client/rest/fn.create_message.html
pub struct CreateMessage(pub BTreeMap<String, Value>);

impl CreateMessage {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    pub fn content(mut self, content: &str) -> Self {
        self.0.insert("content".to_owned(), Value::String(content.to_owned()));

        CreateMessage(self.0)
    }

    /// Set an embed for the message.
    pub fn embed<F>(mut self, f: F) -> Self
        where F: FnOnce(CreateEmbed) -> CreateEmbed {
        let embed = Value::Object(f(CreateEmbed::default()).0);

        self.0.insert("embed".to_owned(), embed);

        CreateMessage(self.0)
    }

    /// Set the nonce. This is used for validation of a sent message. You most
    /// likely don't need to worry about this.
    ///
    /// Defaults to empty.
    pub fn nonce(mut self, nonce: &str) -> Self {
        self.0.insert("nonce".to_owned(), Value::String(nonce.to_owned()));

        CreateMessage(self.0)
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(mut self, tts: bool) -> Self {
        self.0.insert("tts".to_owned(), Value::Bool(tts));

        CreateMessage(self.0)
    }
}

impl Default for CreateMessage {
    /// Creates a map for sending a [`Message`], setting [`tts`] to `false` by
    /// default.
    ///
    /// [`Message`]: ../../model/struct.Message.html
    /// [`tts`]: #method.tts
    fn default() -> CreateMessage {
        let mut map = BTreeMap::default();
        map.insert("tts".to_owned(), Value::Bool(false));

        CreateMessage(map)
    }
}
