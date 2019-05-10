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
    pub fn bitrate(mut self, bitrate: u64) -> Self {
        self.0.insert("bitrate", Value::Number(Number::from(bitrate)));

        self
    }

    /// The name of the channel.
    ///
    /// Must be between 2 and 100 characters long.
    pub fn name(mut self, name: &str) -> Self {
        self.0.insert("name", Value::String(name.to_string()));

        self
    }

    /// The position of the channel in the channel list.
    pub fn position(mut self, position: u64) -> Self {
        self.0.insert("position", Value::Number(Number::from(position)));

        self
    }

    /// The topic of the channel. Can be empty.
    ///
    /// Must be between 0 and 1024 characters long.
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: ../model/channel/enum.ChannelType.html#variant.Text
    pub fn topic(mut self, topic: &str) -> Self {
        self.0.insert("topic", Value::String(topic.to_string()));

        self
    }

    /// Is the channel inappropriate for work?
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: ../model/channel/enum.ChannelType.html#variant.Text
    pub fn nsfw(&mut self, nsfw: bool) -> &mut Self {
        self.0.insert("nsfw", Value::Bool(nsfw));

        self
    }

    /// The number of users that may be in the channel simultaneously.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ../model/channel/enum.ChannelType.html#variant.Voice
    pub fn user_limit(mut self, user_limit: u64) -> Self {
        self.0.insert("user_limit", Value::Number(Number::from(user_limit)));

        self
    }

    /// The parent category of the channel.
    ///
    /// This is for [text] and [voice] channels only.
    ///
    /// [text]: ../model/channel/enum.ChannelType.html#variant.Text
    /// [voice]: ../model/channel/enum.ChannelType.html#variant.Voice
    #[inline]
    pub fn category<C: Into<Option<ChannelId>>>(self, category: C) -> Self {
        self._category(category.into())
    }

    fn _category(mut self, category: Option<ChannelId>) -> Self {
        self.0.insert("parent_id", match category {
            Some(c) => Value::Number(Number::from(c.0)),
            None => Value::Null
        });

        self
    }
}
