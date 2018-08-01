//! Models relating to channels and types within channels.

mod attachment;
mod channel_id;
mod embed;
mod group;
mod guild_channel;
mod message;
mod private_channel;
mod reaction;
mod channel_category;

pub use self::attachment::*;
pub use self::channel_id::*;
pub use self::embed::*;
pub use self::group::*;
pub use self::guild_channel::*;
pub use self::message::*;
pub use self::private_channel::*;
pub use self::reaction::*;
pub use self::channel_category::*;

use internal::RwLockExt;
use model::prelude::*;
use serde::de::Error as DeError;
use serde::ser::{SerializeStruct, Serialize, Serializer};
use serde_json;
use super::utils::deserialize_u64;

#[cfg(feature = "model")]
use builder::{CreateMessage, EditMessage, GetMessages};
#[cfg(feature = "model")]
use http::AttachmentType;
#[cfg(feature = "model")]
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A container for any channel.
#[derive(Clone, Debug)]
pub enum Channel {
    /// A group. A group comprises of only one channel.
    Group(Arc<RwLock<Group>>),
    /// A [text] or [voice] channel within a [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    /// [text]: enum.ChannelType.html#variant.Text
    /// [voice]: enum.ChannelType.html#variant.Voice
    Guild(Arc<RwLock<GuildChannel>>),
    /// A private channel to another [`User`]. No other users may access the
    /// channel. For multi-user "private channels", use a group.
    ///
    /// [`User`]: struct.User.html
    Private(Arc<RwLock<PrivateChannel>>),
    /// A category of [`GuildChannel`]s
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    Category(Arc<RwLock<ChannelCategory>>),
}

impl Channel {

    /////////////////////////////////////////////////////////////////////////
    // Adapter for each variant
    /////////////////////////////////////////////////////////////////////////

    /// Converts from `Channel` to `Option<Arc<RwLock<Group>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<Group>>>`, consuming `self`,
    /// and discarding a GuildChannel, PrivateChannel, or ChannelCategory, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # extern crate serenity;
    /// # use self::serenity::model::id::ChannelId;
    /// # fn main() {
    /// # let channel = ChannelId(0).get().unwrap();
    /// match channel.group() {
    ///     Some(group_lock) => {
    ///         if let Some(ref name) = group_lock.read().name {
    ///             println!("It's a group named {}!", name);
    ///         } else {
    ///              println!("It's an unnamed group!");
    ///         }
    ///     },
    ///     None => { println!("It's not a group!"); },
    /// }
    /// # }
    /// ```


    pub fn group(self) -> Option<Arc<RwLock<Group>>> {
        match self {
            Channel::Group(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<Arc<RwLock<GuildChannel>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<GuildChannel>>>`, consuming `self`,
    /// and discarding a Group, PrivateChannel, or ChannelCategory, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # extern crate serenity;
    /// # use self::serenity::model::id::ChannelId;
    /// # fn main() {
    /// let channel = ChannelId(0).get().unwrap();
    /// match channel.guild() {
    ///     Some(guild_lock) => {
    ///         println!("It's a guild named {}!", guild_lock.read().name);
    ///     },
    ///     None => { println!("It's not a guild!"); },
    /// }
    /// # }
    /// ```

    pub fn guild(self) -> Option<Arc<RwLock<GuildChannel>>> {
        match self {
            Channel::Guild(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<Arc<RwLock<PrivateChannel>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<PrivateChannel>>>`, consuming `self`,
    /// and discarding a Group, GuildChannel, or ChannelCategory, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # extern crate serenity;
    /// # use self::serenity::model::id::ChannelId;
    /// # fn main() {
    /// # let channel = ChannelId(0).get().unwrap();
    /// match channel.private() {
    ///     Some(private_lock) => {
    ///         let private = private_lock.read();
    ///         let recipient_lock = &private.recipient;
    ///         let recipient = recipient_lock.read();
    ///         println!("It's a private channel with {}!", recipient.name);
    ///     },
    ///     None => { println!("It's not a private channel!"); },
    /// }
    /// # }
    /// ```

    pub fn private(self) -> Option<Arc<RwLock<PrivateChannel>>> {
        match self {
            Channel::Private(lock) => Some(lock),
            _ => None,
        }
    }

    /// Converts from `Channel` to `Option<Arc<RwLock<ChannelCategory>>>`.
    ///
    /// Converts `self` into an `Option<Arc<RwLock<ChannelCategory>>>`, consuming `self`,
    /// and discarding a Group, GuildChannel, or PrivateChannel, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # extern crate serenity;
    /// # use self::serenity::model::id::ChannelId;
    /// # fn main() {
    /// # let channel = ChannelId(0).get().unwrap();
    /// match channel.category() {
    ///     Some(category_lock) => {
    ///         println!("It's a category named {}!", category_lock.read().name);
    ///     },
    ///     None => { println!("It's not a category!"); },
    /// }
    /// # }
    /// ```

    pub fn category(self) -> Option<Arc<RwLock<ChannelCategory>>> {
        match self {
            Channel::Category(lock) => Some(lock),
            _ => None,
        }
    }

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most
    /// cases.
    ///
    /// Requires the [Add Reactions] permission, _if_ the current user is the
    /// first user to perform a react with a certain emoji.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`Message::react`]: struct.Message.html#method.react
    /// [Add Reactions]: permissions/constant.ADD_REACTIONS.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn create_reaction<M, R>(&self, message_id: M, reaction_type: R) -> Result<()>
        where M: Into<MessageId>, R: Into<ReactionType> {
        self.id().create_reaction(message_id, reaction_type)
    }

    /// Deletes the inner channel.
    ///
    /// **Note**: There is no real function as _deleting_ a [`Group`]. The
    /// closest functionality is leaving it.
    ///
    /// [`Group`]: struct.Group.html
    #[cfg(feature = "model")]
    pub fn delete(&self) -> Result<()> {
        match *self {
            Channel::Group(ref group) => {
                let _ = group.read().leave()?;
            },
            Channel::Guild(ref public_channel) => {
                let _ = public_channel.read().delete()?;
            },
            Channel::Private(ref private_channel) => {
                let _ = private_channel.read().delete()?;
            },
            Channel::Category(ref category) => {
                category.read().delete()?;
            },
        }

        Ok(())
    }

    /// Deletes a [`Message`] given its Id.
    ///
    /// Refer to [`Message::delete`] for more information.
    ///
    /// Requires the [Manage Messages] permission, if the current user is not
    /// the author of the message.
    ///
    /// [`Message`]: struct.Message.html
    /// [`Message::delete`]: struct.Message.html#method.delete
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn delete_message<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.id().delete_message(message_id)
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn delete_reaction<M, R>(&self,
                                 message_id: M,
                                 user_id: Option<UserId>,
                                 reaction_type: R)
                                 -> Result<()>
        where M: Into<MessageId>, R: Into<ReactionType> {
        self.id()
            .delete_reaction(message_id, user_id, reaction_type)
    }

    /// Edits a [`Message`] in the channel given its Id.
    ///
    /// Message editing preserves all unchanged message data.
    ///
    /// Refer to the documentation for [`EditMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Requires that the current user be the author of the message.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the [`the limit`], containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    /// [`EditMessage`]: ../builder/struct.EditMessage.html
    /// [`Message`]: struct.Message.html
    /// [`the limit`]: ../builder/struct.EditMessage.html#method.content
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn edit_message<F, M>(&self, message_id: M, f: F) -> Result<Message>
        where F: FnOnce(EditMessage) -> EditMessage, M: Into<MessageId> {
        self.id().edit_message(message_id, f)
    }

    /// Determines if the channel is NSFW.
    ///
    /// Refer to [`utils::is_nsfw`] for more details.
    ///
    /// [`utils::is_nsfw`]: ../../utils/fn.is_nsfw.html
    #[cfg(all(feature = "model", feature = "utils"))]
    #[inline]
    pub fn is_nsfw(&self) -> bool {
        match *self {
            Channel::Guild(ref channel) => channel.with(|c| c.is_nsfw()),
            Channel::Category(ref category) => category.with(|c| c.is_nsfw()),
            Channel::Group(_) | Channel::Private(_) => false,
        }
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.id().message(message_id)
    }

    /// Gets messages from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use serenity::model::MessageId;
    ///
    /// let id = MessageId(81392407232380928);
    ///
    /// // Maximum is 100.
    /// let _messages = channel.messages(|g| g.after(id).limit(100));
    /// ```
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.id().messages(f)
    }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different
    /// maximum number of users. The maximum that may be retrieve at a time is
    /// `100`, if a greater number is provided then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain
    /// user. This is useful for pagination.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: struct.User.html
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn reaction_users<M, R, U>(&self,
        message_id: M,
        reaction_type: R,
        limit: Option<u8>,
        after: U,
    ) -> Result<Vec<User>> where M: Into<MessageId>,
                                 R: Into<ReactionType>,
                                 U: Into<Option<UserId>> {
        self.id().reaction_users(message_id, reaction_type, limit, after)
    }

    /// Retrieves the Id of the inner [`Group`], [`GuildChannel`], or
    /// [`PrivateChannel`].
    ///
    /// [`Group`]: struct.Group.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    pub fn id(&self) -> ChannelId {
        match *self {
            Channel::Group(ref group) => group.with(|g| g.channel_id),
            Channel::Guild(ref ch) => ch.with(|c| c.id),
            Channel::Private(ref ch) => ch.with(|c| c.id),
            Channel::Category(ref category) => category.with(|c| c.id),
        }
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ChannelId`]: struct.ChannelId.html
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn say(&self, content: &str) -> Result<Message> { self.id().say(content) }

    /// Sends (a) file(s) along with optional message contents.
    ///
    /// Refer to [`ChannelId::send_files`] for examples and more information.
    ///
    /// The [Attach Files] and [Send Messages] permissions are required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ClientError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// [`ChannelId::send_files`]: struct.ChannelId.html#method.send_files
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn send_files<'a, F, T, It: IntoIterator<Item=T>>(&self, files: It, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, T: Into<AttachmentType<'a>> {
        self.id().send_files(files, f)
    }

    /// Sends a message to the channel.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// The [Send Messages] permission is required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    /// [`CreateMessage`]: ../builder/struct.CreateMessage.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn send_message<F>(&self, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        self.id().send_message(f)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[cfg(feature = "model")]
    #[deprecated(since = "0.4.2", note = "Use the inner channel's method")]
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.id().unpin(message_id)
    }
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let v = JsonMap::deserialize(deserializer)?;
        let kind = {
            let kind = v.get("type").ok_or_else(|| DeError::missing_field("type"))?;

            kind.as_u64().unwrap()
        };

        match kind {
            0 | 2 => serde_json::from_value::<GuildChannel>(Value::Object(v))
                .map(|x| Channel::Guild(Arc::new(RwLock::new(x))))
                .map_err(DeError::custom),
            1 => serde_json::from_value::<PrivateChannel>(Value::Object(v))
                .map(|x| Channel::Private(Arc::new(RwLock::new(x))))
                .map_err(DeError::custom),
            3 => serde_json::from_value::<Group>(Value::Object(v))
                .map(|x| Channel::Group(Arc::new(RwLock::new(x))))
                .map_err(DeError::custom),
            4 => serde_json::from_value::<ChannelCategory>(Value::Object(v))
                .map(|x| Channel::Category(Arc::new(RwLock::new(x))))
                .map_err(DeError::custom),
            _ => Err(DeError::custom("Unknown channel type")),
        }
    }
}

impl Serialize for Channel {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        match *self {
            Channel::Category(ref c) => {
                ChannelCategory::serialize(&*c.read(), serializer)
            },
            Channel::Group(ref c) => {
                Group::serialize(&*c.read(), serializer)
            },
            Channel::Guild(ref c) => {
                GuildChannel::serialize(&*c.read(), serializer)
            },
            Channel::Private(ref c) => {
                PrivateChannel::serialize(&*c.read(), serializer)
            },
        }
    }
}

#[cfg(feature = "model")]
impl Display for Channel {
    /// Formats the channel into a "mentioned" string.
    ///
    /// This will return a different format for each type of channel:
    ///
    /// - [`Group`]s: the generated name retrievable via [`Group::name`];
    /// - [`PrivateChannel`]s: the recipient's name;
    /// - [`GuildChannel`]s: a string mentioning the channel that users who can
    /// see the channel can click on.
    ///
    /// [`Group`]: struct.Group.html
    /// [`Group::name`]: struct.Group.html#method.name
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Channel::Group(ref group) => Display::fmt(&group.read().name(), f),
            Channel::Guild(ref ch) => Display::fmt(&ch.read().id.mention(), f),
            Channel::Private(ref ch) => {
                let channel = ch.read();
                let recipient = channel.recipient.read();

                Display::fmt(&recipient.name, f)
            },
            Channel::Category(ref category) => Display::fmt(&category.read().name, f),
        }
    }
}

/// A representation of a type of channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ChannelType {
    /// An indicator that the channel is a text [`GuildChannel`].
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    Text = 0,
    /// An indicator that the channel is a [`PrivateChannel`].
    ///
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    Private = 1,
    /// An indicator that the channel is a voice [`GuildChannel`].
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    Voice = 2,
    /// An indicator that the channel is the channel of a [`Group`].
    ///
    /// [`Group`]: struct.Group.html
    Group = 3,
    /// An indicator that the channel is the channel of a [`ChannelCategory`].
    ///
    /// [`ChannelCategory`]: struct.ChannelCategory.html
    Category = 4,
}

enum_number!(
    ChannelType {
        Text,
        Private,
        Voice,
        Group,
        Category,
    }
);

impl ChannelType {
    pub fn name(&self) -> &str {
        match *self {
            ChannelType::Group => "group",
            ChannelType::Private => "private",
            ChannelType::Text => "text",
            ChannelType::Voice => "voice",
            ChannelType::Category => "category",
        }
    }

    pub fn num(&self) -> u64 {
        match *self {
            ChannelType::Text => 0,
            ChannelType::Private => 1,
            ChannelType::Voice => 2,
            ChannelType::Group => 3,
            ChannelType::Category => 4,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    #[serde(deserialize_with = "deserialize_u64")] id: u64,
    #[serde(rename = "type")] kind: String,
}

/// A channel-specific permission overwrite for a member or role.
#[derive(Clone, Debug)]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteType,
}

impl<'de> Deserialize<'de> for PermissionOverwrite {
    fn deserialize<D: Deserializer<'de>>(deserializer: D)
                                         -> StdResult<PermissionOverwrite, D::Error> {
        let data = PermissionOverwriteData::deserialize(deserializer)?;

        let kind = match &data.kind[..] {
            "member" => PermissionOverwriteType::Member(UserId(data.id)),
            "role" => PermissionOverwriteType::Role(RoleId(data.id)),
            _ => return Err(DeError::custom("Unknown PermissionOverwriteType")),
        };

        Ok(PermissionOverwrite {
            allow: data.allow,
            deny: data.deny,
            kind,
        })
    }
}

impl Serialize for PermissionOverwrite {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        let (id, kind) = match self.kind {
            PermissionOverwriteType::Member(id) => (id.0, "member"),
            PermissionOverwriteType::Role(id) => (id.0, "role"),
        };

        let mut state = serializer.serialize_struct("PermissionOverwrite", 4)?;
        state.serialize_field("allow", &self.allow.bits())?;
        state.serialize_field("deny", &self.deny.bits())?;
        state.serialize_field("id", &id)?;
        state.serialize_field("type", kind)?;

        state.end()
    }
}

/// The type of edit being made to a Channel's permissions.
///
/// This is for use with methods such as `GuildChannel::create_permission`.
///
/// [`GuildChannel::create_permission`]: struct.GuildChannel.html#method.create_permission
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PermissionOverwriteType {
    /// A member which is having its permission overwrites edited.
    Member(UserId),
    /// A role which is having its permission overwrites edited.
    Role(RoleId),
}

#[cfg(test)]
mod test {
    #[cfg(feature = "utils")]
    mod utils {
        use model::prelude::*;
        use parking_lot::RwLock;
        use std::collections::HashMap;
        use std::sync::Arc;

        fn group() -> Group {
            Group {
                channel_id: ChannelId(1),
                icon: None,
                last_message_id: None,
                last_pin_timestamp: None,
                name: None,
                owner_id: UserId(2),
                recipients: HashMap::new(),
            }
        }

        fn guild_channel() -> GuildChannel {
            GuildChannel {
                id: ChannelId(1),
                bitrate: None,
                category_id: None,
                guild_id: GuildId(2),
                kind: ChannelType::Text,
                last_message_id: None,
                last_pin_timestamp: None,
                name: "nsfw-stuff".to_string(),
                permission_overwrites: vec![],
                position: 0,
                topic: None,
                user_limit: None,
                nsfw: false,
            }
        }

        fn private_channel() -> PrivateChannel {
            PrivateChannel {
                id: ChannelId(1),
                last_message_id: None,
                last_pin_timestamp: None,
                kind: ChannelType::Private,
                recipient: Arc::new(RwLock::new(User {
                    id: UserId(2),
                    avatar: None,
                    bot: false,
                    discriminator: 1,
                    name: "ab".to_string(),
                })),
            }
        }

        #[test]
        fn nsfw_checks() {
            let mut channel = guild_channel();
            assert!(channel.is_nsfw());
            channel.kind = ChannelType::Voice;
            assert!(!channel.is_nsfw());

            channel.kind = ChannelType::Text;
            channel.name = "nsfw-".to_string();
            assert!(!channel.is_nsfw());

            channel.name = "nsfw".to_string();
            assert!(channel.is_nsfw());
            channel.kind = ChannelType::Voice;
            assert!(!channel.is_nsfw());
            channel.kind = ChannelType::Text;

            channel.name = "nsf".to_string();
            channel.nsfw = true;
            assert!(channel.is_nsfw());
            channel.nsfw = false;
            assert!(!channel.is_nsfw());

            let channel = Channel::Guild(Arc::new(RwLock::new(channel)));
            assert!(!channel.is_nsfw());

            let group = group();
            assert!(!group.is_nsfw());

            let private_channel = private_channel();
            assert!(!private_channel.is_nsfw());
        }
    }
}
