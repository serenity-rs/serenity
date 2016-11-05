//! A set of builders used to make using methods on certain structs simpler to
//! use.
//!
//! These are used when not all parameters are required, all parameters are
//! optional, and/or sane default values for required parameters can be applied
//! by a builder.

use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use std::collections::BTreeMap;
use std::default::Default;
use ::model::{
    ChannelId,
    MessageId,
    Permissions,
    Region,
    RoleId,
    Role,
    VerificationLevel,
    permissions,
};

/// A builder to create a [`RichInvite`] for use via [`Context::create_invite`].
///
/// This is a structured and cleaner way of creating an invite, as all
/// parameters are optional.
///
/// # Examples
///
/// Create an invite with a max age of 3600 seconds and 10 max uses:
///
/// ```rust,ignore
/// // assuming a `client` has been bound
/// client.on_message(|context, message| {
///     if message.content == "!invite" {
///         let invite = context.create_invite(message.channel_id, |i| i
///             .max_age(3600)
///             .max_uses(10));
///     }
/// });
/// ```
///
/// [`Context::create_invite`]: ../client/struct.Context.html#method.create_invite
/// [`RichInvite`]: ../model/struct.Invite.html
pub struct CreateInvite(pub ObjectBuilder);

impl CreateInvite {
    /// The duration that the invite will be valid for.
    ///
    /// Set to `0` for an invite which does not expire after an amount of time.
    ///
    /// Defaults to `86400`, or 24 hours.
    pub fn max_age(self, max_age: u64) -> Self {
        CreateInvite(self.0.insert("max_age", max_age))
    }

    /// The number of uses that the invite will be valid for.
    ///
    /// Set to `0` for an invite which does not expire after a number of uses.
    ///
    /// Defaults to `0`.
    pub fn max_uses(self, max_uses: u64) -> Self {
        CreateInvite(self.0.insert("max_uses", max_uses))
    }

    /// Whether an invite grants a temporary membership.
    ///
    /// Defaults to `false`.
    pub fn temporary(self, temporary: bool) -> Self {
        CreateInvite(self.0.insert("temporary", temporary))
    }

    /// Whether or not to try to reuse a similar invite.
    ///
    /// Defaults to `false`.
    pub fn unique(self, unique: bool) -> Self {
        CreateInvite(self.0.insert("unique", unique))
    }
}

impl Default for CreateInvite {
    fn default() -> CreateInvite {
        CreateInvite(ObjectBuilder::new().insert("validate", Value::Null))
    }
}

/// A builer to create or edit a [`Role`] for use via a number of model and
/// context methods.
///
/// These are:
///
/// - [`Context::create_role`]
/// - [`Context::edit_role`]
/// - [`LiveGuild::create_role`]
/// - [`Role::edit`]
///
/// Defaults are provided for each parameter on role creation.
///
/// # Examples
///
/// Create a hoisted, mentionable role named "a test role":
///
/// ```rust,ignore
/// // assuming you are in a `context` and a `guild_id` has been bound
/// let role = context.create_role(guild_id, |r| r
///     .hoist(true)
///     .mentionable(true)
///     .name("a test role"));
/// ```
///
/// [`Context::create_role`]: ../client/struct.Context.html#method.create_role
/// [`Context::edit_role`]: ../client/struct.Context.html#method.edit_role
/// [`LiveGuild::create_role`]: ../model/struct.LiveGuild.html#method.create_role
/// [`Role`]: ../model/struct.Role.html
/// [`Role::edit`]: ../model/struct.Role.html#method.edit
pub struct EditRole(pub ObjectBuilder);

impl EditRole {
    /// Creates a new builder with the values of the given [`Role`].
    pub fn new(role: &Role) -> Self {
        EditRole(ObjectBuilder::new()
            .insert("color", role.colour)
            .insert("hoist", role.hoist)
            .insert("managed", role.managed)
            .insert("mentionable", role.mentionable)
            .insert("name", &role.name)
            .insert("permissions", role.permissions.bits())
            .insert("position", role.position))
    }

    /// Sets the colour of the role.
    pub fn colour(self, colour: u64) -> Self {
        EditRole(self.0.insert("color", colour))
    }

    /// Whether or not to hoist the role above lower-positioned role in the user
    /// list.
    pub fn hoist(self, hoist: bool) -> Self {
        EditRole(self.0.insert("hoist", hoist))
    }

    /// Whether or not to make the role mentionable, notifying its users.
    pub fn mentionable(self, mentionable: bool) -> Self {
        EditRole(self.0.insert("mentionable", mentionable))
    }

    /// The name of the role to set.
    pub fn name(self, name: &str) -> Self {
        EditRole(self.0.insert("name", name))
    }

    /// The set of permissions to assign the role.
    pub fn permissions(self, permissions: Permissions) -> Self {
        EditRole(self.0.insert("permissions", permissions.bits()))
    }

    /// The position to assign the role in the role list. This correlates to the
    /// role's position in the user list.
    pub fn position(self, position: u8) -> Self {
        EditRole(self.0.insert("position", position))
    }
}

impl Default for EditRole {
    /// Creates a builder with default parameters.
    ///
    /// The defaults are:
    ///
    /// - **color**: 10070709
    /// - **hoist**: false
    /// - **mentionable**: false
    /// - **name**: new role
    /// - **permissions**: the [general permissions set]
    /// - **position**: 1
    ///
    /// [general permissions set]: ../model/permissions/fn.general.html
    fn default() -> EditRole {
        EditRole(ObjectBuilder::new()
            .insert("color", 10070709)
            .insert("hoist", false)
            .insert("mentionable", false)
            .insert("name", String::from("new role"))
            .insert("permissions", permissions::general().bits())
            .insert("position", 1))
    }
}

/// A builder to edit a [`PublicChannel`] for use via one of a couple methods.
///
/// These methods are:
///
/// - [`Context::edit_channel`]
/// - [`PublicChannel::edit`]
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
/// [`PublicChannel`]: ../model/struct.PublicChannel.html
/// [`PublicChannel::edit`]: ../model/struct.PublicChannel.html#method.edit
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

pub struct EditGuild(pub ObjectBuilder);

impl EditGuild {
    pub fn afk_channel<C: Into<ChannelId>>(self, channel: Option<C>) -> Self {
        EditGuild(match channel {
            Some(channel) => self.0.insert("afk_channel_id", channel.into().0),
            None => self.0.insert("afk-channel_id", Value::Null),
        })
    }

    pub fn afk_timeout(self, timeout: u64) -> Self {
        EditGuild(self.0.insert("afk_timeout", timeout))
    }

    pub fn icon(self, icon: Option<&str>) -> Self {
        EditGuild(self.0
            .insert("icon",
                    icon.map_or_else(|| Value::Null,
                                     |x| Value::String(x.to_owned()))))
    }

    pub fn name(self, name: &str) -> Self {
        EditGuild(self.0.insert("name", name))
    }

    pub fn region(self, region: Region) -> Self {
        EditGuild(self.0.insert("region", region.name()))
    }

    pub fn splash(self, splash: Option<&str>) -> Self {
        EditGuild(self.0
            .insert("splash",
                    splash.map_or_else(|| Value::Null,
                                       |x| Value::String(x.to_owned()))))
    }

    pub fn verification_level<V>(self, verification_level: V) -> Self
        where V: Into<VerificationLevel> {
        EditGuild(self.0.insert("verification_level",
                                verification_level.into().num()))
    }
}

impl Default for EditGuild {
    fn default() -> EditGuild {
        EditGuild(ObjectBuilder::new())
    }
}

pub struct EditMember(pub ObjectBuilder);

impl EditMember {
    pub fn deafen(self, deafen: bool) -> Self {
        EditMember(self.0.insert("deaf", deafen))
    }

    pub fn mute(self, mute: bool) -> Self {
        EditMember(self.0.insert("mute", mute))
    }

    pub fn nickname(self, nickname: &str) -> Self {
        EditMember(self.0.insert("nick", nickname))
    }

    pub fn roles(self, roles: &[RoleId]) -> Self {
        EditMember(self.0
            .insert_array("roles",
                          |a| roles.iter().fold(a, |a, id| a.push(id.0))))
    }

    pub fn voice_channel<C: Into<ChannelId>>(self, channel_id: C) -> Self {
        EditMember(self.0.insert("channel_id", channel_id.into().0))
    }
}

impl Default for EditMember {
    fn default() -> EditMember {
        EditMember(ObjectBuilder::new())
    }
}

pub struct EditProfile(pub ObjectBuilder);

impl EditProfile {
    /// Sets the avatar of the current user. `None` can be passed to remove an
    /// avatar.
    ///
    /// A base64-encoded string is accepted as the avatar content.
    ///
    /// # Examples
    ///
    /// A utility method - [`utils::read_message`] - is provided to read an
    /// image from a file and return its contents in base64-encoded form:
    ///
    /// ```rust,ignore
    /// use serenity::utils;
    ///
    /// // assuming you are in a context
    ///
    /// let base64 = match utils::read_image("./my_image.jpg") {
    ///     Ok(base64) => base64,
    ///     Err(why) => {
    ///         println!("Error reading image: {:?}", why);
    ///
    ///         return;
    ///     },
    /// };
    ///
    /// let _ = context.edit_profile(|profile| {
    ///     profile.avatar(Some(base64))
    /// });
    /// ```
    ///
    /// [`utils::read_image`]: ../utils/fn.read_image.html
    pub fn avatar(self, icon: Option<&str>) -> Self {
        EditProfile(self.0
            .insert("avatar",
                    icon.map_or_else(|| Value::Null,
                                     |x| Value::String(x.to_owned()))))
    }

    /// Modifies the current user's email address.
    ///
    /// Note that when modifying the email address, the current password must
    /// also be [provided].
    ///
    /// No validation is performed on this by the library.
    ///
    /// **Note**: This can only be used by user accounts.
    ///
    /// [provided]: #method.password
    pub fn email(self, email: &str) -> Self {
        EditProfile(self.0.insert("email", email))
    }

    /// Modifies the current user's password.
    ///
    /// Note that when modifying the password, the current password must also be
    /// [provided].
    ///
    /// [provided]: #method.password
    pub fn new_password(self, new_password: &str) -> Self {
        EditProfile(self.0.insert("new_password", new_password))
    }

    /// Used for providing the current password as verification when
    /// [modifying the password] or [modifying the associated email address].
    ///
    /// [modifying the password]: #method.new_password
    /// [modifying the associated email address]: #method.email
    pub fn password(self, password: &str) -> Self {
        EditProfile(self.0.insert("password", password))
    }

    /// Modifies the current user's username.
    ///
    /// When modifying the username, if another user has the same _new_ username
    /// and current discriminator, a new unique discriminator will be assigned.
    /// If there are no available discriminators with the requested username,
    /// an error will occur.
    pub fn username(self, username: &str) -> Self {
        EditProfile(self.0.insert("username", username))
    }
}

impl Default for EditProfile {
    fn default() -> EditProfile {
        EditProfile(ObjectBuilder::new())
    }
}

/// Builds a request for a request to the API to retrieve messages.
///
/// This can have 2 different sets of parameters. The first set is around where
/// to get the messages:
///
/// - `after`
/// - `around`
/// - `before`
/// - `most_recent`
///
/// These can not be mixed, and the first in the list alphabetically will be
/// used. If one is not specified, `most_recent` will be used.
///
/// The fourth parameter is to specify the number of messages to retrieve. This
/// does not _need_ to be called and defaults to a value of 50.
///
/// This should be used only for retrieving messages; see
/// `client::Client::get_messages` for examples.
pub struct GetMessages(pub BTreeMap<String, u64>);

impl GetMessages {
    /// Indicates to retrieve the messages after a specific message, given by
    /// its Id.
    pub fn after<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self.0.insert("after".to_owned(), message_id.into().0);

        self
    }

    /// Indicates to retrieve the messages _around_ a specific message in either
    /// direction (before+after) the given message.
    pub fn around<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self.0.insert("around".to_owned(), message_id.into().0);

        self
    }

    /// Indicates to retrieve the messages before a specific message, given by
    /// its Id.
    pub fn before<M: Into<MessageId>>(mut self, message_id: M) -> Self {
        self.0.insert("before".to_owned(), message_id.into().0);

        self
    }

    /// The maximum number of messages to retrieve for the query.
    ///
    /// If this is not specified, a default value of 50 is used.
    ///
    /// **Note**: This field is capped to 100 messages due to a Discord
    /// limitation. If an amount larger than 100 is supplied, it will be
    /// reduced.
    pub fn limit(mut self, limit: u64) -> Self {
        self.0.insert("limit".to_owned(), if limit > 100 {
            100
        } else {
            limit
        });

        self
    }

    /// This is a function that is here for completeness. You do not need to
    /// call this - except to clear previous calls to `after`, `around`, and
    /// `before` - as it is the default value.
    pub fn most_recent(self) -> Self {
        self
    }
}

impl Default for GetMessages {
    fn default() -> GetMessages {
        GetMessages(BTreeMap::default())
    }
}
