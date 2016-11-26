use serde_json::builder::ObjectBuilder;
use std::default::Default;

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
pub struct EditChannel(pub ObjectBuilder);

impl EditChannel {
    /// The bitrate of the channel in bits.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ../model/enum.ChannelType.html#variant.Voice
    pub fn bitrate(self, bitrate: u64) -> Self {
        EditChannel(self.0.insert("bitrate", bitrate))
    }

    /// The name of the channel.
    ///
    /// Must be between 2 and 100 characters long.
    pub fn name(self, name: &str) -> Self {
        EditChannel(self.0.insert("name", name))
    }

    /// The position of the channel in the channel list.
    pub fn position(self, position: u64) -> Self {
        EditChannel(self.0.insert("position", position))
    }

    /// The topic of the channel. Can be empty.
    ///
    /// Must be between 0 and 1024 characters long.
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: ../model/enum.ChannelType.html#variant.Text
    pub fn topic(self, topic: &str) -> Self {
        EditChannel(self.0.insert("topic", topic))
    }

    /// The number of users that may be in the channel simultaneously.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: ../model/enum.ChannelType.html#variant.Voice
    pub fn user_limit(self, user_limit: u64) -> Self {
        EditChannel(self.0.insert("user_limit", user_limit))
    }
}

impl Default for EditChannel {
    /// Creates a builder with no default parameters.
    fn default() -> EditChannel {
        EditChannel(ObjectBuilder::new())
    }
}
