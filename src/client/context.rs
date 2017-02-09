use serde_json::builder::ObjectBuilder;
use std::io::Read;
use std::sync::{Arc, Mutex};
use super::gateway::Shard;
use super::rest::{self, GuildPagination};
use super::login_type::LoginType;
use typemap::ShareMap;
use ::utils::builder::{
    CreateEmbed,
    CreateMessage,
    EditChannel,
    EditProfile,
    Search,
};
use ::internal::prelude::*;
use ::model::*;

#[cfg(feature="extras")]
use std::ops::ShlAssign;

#[cfg(feature="cache")]
use super::CACHE;

/// The context is a general utility struct provided on event dispatches, which
/// helps with dealing with the current "context" of the event dispatch,
/// and providing helper methods where possible. The context also acts as a
/// general high-level interface over the associated [`Shard`] which
/// received the event, or the low-level [`rest`] module.
///
/// For example, when the [`Client::on_message`] handler is dispatched to, the
/// context will contain the Id of the [`Channel`] that the message was created
/// for. This allows for using shortcuts like [`say`], which will
/// post its given argument to the associated channel for you as a [`Message`].
///
/// Additionally, the context contains "shortcuts", like for interacting with
/// the shard. Methods like [`set_game`] will unlock the shard and perform an
/// update for you to save a bit of work.
///
/// A context will only live for the event it was dispatched for. After the
/// event handler finished, it is destroyed and will not be re-used.
///
/// # Automatically using the Cache
///
/// The context makes use of the [`Cache`] being global, and will first check
/// the cache for associated data before hitting the REST API. This is to save
/// Discord requests, and ultimately save your bot bandwidth and time. This also
/// acts as a clean interface for retrieving from the cache without needing to
/// check it yourself first, and then performing a request if it does not exist.
/// The context ultimately acts as a means to simplify these two operations into
/// one.
///
/// For example, if you are needing information about a
/// [channel][`GuildChannel`] within a [guild][`Guild`], then you can
/// use [`get_channel`] to retrieve it. Under most circumstances, the guild and
/// its channels will be cached within the cache, and `get_channel` will just
/// pull from the cache. If it does not exist, it will make a request to the
/// REST API, and then insert a clone of the channel into the cache, returning
/// you the channel.
///
/// In this scenario, now that the cache has the channel, performing the same
/// request to `get_channel` will instead pull from the cache, as it is now
/// cached.
///
/// [`Channel`]: ../model/enum.Channel.html
/// [`Client::on_message`]: struct.Client.html#method.on_message
/// [`Guild`]: ../model/struct.Guild.html
/// [`Message`]: ../model/struct.Message.html
/// [`GuildChannel`]: ../model/struct.GuildChannel.html
/// [`Shard`]: gateway/struct.Shard.html
/// [`Cache`]: ../ext/cache/struct.Cache.html
/// [`get_channel`]: #method.get_channel
/// [`rest`]: rest/index.html
/// [`say`]: #method.say
/// [`set_game`]: #method.set_game
#[derive(Clone)]
pub struct Context {
    /// The Id of the relevant channel, if there is one. This is present on the
    /// [`on_message`] handler, for example.
    ///
    /// [`on_message`]: struct.Client.html#method.on_message
    pub channel_id: Option<ChannelId>,
    /// A clone of [`Client::data`]. Refer to its documentation for more
    /// information.
    ///
    /// [`Client::data`]: struct.Client.html#structfield.data
    pub data: Arc<Mutex<ShareMap>>,
    /// The associated shard which dispatched the event handler.
    ///
    /// Note that if you are sharding, in relevant terms, this is the shard
    /// which received the event being dispatched.
    pub shard: Arc<Mutex<Shard>>,
    /// The queue of messages that are sent after context goes out of scope.
    pub queue: String,
    login_type: LoginType,
}

impl Context {
    /// Create a new Context to be passed to an event handler.
    ///
    /// There's no real reason to use this yourself. But the option is there.
    /// Highly re-consider _not_ using this if you're tempted.
    ///
    /// Or don't do what I say. I'm just a comment hidden from the generated
    /// documentation.
    #[doc(hidden)]
    pub fn new(channel_id: Option<ChannelId>,
               shard: Arc<Mutex<Shard>>,
               data: Arc<Mutex<ShareMap>>,
               login_type: LoginType) -> Context {
        Context {
            channel_id: channel_id,
            data: data,
            shard: shard,
            login_type: login_type,
            queue: String::new(),
        }
    }

    /// Marks the contextual channel as being read up to a certain [`Message`].
    ///
    /// Refer to the documentation for [`rest::ack_message`] for more
    /// information.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidOperationAsBot`] if the current user is
    /// a bot user.
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`ChannelId`]: ../model/struct.ChannelId.html
    /// [`ClientError::InvalidOperationAsBot`]: enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`Message`]: ../model/struct.Message.html
    /// [`rest::ack_message`]: rest/fn.ack_message.html
    /// [`say`]: #method.say
    pub fn ack<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        if self.login_type == LoginType::User {
            return Err(Error::Client(ClientError::InvalidOperationAsUser));
        }

        match self.channel_id {
            Some(channel_id) => channel_id.ack(message_id),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Broadcasts that you are typing to a channel for the next 5 seconds.
    ///
    /// After 5 seconds, another request must be made to continue broadcasting
    /// that you are typing.
    ///
    /// This should rarely be used for bots, and should likely only be used for
    /// signifying that a long-running command is still being executed.
    ///
    /// Requires the [Send Messages] permission.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    /// let _ = context.broadcast_typing();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidOperationAsBot`] if the current user is
    /// a bot user.
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ChannelId`]: ../model/struct.ChannelId.html
    /// [`ClientError::InvalidOperationAsBot`]: enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`say`]: #method.say
    /// [Send Messages]: ../model/permissions/constant.SEND_MESSAGES.html
    pub fn broadcast_typing(&self) -> Result<()> {
        if self.login_type == LoginType::User {
            return Err(Error::Client(ClientError::InvalidOperationAsUser));
        }

        match self.channel_id {
            Some(channel_id) => channel_id.broadcast_typing(),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a
    /// single [`Member`] or [`Role`] within the channel.
    ///
    /// Refer to the documentation for [`GuildChannel::create_permission`] for
    /// more information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`GuildChannel::create_permission`]: ../model/struct.GuildChannel.html#method.create_permission
    /// [`Member`]: ../model/struct.Member.html
    /// [`PermissionOverwrite`]: ../model/struct.PermissionOverwrite.html
    /// [`Role`]: ../model/struct.Role.html
    /// [Manage Channels]: ../model/permissions/constant.MANAGE_CHANNELS.html
    pub fn create_permission(&self, target: PermissionOverwrite)
        -> Result<()> {
        match self.channel_id {
            Some(channel_id) => channel_id.create_permission(target),
            None => Err(Error::Client(ClientError::NoChannelId)),
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
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`Emoji`]: ../model/struct.Emoji.html
    /// [`Message`]: ../model/struct.Message.html
    /// [`Message::react`]: ../model/struct.Message.html#method.react
    /// [Add Reactions]: ../model/permissions/constant.ADD_REACTIONS.html
    pub fn create_reaction<M, R>(&self, message_id: M, reaction_type: R)
        -> Result<()> where M: Into<MessageId>, R: Into<ReactionType> {
        match self.channel_id {
            Some(channel_id) => channel_id.create_reaction(message_id, reaction_type),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Deletes the contextual channel.
    ///
    /// If the channel being deleted is a [`GuildChannel`], then the
    /// [Manage Channels] permission is required.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`GuildChannel`]: ../model/struct.GuildChannel.html
    /// [Manage Channels]: ../model/permissions/constant.MANAGE_CHANNELS.html
    pub fn delete_channel(&self) -> Result<Channel> {
        match self.channel_id {
            Some(channel_id) => channel_id.delete(),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Deletes a [`Message`] given its Id from the contextual channel.
    ///
    /// Refer to [`Message::delete`] for more information.
    ///
    /// # Examples
    ///
    /// Deleting every message that is received:
    ///
    /// ```rust,ignore
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let client = Client::login_bot(&env::var("DISCORD_BOT_TOKEN").unwrap());
    /// client.on_message(|ctx, message| {
    ///     ctx.delete_message(message);
    /// });
    /// ```
    ///
    /// (in practice, please do not do this)
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`Message`]: ../model/struct.Message.html
    /// [`Message::delete`]: ../model/struct.Message.html#method.delete
    /// [Manage Messages]: ../model/permissions/constant.MANAGE_MESSAGES.html
    pub fn delete_message<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        match self.channel_id {
            Some(channel_id) => channel_id.delete_message(message_id),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Deletes all messages by Ids from the given vector in the given channel.
    ///
    /// The minimum amount of messages is 2 and the maximum amount is 100.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// **Note**: This uses bulk delete endpoint which is not available
    /// for user accounts.
    ///
    /// **Note**: Messages that are older than 2 weeks can't be deleted using this method.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// Returns a [`ClientError::InvalidOperationAsUser`] if the current user is
    /// not a bot user.
    ///
    /// [`ClientError::InvalidOperationAsUser`]: enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [Manage Messages]: ../model/permissions/constant.MANAGE_MESSAGES.html
    pub fn delete_messages(&self, message_ids: &[MessageId]) -> Result<()> {
        if self.login_type == LoginType::User {
            return Err(Error::Client(ClientError::InvalidOperationAsUser))
        }

        match self.channel_id {
            Some(channel_id) => channel_id.delete_messages(message_ids),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Deletes all permission overrides in the contextual channel from a member
    /// or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [Manage Channel]: ../model/permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType) -> Result<()> {
        match self.channel_id {
            Some(channel_id) => channel_id.delete_permission(permission_type),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Deletes the given [`Reaction`] from the contextual channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`Reaction`]: ../model/struct.Reaction.html
    /// [Manage Messages]: ../model/permissions/constant.MANAGE_MESSAGES.html
    pub fn delete_reaction<M, R>(&self, message_id: M, user_id: Option<UserId>, reaction_type: R)
        -> Result<()> where M: Into<MessageId>, R: Into<ReactionType> {
        match self.channel_id {
            Some(channel_id) => channel_id.delete_reaction(message_id, user_id, reaction_type),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Edits the settings of a [`Channel`], optionally setting new values.
    ///
    /// Refer to `EditChannel`'s documentation for its methods.
    ///
    /// Requires the [Manage Channel] permission.
    ///
    /// # Examples
    ///
    /// Change a voice channel's name and bitrate:
    ///
    /// ```rust,ignore
    /// context.edit_channel(channel_id, |c| c
    ///     .name("test")
    ///     .bitrate(64000));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a guild channel.
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [Manage Channel]: ../model/permissions/constant.MANAGE_CHANNEL.html
    pub fn edit_channel<F>(&self, f: F) -> Result<GuildChannel>
        where F: FnOnce(EditChannel) -> EditChannel {
        let channel_id = match self.channel_id {
            Some(channel_id) => channel_id,
            None => return Err(Error::Client(ClientError::NoChannelId)),
        };

        #[cfg(feature="cache")]
        {

            if let Channel::Guild(ref channel) = channel_id.get()? {
                let ch = channel.read().unwrap();

                if ch.kind != ChannelType::Text && ch.kind != ChannelType::Voice {
                    return Err(Error::Client(ClientError::UnexpectedChannelType(ch.kind)));
                }
            }
        }

        channel_id.edit(f)
    }

    /// Edits the current user's profile settings.
    ///
    /// Refer to `EditProfile`'s documentation for its methods.
    ///
    /// # Examples
    ///
    /// Change the current user's username:
    ///
    /// ```rust,ignore
    /// context.edit_profile(|p| p.username("Hakase"));
    /// ```
    pub fn edit_profile<F: FnOnce(EditProfile) -> EditProfile>(&self, f: F) -> Result<CurrentUser> {
        let mut map = ObjectBuilder::new();

        feature_cache! {{
            let cache = CACHE.read().unwrap();

            map = map.insert("avatar", &cache.user.avatar)
                .insert("username", &cache.user.name);

            if let Some(email) = cache.user.email.as_ref() {
                map = map.insert("email", email);
            }
        } else {
            let user = rest::get_current_user()?;

            map = map.insert("avatar", user.avatar)
                .insert("username", user.name);

            if let Some(email) = user.email.as_ref() {
                map = map.insert("email", email);
            }
        }}

        let edited = f(EditProfile(map)).0.build();

        rest::edit_profile(edited)
    }

    /// Edits a [`Message`] given its Id and the Id of the channel it belongs
    /// to.
    ///
    /// Refer to [`Channel::edit_message`] for more information.
    ///
    /// **Note**: Requires that the current user be the author of the message.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`Channel::edit_message`]: ../model/enum.Channel.html#method.edit_message
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`Message`]: ../model/struct.Message.html
    pub fn edit_message<F, M>(&self, message_id: M, text: &str, f: F) -> Result<Message>
        where F: FnOnce(CreateEmbed) -> CreateEmbed, M: Into<MessageId> {
        match self.channel_id {
            Some(channel_id) => channel_id.edit_message(message_id, text, f),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Gets a fresh version of the channel over the REST API.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    pub fn get_channel(&self) -> Result<Channel> {
        match self.channel_id {
            Some(channel_id) => channel_id.get(),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Gets all of a [`GuildChannel`]'s invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`GuildChannel`]: ../model/struct.GuildChannel.html
    /// [Manage Guild]: ../model/permissions/constant.MANAGE_GUILD.html
    pub fn get_channel_invites(&self) -> Result<Vec<RichInvite>> {
        match self.channel_id {
            Some(channel_id) => channel_id.get_invites(),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Gets a paginated list of guilds that the current user is in.
    ///
    /// The `limit` has a maximum value of 100.
    ///
    /// See also: [`CurrentUser::guilds`].
    ///
    /// # Examples
    ///
    /// Get the first 10 guilds after the current [`Message`]'s guild's Id:
    ///
    /// ```rust,ignore
    /// use serenity::client::rest::GuildPagination;
    ///
    /// // assuming you are in a context
    ///
    /// let guild_id = message.guild_id().unwrap();
    /// context.get_guilds(GuildPagination::After(guild_id, 10)).unwrap();
    /// ```
    ///
    /// [`CurrentUser::guilds`]: ../model/struct.CurrentUser.html#method.guilds
    /// [`Message`]: ../model/struct.Message.html
    #[inline]
    pub fn get_guilds(&self, target: GuildPagination, limit: u8) -> Result<Vec<GuildInfo>> {
        rest::get_guilds(target, limit as u64)
    }

    /// Gets a single [`Message`] from the contextual channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidOperationAsUser`] if the current user is
    /// not a user account.
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ClientError::InvalidOperationAsUser`]: enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`Message`]: ../model/struct.Message.html
    /// [Read Message History]: ../model/permissions/constant.READ_MESSAGE_HISTORY.html
    pub fn get_message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        if self.login_type == LoginType::User {
            return Err(Error::Client(ClientError::InvalidOperationAsUser))
        }

        match self.channel_id {
            Some(channel_id) => channel_id.get_message(message_id),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
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
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`Emoji`]: ../model/struct.Emoji.html
    /// [`Message`]: ../model/struct.Message.html
    /// [`User`]: ../model/struct.User.html
    /// [Read Message History]: ../model/permissions/constant.READ_MESSAGE_HISTORY.html
    pub fn get_reaction_users<M, R, U>(&self,
                                       message_id: M,
                                       reaction_type: R,
                                       limit: Option<u8>,
                                       after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        match self.channel_id {
            Some(c) => c.get_reaction_users(message_id, reaction_type, limit, after),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Pins a [`Message`] in the specified [`Channel`] by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] if the current context is not
    /// related to a channel.
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#variant.NoChannelId
    /// [`Message`]: ../model/struct.Message.html
    /// [Manage Messages]: ../model/permissions/constant.MANAGE_MESSAGES.html
    pub fn pin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        match self.channel_id {
            Some(channel_id) => channel_id.pin(message_id),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Gets the list of [`Message`]s which are pinned to the specified
    /// [`Channel`].
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`Message`]: ../model/struct.Message.html
    pub fn pins(&self) -> Result<Vec<Message>> {
        match self.channel_id {
            Some(channel_id) => channel_id.pins(),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Sends a message with just the given message content in the channel that
    /// a message was received from.
    ///
    /// # Supported Events
    ///
    /// This will only work through the context of one of the following event
    /// dispatches:
    ///
    /// - [`ChannelCreate`][`Event::ChannelCreate`]
    /// - [`ChannelPinsAck`][`Event::ChannelPinsAck`]
    /// - [`ChannelPinsUpdate`][`Event::ChannelPinsUpdate`]
    /// - [`ChannelRecipientAdd`][`Event::ChannelRecipientAdd`]
    /// - [`ChannelRecipientRemove`][`Event::ChannelRecipientRemove`]
    /// - [`ChannelUpdate`][`Event::ChannelUpdate`]
    /// - [`MessageAck`][`Event::MessageAck`]
    /// - [`MessageDelete`][`Event::MessageDelete`]
    /// - [`MessageDeleteBulk`][`Event::MessageDeleteBulk`]
    /// - [`MessageUpdate`][`Event::MessageUpdate`]
    /// - [`ReactionAdd`][`Event::ReactionAdd`]
    /// - [`ReactionRemove`][`Event::ReactionRemove`]
    /// - [`ReactionRemoveAll`][`Event::ReactionRemoveAll`]
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// Returns a [`ClientError::NoChannelId`] when there is no [`ChannelId`]
    /// directly available; i.e. when not under the context of one of the above
    /// events.
    ///
    /// [`ChannelId`]: ../model/struct.ChannelId.html
    /// [`ClientError::MessageTooLong`]: enum.ClientError.html#variant.MessageTooLong
    /// [`ClientError::NoChannelId`]: enum.ClientError.html#NoChannelId
    /// [`Event::ChannelCreate`]: ../model/event/enum.Event.html#variant.ChannelCreate
    /// [`Event::ChannelPinsAck`]: ../model/event/enum.Event.html#variant.ChannelPinsAck
    /// [`Event::ChannelPinsUpdate`]: ../model/event/enum.Event.html#variant.ChannelPinsUpdate
    /// [`Event::ChannelRecipientAdd`]: ../model/event/enum.Event.html#variant.ChannelRecipientAdd
    /// [`Event::ChannelRecipientRemove`]: ../model/event/enum.Event.html#variant.ChannelRecipientRemove
    /// [`Event::ChannelUpdate`]: ../model/event/enum.Event.html#variant.ChannelUpdate
    /// [`Event::MessageAck`]: ../model/event/enum.Event.html#variant.MessageAck
    /// [`Event::MessageDelete`]: ../model/event/enum.Event.html#variant.MessageDelete
    /// [`Event::MessageDeleteBulk`]: ../model/event/enum.Event.html#variant.MessageDeleteBulk
    /// [`Event::MessageUpdate`]: ../model/event/enum.Event.html#variant.MessageUpdate
    /// [`Event::ReactionAdd`]: ../model/event/enum.Event.html#variant.ReactionAdd
    /// [`Event::ReactionRemove`]: ../model/event/enum.Event.html#variant.ReactionRemove
    /// [`Event::ReactionRemoveAll`]: ../model/event/enum.Event.html#variant.ReactionRemoveAll
    /// [`Message`]: ../model/struct.Message.html
    pub fn say(&self, content: &str) -> Result<Message> {
        match self.channel_id {
            Some(channel_id) => channel_id.send_message(|m| m.content(content)),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Adds a string to message queue, which is sent joined by a newline
    /// when context goes out of scope.
    ///
    /// **Note**: Only works in a context where a channel is present. Refer to
    /// [`say`] for a list of events where this is applicable.
    ///
    /// [`say`]: #method.say
    pub fn queue(&mut self, content: &str) -> &mut Self {
        self.queue.push('\n');
        self.queue.push_str(content);

        self
    }

    /// Searches a [`Channel`]'s messages by providing query parameters via the
    /// search builder.
    ///
    /// Refer to the documentation for the [`Search`] builder for restrictions
    /// and defaults parameters, as well as potentially advanced usage.
    ///
    /// **Note**: Bot users can not search.
    ///
    /// # Examples
    ///
    /// Refer to the [`Search`] builder's documentation for examples,
    /// specifically the section on [searching a channel][search channel].
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    /// [search channel]: ../utils/builder/struct.Search.html#searching-a-channel
    pub fn search_channel<F>(&self, f: F) -> Result<SearchResult>
        where F: FnOnce(Search) -> Search {
        let channel_id = match self.channel_id {
            Some(channel_id) => channel_id,
            None => return Err(Error::Client(ClientError::NoChannelId)),
        };

        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        channel_id.search(f)
    }

    /// Sends a file along with optional message contents. The filename _must_
    /// be specified.
    ///
    /// Message contents may be passed by using the [`CreateMessage::content`]
    /// method.
    ///
    /// An embed can _not_ be sent when sending a file. If you set one, it will
    /// be automatically removed.
    ///
    /// Requires the [Attach Files] and [Send Messages] permissions are required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ClientError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// [`ClientError::MessageTooLong`]: enum.ClientError.html#variant.MessageTooLong
    /// [`CreateMessage::content`]: ../utils/builder/struct.CreateMessage.html#method.content
    /// [`GuildChannel`]: ../model/struct.GuildChannel.html
    /// [Attach Files]: ../model/permissions/constant.ATTACH_FILES.html
    /// [Send Messages]: ../model/permissions/constant.SEND_MESSAGES.html
    pub fn send_file<F, R>(&self, file: R, filename: &str, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, R: Read {
        match self.channel_id {
            Some(channel_id) => channel_id.send_file(file, filename, f),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Sends a message to a [`Channel`].
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// Requires the [Send Messages] permission is required.
    ///
    /// # Example
    ///
    /// Send a message with just the content `test`:
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    /// let _ = context.send_message(|f| f.content("test"));
    /// ```
    ///
    /// Send a message on `!ping` with a very descriptive [`Embed`]. This sends
    /// a message content of `"Pong! Here's some info"`, with an embed with the
    /// following attributes:
    ///
    /// - Dark gold in colour;
    /// - A description of `"Information about the message just posted"`;
    /// - A title of `"Message Information"`;
    /// - A URL of `"https://rust-lang.org"`;
    /// - An [author structure] containing an icon and the user's name;
    /// - An inline [field structure] containing the message's content with a
    ///   label;
    /// - An inline field containing the channel's name with a label;
    /// - A footer containing the current user's icon and name, saying that the
    ///   information was generated by them.
    ///
    /// ```rust,ignore
    /// use serenity::client::{CACHE, Client, Context};
    /// use serenity::model::{Channel, Message};
    /// use serenity::utils::Colour;
    /// use std::env;
    ///
    /// let mut client = Client::login_bot(&env::var("DISCORD_TOKEN").unwrap());
    /// client.with_framework(|f| f
    ///     .configure(|c| c.prefix("~"))
    ///     .on("ping", ping));
    ///
    /// client.on_ready(|_context, ready| {
    ///     println!("{} is connected!", ready.user.name);
    /// });
    ///
    /// let _ = client.start();
    ///
    /// command!(ping(context, message) {
    ///     let cache = CACHE.read().unwrap();
    ///     let channel = cache.get_guild_channel(message.channel_id);
    ///
    ///     let _ = context.send_message(|m| m
    ///         .content("Pong! Here's some info")
    ///         .embed(|e| e
    ///             .colour(Colour::dark_gold())
    ///             .description("Information about the message just posted")
    ///             .title("Message information")
    ///             .url("https://rust-lang.org")
    ///             .author(|mut a| {
    ///                 a = a.name(&message.author.name);
    ///
    ///                 if let Some(avatar) = message.author.avatar_url() {
    ///                     a = a.icon_url(&avatar);
    ///                 }
    ///
    ///                 a
    ///             })
    ///             .field(|f| f
    ///                 .inline(true)
    ///                 .name("Message content:")
    ///                 .value(&message.content))
    ///             .field(|f| f
    ///                 .inline(true)
    ///                 .name("Channel name:")
    ///                 .value(&channel.map_or_else(|| "Unknown", |c| &c.name)))
    ///             .footer(|mut f| {
    ///                 f = f.text(&format!("Generated by {}", cache.user.name));
    ///
    ///                 if let Some(avatar) = cache.user.avatar_url() {
    ///                     f = f.icon_url(&avatar);
    ///                 }
    ///
    ///                 f
    ///             })));
    ///
    ///     Ok(())
    /// });
    /// ```
    ///
    /// Note that for most use cases, your embed layout will _not_ be this ugly.
    /// This is an example of a very involved and conditional embed.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`ClientError::MessageTooLong`]: enum.ClientError.html#variant.MessageTooLong
    /// [`CreateMessage`]: ../utils/builder/struct.CreateMessage.html
    /// [`Embed`]: ../model/struct.Embed.html
    /// [`GuildChannel`]: ../model/struct.GuildChannel.html
    /// [Send Messages]: ../model/permissions/constant.SEND_MESSAGES.html
    /// [author structure]: ../utils/builder/struct.CreateEmbedAuthor.html
    /// [field structure]: ../utils/builder/struct.CreateEmbedField.html
    pub fn send_message<F>(&self, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        match self.channel_id {
            Some(channel_id) => channel_id.send_message(f),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }

    /// Sets the current user as being [`Online`]. This maintains the current
    /// game and `afk` setting.
    ///
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    pub fn online(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::Online);
    }

    /// Sets the current user as being [`Idle`]. This maintains the current
    /// game and `afk` setting.
    ///
    /// [`Idle`]: ../model/enum.OnlineStatus.html#variant.Idle
    pub fn idle(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::Idle);
    }

    /// Sets the current user as being [`DoNotDisturb`]. This maintains the
    /// current game and `afk` setting.
    ///
    /// [`DoNotDisturb`]: ../model/enum.OnlineStatus.html#variant.DoNotDisturb
    pub fn dnd(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::DoNotDisturb);
    }

    /// Sets the current user as being [`Invisible`]. This maintains the current
    /// game and `afk` setting.
    ///
    /// [`Invisible`]: ../model/enum.OnlineStatus.html#variant.Invisible
    pub fn invisible(&self) {
        self.shard.lock().unwrap().set_status(OnlineStatus::Invisible);
    }

    /// "Resets" the current user's presence, by setting the game to `None`,
    /// the online status to [`Online`], and `afk` to `false`.
    ///
    /// Use [`set_presence`] for fine-grained control over individual details.
    ///
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    /// [`set_presence`]: #method.set_presence
    pub fn reset_presence(&self) {
        self.shard.lock()
            .unwrap()
            .set_presence(None, OnlineStatus::Online, false)
    }

    /// Sets the current game, defaulting to an online status of [`Online`], and
    /// setting `afk` to `false`.
    ///
    /// # Examples
    ///
    /// Set the current user as playing "Heroes of the Storm":
    ///
    /// ```rust,ignore
    /// use serenity::model::Game;
    ///
    /// // assuming you are in a context
    ///
    /// context.set_game(Game::playing("Heroes of the Storm"));
    /// ```
    ///
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    pub fn set_game(&self, game: Game) {
        self.shard.lock()
            .unwrap()
            .set_presence(Some(game), OnlineStatus::Online, false);
    }

    /// Sets the current game, passing in only its name. This will automatically
    /// set the current user's [`OnlineStatus`] to [`Online`], and its
    /// [`GameType`] as [`Playing`].
    ///
    /// Use [`reset_presence`] to clear the current game, or [`set_presence`]
    /// for more fine-grained control.
    ///
    /// **Note**: Maximum length is 128.
    ///
    /// [`GameType`]: ../model/enum.GameType.html
    /// [`Online`]: ../model/enum.OnlineStatus.html#variant.Online
    /// [`OnlineStatus`]: ../model/enum.OnlineStatus.html
    /// [`Playing`]: ../model/enum.GameType.html#variant.Playing
    /// [`reset_presence`]: #method.reset_presence
    /// [`set_presence`]: #method.set_presence
    pub fn set_game_name(&self, game_name: &str) {
        let game = Game {
            kind: GameType::Playing,
            name: game_name.to_owned(),
            url: None,
        };

        self.shard.lock()
            .unwrap()
            .set_presence(Some(game), OnlineStatus::Online, false);
    }

    /// Sets the current user's presence, providing all fields to be passed.
    ///
    /// # Examples
    ///
    /// Setting the current user as having no game, being [`Idle`],
    /// and setting `afk` to `true`:
    ///
    /// ```rust,ignore
    /// use serenity::model::OnlineStatus;
    ///
    /// // assuming you are in a context
    ///
    /// context.set_game(None, OnlineStatus::Idle, true);
    /// ```
    ///
    /// Setting the current user as playing "Heroes of the Storm", being
    /// [`DoNotDisturb`], and setting `afk` to `false`:
    ///
    /// ```rust,ignore
    /// use serenity::model::{Game, OnlineStatus};
    ///
    /// // assuming you are in a context
    ///
    /// let game = Game::playing("Heroes of the Storm");
    /// let status = OnlineStatus::DoNotDisturb;
    ///
    /// context.set_game(Some(game), status, false);
    /// ```
    ///
    /// [`DoNotDisturb`]: ../model/enum.OnlineStatus.html#variant.DoNotDisturb
    /// [`Idle`]: ../model/enum.OnlineStatus.html#variant.Idle
    pub fn set_presence(&self,
                        game: Option<Game>,
                        status: OnlineStatus,
                        afk: bool) {
        self.shard.lock()
            .unwrap()
            .set_presence(game, status, afk)
    }

    /// Unpins a [`Message`] in the contextual channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`Message`]: ../model/struct.Message.html
    /// [Manage Messages]: ../model/permissions/constant.MANAGE_MESSAGES.html
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        match self.channel_id {
            Some(channel_id) => channel_id.unpin(message_id),
            None => Err(Error::Client(ClientError::NoChannelId)),
        }
    }
}

impl Drop for Context {
    /// Combines and sends all queued messages.
    fn drop(&mut self) {
        if !self.queue.is_empty() {
            let _ = self.say(&self.queue);
        }
    }
}

/// Allows the `<<=` operator to be used to queue messages.
#[cfg(feature="extras")]
impl<'a> ShlAssign<&'a str> for &'a mut Context {
    fn shl_assign(&mut self, rhs: &str) {
        self.queue(rhs);
    }
}
