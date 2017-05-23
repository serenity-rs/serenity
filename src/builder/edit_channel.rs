use ::internal::prelude::*;

/// A builder to edit a [`GuildChannel`] for use via one of a couple methods.
///
/// These methods are:
///
/// - [`Context::edit_channel`]
/// - [`GuildChannel::edit`]
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
/// [`Context::edit_channel`]: ../client/struct.Context.html#method.edit_channel
/// [`GuildChannel`]: ../model/struct.GuildChannel.html
/// [`GuildChannel::edit`]: ../model/struct.GuildChannel.html#method.edit
#[derive(Clone, Debug, Default)]
pub struct EditChannel(pub JsonMap);

impl EditChannel {
    /// The bitrate of the channel in bits.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ../model/enum.ChannelType.html#variant.Voice
    pub fn bitrate(mut self, bitrate: u64) -> Self {
        self.0.insert("bitrate".to_owned(), Value::Number(Number::from(bitrate)));

        self
    }

    /// The name of the channel.
    ///
    /// Must be between 2 and 100 characters long.
    pub fn name(mut self, name: &str) -> Self {
        self.0.insert("name".to_owned(), Value::String(name.to_owned()));

        self
    }

    /// The position of the channel in the channel list.
    pub fn position(mut self, position: u64) -> Self {
        self.0.insert("position".to_owned(), Value::Number(Number::from(position)));

        self
    }

    /// The topic of the channel. Can be empty.
    ///
    /// Must be between 0 and 1024 characters long.
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: ../model/enum.ChannelType.html#variant.Text
    pub fn topic(mut self, topic: &str) -> Self {
        self.0.insert("topic".to_owned(), Value::String(topic.to_owned()));

        self
    }

    /// The number of users that may be in the channel simultaneously.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ../model/enum.ChannelType.html#variant.Voice
    pub fn user_limit(mut self, user_limit: u64) -> Self {
        self.0.insert("user_limit".to_owned(), Value::Number(Number::from(user_limit)));

        self
    }
}
