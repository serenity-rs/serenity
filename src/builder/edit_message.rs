use internal::prelude::*;
use std::fmt::Display;
use super::CreateEmbed;
use utils::{self, VecMap};

/// A builder to specify the fields to edit in an existing message.
///
/// # Examples
///
/// Editing the content of a [`Message`] to `"hello"`:
///
/// ```rust,no_run
/// # use serenity::model::id::{ChannelId, MessageId};
/// #
/// # let mut message = ChannelId(7).message(MessageId(8)).unwrap();
/// #
/// let _ = message.edit(|mut m| {
///     m.content("hello");
///
///     m
/// });
/// ```
///
/// [`Message`]: ../model/channel/struct.Message.html
#[derive(Clone, Debug, Default)]
pub struct EditMessage(pub VecMap<&'static str, Value>);

impl EditMessage {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content<D: Display>(&mut self, content: D) {
        self._content(content.to_string())
    }

    fn _content(&mut self, content: String) {
        self.0.insert("content", Value::String(content));
    }

    /// Set an embed for the message.
    pub fn embed<F: FnOnce(CreateEmbed) -> CreateEmbed>(&mut self, f: F) {
        let map = utils::vecmap_to_json_map(f(CreateEmbed::default()).0);
        let embed = Value::Object(map);

        self.0.insert("embed", embed);
    }
}
