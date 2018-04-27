use internal::prelude::*;
use utils::VecMap;
use model::id::ChannelId;

/// A builder to edit a [`GuildChannel`] for use via [`GuildChannel::edit`]
///
/// Defaults are not directly provided by the builder itself.
///
/// # Examples
///
/// Edit a channel, providing a new name and topic:
///
/// ```rust,ignore
/// // assuming a channel has already been bound
/// if let Err(why) = channel::edit(|c| c.name("new name").topic("a test topic")) {
///     // properly handle the error
/// }
/// ```
///
/// [`GuildChannel`]: ../model/channel/struct.GuildChannel.html
/// [`GuildChannel::edit`]: ../model/channel/struct.GuildChannel.html#method.edit
#[derive(Clone, Debug, Default)]
pub struct EditChannel(pub VecMap<&'static str, Value>);

impl EditChannel {
    /// The bitrate of the channel in bits.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ../model/channel/enum.ChannelType.html#variant.Voice
    pub fn bitrate(&mut self, bitrate: u64) {
        self.0.insert("bitrate", Value::Number(Number::from(bitrate)));
    }

    /// The name of the channel.
    ///
    /// Must be between 2 and 100 characters long.
    pub fn name(&mut self, name: &str) {
        self.0.insert("name", Value::String(name.to_string()));
    }

    /// The position of the channel in the channel list.
    pub fn position(&mut self, position: u64) {
        self.0.insert("position", Value::Number(Number::from(position)));
    }

    /// The topic of the channel. Can be empty.
    ///
    /// Must be between 0 and 1024 characters long.
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: ../model/channel/enum.ChannelType.html#variant.Text
    pub fn topic(&mut self, topic: &str) {
        self.0.insert("topic", Value::String(topic.to_string()));
    }

    /// The number of users that may be in the channel simultaneously.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ../model/channel/enum.ChannelType.html#variant.Voice
    pub fn user_limit(&mut self, user_limit: u64) {
        self.0.insert("user_limit", Value::Number(Number::from(user_limit)));
    }

    /// The parent category of the channel.
    ///
    /// This is for [text] and [voice] channels only.
    ///
    /// [text]: ../model/channel/enum.ChannelType.html#variant.Text
    /// [voice]: ../model/channel/enum.ChannelType.html#variant.Voice
    pub fn category<C: Into<Option<ChannelId>>>(&mut self, category: C) {
        let parent_id = match category.into() {
            Some(c) => Value::Number(Number::from(c.0)),
            None => Value::Null
        };

        self.0.insert("parent_id", parent_id);
    }
}
