use serde_json::builder::ObjectBuilder;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use super::gateway::Shard;
use super::rest;
use super::login_type::LoginType;
use ::utils::builder::{
    CreateEmbed,
    CreateInvite,
    CreateMessage,
    EditChannel,
    EditGuild,
    EditMember,
    EditProfile,
    EditRole,
    GetMessages
};
use ::internal::prelude::*;
use ::model::*;
use ::utils;

#[cfg(feature = "cache")]
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
    /// The associated shard which dispatched the event handler.
    ///
    /// Note that if you are sharding, in relevant terms, this is the shard
    /// which received the event being dispatched.
    pub shard: Arc<Mutex<Shard>>,
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
               login_type: LoginType) -> Context {
        Context {
            channel_id: channel_id,
            shard: shard,
            login_type: login_type,
        }
    }

    /// Accepts the given invite.
    ///
    /// Refer to the documentation for [`rest::accept_invite`] for restrictions
    /// on accepting an invite.
    ///
    /// **Note**: Requires that the current user be a user account.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidOperationAsBot`] if the current user is
    /// a bot user.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Invite::accept`]: ../model/struct.Invite.html#method.accept
    pub fn accept_invite(&self, invite: &str) -> Result<Invite> {
        if self.login_type == LoginType::Bot {
            return Err(Error::Client(ClientError::InvalidOperationAsBot));
        }

        let code = utils::parse_invite(invite);

        rest::accept_invite(code)
    }

    /// Mark a [`Channel`] as being read up to a certain [`Message`].
    ///
    /// Refer to the documentation for [`rest::ack_message`] for more
    /// information.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidOperationAsBot`] if this is a bot.
    ///
    /// [`Channel`]: ../../model/enum.Channel.html
    /// [`ClientError::InvalidOperationAsBot`]: ../enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`Message`]: ../../model/struct.Message.html
    /// [`rest::ack_message`]: rest/fn.ack_message.html
    pub fn ack<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        if self.login_type == LoginType::User {
            return Err(Error::Client(ClientError::InvalidOperationAsUser));
        }

        rest::ack_message(channel_id.into().0, message_id.into().0)
    }

    /// Ban a [`User`] from a [`Guild`], removing their messages sent in the
    /// last X number of days.
    ///
    /// Refer to the documentation for [`rest::ban_user`] for more information.
    ///
    /// **Note**: Requires that you have the [Ban Members] permission.
    ///
    /// # Examples
    ///
    /// Ban the user that sent a message for `7` days:
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    /// context.ban_user(context.guild_id, context.message.author, 7);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::DeleteMessageDaysAmount`] if the number of days
    /// given is over the maximum allowed.
    ///
    /// [`ClientError::DeleteMessageDaysAmount`]: enum.ClientError.html#variant.DeleteMessageDaysAmount
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`User`]: ../model/struct.User.html
    /// [Ban Members]: ../model/permissions/constant.BAN_MEMBERS.html
    pub fn ban<G, U>(&self, guild_id: G, user_id: U, delete_message_days: u8)
        -> Result<()> where G: Into<GuildId>, U: Into<UserId> {
        if delete_message_days > 7 {
            return Err(Error::Client(ClientError::DeleteMessageDaysAmount(delete_message_days)));
        }

        rest::ban_user(guild_id.into().0, user_id.into().0, delete_message_days)
    }

    /// Broadcast that you are typing to a channel for the next 5 seconds.
    ///
    /// After 5 seconds, another request must be made to continue broadcasting
    /// that you are typing.
    ///
    /// This should rarely be used for bots, and should likely only be used for
    /// signifying that a long-running command is still being executed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    /// context.broadcast_typing(context.channel_id);
    /// ```
    pub fn broadcast_typing<C>(&self, channel_id: C) -> Result<()>
        where C: Into<ChannelId> {
        rest::broadcast_typing(channel_id.into().0)
    }

    /// Creates a [`GuildChannel`] in the given [`Guild`].
    ///
    /// Refer to [`rest::create_channel`] for more information.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Create a voice channel in a guild with the name `test`:
    ///
    /// ```rust,ignore
    /// use serenity::model::ChannelType;
    ///
    /// context.create_channel(context.guild_id, "test", ChannelType::Voice);
    /// ```
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`GuildChannel`]: ../model/struct.GuildChannel.html
    /// [`rest::create_channel`]: rest/fn.create_channel.html
    /// [Manage Channels]: ../model/permissions/constant.MANAGE_CHANNELS.html
    pub fn create_channel<G>(&self, guild_id: G, name: &str, kind: ChannelType)
        -> Result<Channel> where G: Into<GuildId> {
        let map = ObjectBuilder::new()
            .insert("name", name)
            .insert("type", kind.name())
            .build();

        rest::create_channel(guild_id.into().0, map)
    }

    /// Creates an emoji in the given guild with a name and base64-encoded
    /// image. The [`utils::read_image`] function is provided for you as a
    /// simple method to read an image and encode it into base64, if you are
    /// reading from the filesystem.
    ///
    /// **Note**: Requires the [Manage Emojis] permission.
    ///
    /// # Examples
    ///
    /// See the [`EditProfile::avatar`] example for an in-depth example as to
    /// how to read an image from the filesystem and encode it as base64. Most
    /// of the example can be applied similarly for this method.
    ///
    /// [`EditProfile::avatar`]: ../utils/builder/struct.EditProfile.html#method.avatar
    /// [`utils::read_image`]: ../utils/fn.read_image.html
    /// [Manage Emojis]: ../model/permissions/constant.MANAGE_EMOJIS.html
    pub fn create_emoji<G>(&self, guild_id: G, name: &str, image: &str)
        -> Result<Emoji> where G: Into<GuildId> {
        let map = ObjectBuilder::new()
            .insert("name", name)
            .insert("image", image)
            .build();

        rest::create_emoji(guild_id.into().0, map)
    }

    /// Creates a guild with the data provided.
    ///
    /// Only a [`PartialGuild`] will be immediately returned, and a full
    /// [`Guild`] will be received over a [`Shard`].
    ///
    /// **Note**: This endpoint is usually only available for user accounts.
    /// Refer to Discord's information for the endpoint [here][whitelist] for
    /// more information. If you require this as a bot, re-think what you are
    /// doing and if it _really_ needs to be doing this.
    ///
    /// # Examples
    ///
    /// Create a guild called `"test"` in the [US West region] with no icon:
    ///
    /// ```rust,ignore
    /// use serenity::model::Region;
    ///
    /// context.create_guild("test", Region::UsWest, None);
    /// ```
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`PartialGuild`]: ../model/struct.PartialGuild.html
    /// [`Shard`]: ../gateway/struct.Shard.html
    /// [US West region]: ../model/enum.Region.html#variant.UsWest
    /// [whitelist]: https://discordapp.com/developers/docs/resources/guild#create-guild
    pub fn create_guild(&self, name: &str, region: Region, icon: Option<&str>)
        -> Result<PartialGuild> {
        let map = ObjectBuilder::new()
            .insert("icon", icon)
            .insert("name", name)
            .insert("region", region.name())
            .build();

        rest::create_guild(map)
    }

    /// Creates an [`Integration`] for a [`Guild`].
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`Integration`]: ../model/struct.Integration.html
    /// [Manage Guild]: ../model/permissions/constant.MANAGE_GUILD.html
    pub fn create_integration<G, I>(&self,
                                    guild_id: G,
                                    integration_id: I,
                                    kind: &str)
                                    -> Result<()> where G: Into<GuildId>,
                                                        I: Into<IntegrationId> {
        let integration_id = integration_id.into();
        let map = ObjectBuilder::new()
            .insert("id", integration_id.0)
            .insert("type", kind)
            .build();

        rest::create_guild_integration(guild_id.into().0, integration_id.0, map)
    }

    /// Creates an invite for the channel, providing a builder so that fields
    /// may optionally be set.
    ///
    /// See the documentation for the [`CreateInvite`] builder for information
    /// on how to use this and the default values that it provides.
    ///
    /// **Note**: Requires the [Create Invite] permission.
    ///
    /// [`CreateInvite`]: ../utils/builder/struct.CreateInvite.html
    /// [Create Invite]: ../model/permissions/constant.CREATE_INVITE.html
    pub fn create_invite<C, F>(&self, channel_id: C, f: F) -> Result<RichInvite>
        where C: Into<ChannelId>, F: FnOnce(CreateInvite) -> CreateInvite {
        let map = f(CreateInvite::default()).0.build();

        rest::create_invite(channel_id.into().0, map)
    }

    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a
    /// single [`Member`] or [`Role`] within a [`Channel`].
    ///
    /// Refer to the documentation for [`PermissionOverwrite`]s for more
    /// information.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Creating a permission overwrite for a member by specifying the
    /// [`PermissionOverwrite::Member`] variant, allowing it the [Send Messages]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,ignore
    /// use serenity::model::{ChannelId, PermissionOverwrite, permissions};
    ///
    /// // assuming you are in a context
    ///
    /// let channel_id = 7;
    /// let user_id = 8;
    ///
    /// let allow = permissions::SEND_MESSAGES;
    /// let deny = permissions::SEND_TTS_MESSAGES | permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let _result = context.create_permission(channel_id, overwrite);
    /// ```
    ///
    /// Creating a permission overwrite for a role by specifying the
    /// [`PermissionOverwrite::Role`] variant, allowing it the [Manage Webhooks]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,ignore
    /// use serenity::model::{ChannelId, PermissionOverwrite, permissions};
    ///
    /// // assuming you are in a context
    ///
    /// let channel_id = 7;
    /// let user_id = 8;
    ///
    /// let allow = permissions::SEND_MESSAGES;
    /// let deny = permissions::SEND_TTS_MESSAGES | permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let _result = context.create_permission(channel_id, overwrite);
    /// ```
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`Member`]: ../model/struct.Member.html
    /// [`PermissionOverwrite`]: ../model/struct.PermissionOverWrite.html
    /// [`PermissionOverwrite::Member`]: ../model/struct.PermissionOverwrite.html#variant.Member
    /// [`Role`]: ../model/struct.Role.html
    /// [Attach Files]: ../model/permissions/constant.ATTACH_FILES.html
    /// [Manage Channels]: ../model/permissions/constant.MANAGE_CHANNELS.html
    /// [Send TTS Messages]: ../model/permissions/constant.SEND_TTS_MESSAGES.html
    pub fn create_permission<C>(&self,
                                channel_id: C,
                                target: PermissionOverwrite)
                                -> Result<()> where C: Into<ChannelId> {
        let (id, kind) = match target.kind {
            PermissionOverwriteType::Member(id) => (id.0, "member"),
            PermissionOverwriteType::Role(id) => (id.0, "role"),
        };

        let map = ObjectBuilder::new()
            .insert("allow", target.allow.bits())
            .insert("deny", target.deny.bits())
            .insert("id", id)
            .insert("type", kind)
            .build();

        rest::create_permission(channel_id.into().0, id, map)
    }

    /// Creates a direct message channel between the [current user] and another
    /// [`User`]. This can also retrieve the channel if one already exists.
    ///
    /// [`User`]: ../model/struct.User.html
    /// [current user]: ../model/struct.CurrentUser.html
    pub fn create_direct_message_channel<U>(&self, user_id: U)
        -> Result<PrivateChannel> where U: Into<UserId> {
        let map = ObjectBuilder::new()
            .insert("recipient_id", user_id.into().0)
            .build();

        rest::create_private_channel(map)
    }

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// [`Message::react`] may be a more suited method of reacting in most
    /// cases.
    ///
    /// **Note**: Requires the [Add Reactions] permission, _if_ the current user
    /// is the first user to perform a react with a certain emoji.
    ///
    /// [`Emoji`]: ../model/struct.Emoji.html
    /// [`Message`]: ../model/struct.Message.html
    /// [`Message::react`]: ../model/struct.Message.html#method.react
    /// [Add Reactions]: ../model/permissions/constant.ADD_REACTIONS.html
    pub fn create_reaction<C, M, R>(&self,
                                    channel_id: C,
                                    message_id: M,
                                    reaction_type: R)
                                    -> Result<()>
                                    where C: Into<ChannelId>,
                                          M: Into<MessageId>,
                                          R: Into<ReactionType> {
        rest::create_reaction(channel_id.into().0,
                              message_id.into().0,
                              reaction_type.into())
    }

    pub fn create_role<F, G>(&self, guild_id: G, f: F) -> Result<Role>
        where F: FnOnce(EditRole) -> EditRole, G: Into<GuildId> {
        let id = guild_id.into().0;

        // The API only allows creating an empty role.
        let role = try!(rest::create_role(id));
        let map = f(EditRole::default()).0.build();

        rest::edit_role(id, role.id.0, map)
    }

    /// Deletes a [`Channel`] based on the Id given.
    ///
    /// If the channel being deleted is a [`GuildChannel`] (a [`Guild`]'s
    /// channel), then the [Manage Channels] permission is required.
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`GuildChannel`]: ../model/struct.GuildChannel.html
    /// [Manage Messages]: ../model/permissions/constant.MANAGE_CHANNELS.html
    pub fn delete_channel<C>(&self, channel_id: C) -> Result<Channel>
        where C: Into<ChannelId> {
        rest::delete_channel(channel_id.into().0)
    }

    /// Deletes an emoji in a [`Guild`] given its Id.
    ///
    /// **Note**: Requires the [Manage Emojis] permission.
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    /// [Manage Emojis]: ../model/permissions/constant.MANAGE_EMOJIS.html
    pub fn delete_emoji<E, G>(&self, guild_id: G, emoji_id: E) -> Result<()>
        where E: Into<EmojiId>, G: Into<GuildId> {
        rest::delete_emoji(guild_id.into().0, emoji_id.into().0)
    }

    /// Deletes a guild. You must be the guild owner to be able to delete it.
    ///
    /// Only a [`PartialGuild`] will be immediately returned.
    ///
    /// [`PartialGuild`]: ../model/struct.PartialGuild.html
    pub fn delete_guild<G: Into<GuildId>>(&self, guild_id: G)
        -> Result<PartialGuild> {
        rest::delete_guild(guild_id.into().0)
    }

    pub fn delete_integration<G, I>(&self, guild_id: G, integration_id: I)
        -> Result<()> where G: Into<GuildId>, I: Into<IntegrationId> {
        rest::delete_guild_integration(guild_id.into().0,
                                       integration_id.into().0)
    }

    /// Deletes the given invite.
    ///
    /// Refer to the documentation for [`Invite::delete`] for restrictions on
    /// deleting an invite.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required [permission].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Invite::delete`]: ../model/struct.Invite.html#method.delete
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn delete_invite(&self, invite: &str) -> Result<Invite> {
        let code = utils::parse_invite(invite);

        rest::delete_invite(code)
    }

    /// Deletes a [`Message`] given its Id.
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
    /// client.on_message(|context, message| {
    ///     context.delete_message(message);
    /// });
    /// ```
    ///
    /// (in practice, please do not do this)
    ///
    /// [`Message`]: ../model/struct.Message.html
    pub fn delete_message<C, M>(&self, channel_id: C, message_id: M)
        -> Result<()> where C: Into<ChannelId>, M: Into<MessageId> {
        rest::delete_message(channel_id.into().0, message_id.into().0)
    }

    pub fn delete_messages<C>(&self, channel_id: C, message_ids: &[MessageId])
        -> Result<()> where C: Into<ChannelId> {
        if self.login_type == LoginType::User {
            return Err(Error::Client(ClientError::InvalidOperationAsUser))
        }

        let ids: Vec<u64> = message_ids.into_iter()
            .map(|message_id| message_id.0)
            .collect();

        let map = ObjectBuilder::new()
            .insert("messages", ids)
            .build();

        rest::delete_messages(channel_id.into().0, map)
    }

    pub fn delete_note<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        let map = ObjectBuilder::new()
            .insert("note", "")
            .build();

        rest::edit_note(user_id.into().0, map)
    }

    pub fn delete_permission<C>(&self,
                                channel_id: C,
                                permission_type: PermissionOverwriteType)
                                -> Result<()> where C: Into<ChannelId> {
        let id = match permission_type {
            PermissionOverwriteType::Member(id) => id.0,
            PermissionOverwriteType::Role(id) => id.0,
        };

        rest::delete_permission(channel_id.into().0, id)
    }


    /// Deletes the given [`Reaction`], but only if the current user is the user
    /// who made the reaction or has permission to.
    ///
    /// **Note**: Requires the [`Manage Messages`] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: ../model/struct.Reaction.html
    /// [Manage Messages]: ../model/permissions/constant.MANAGE_MESSAGES.html
    pub fn delete_reaction<C, M, R>(&self,
                                    channel_id: C,
                                    message_id: M,
                                    user_id: Option<UserId>,
                                    reaction_type: R)
                                    -> Result<()>
                                    where C: Into<ChannelId>,
                                          M: Into<MessageId>,
                                          R: Into<ReactionType> {
        rest::delete_reaction(channel_id.into().0,
                              message_id.into().0,
                              user_id.map(|uid| uid.0),
                              reaction_type.into())
    }

    pub fn delete_role<G, R>(&self, guild_id: G, role_id: R) -> Result<()>
        where G: Into<GuildId>, R: Into<RoleId> {
        rest::delete_role(guild_id.into().0, role_id.into().0)
    }

    /// Sends a message to a user through a direct message channel. This is a
    /// channel that can only be accessed by you and the recipient.
    ///
    /// # Examples
    ///
    /// There are three ways to send a direct message to someone, the first
    /// being an unrelated, although equally helpful method.
    ///
    /// Sending a message via [`User::dm`]:
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    /// let _ = context.message.author.dm("Hello!");
    /// ```
    ///
    /// Sending a message to a `PrivateChannel`:
    ///
    /// ```rust,ignore
    /// assuming you are in a context
    /// let private_channel = context.create_private_channel(context.message.author.id);
    ///
    /// let _ = context.direct_message(private_channel, "Test!");
    /// ```
    ///
    /// Sending a message to a `PrivateChannel` given its ID:
    ///
    /// ```rust,ignore
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login_bot(&env::var("DISCORD_BOT_TOKEN").unwrap());
    ///
    /// client.on_message(|context, message| {
    ///     if message.content == "!pm-me" {
    ///         let channel = context.create_private_channel(message.author.id)
    ///             .unwrap();
    ///
    ///         let _ = channel.send_message("test!");
    ///     }
    /// });
    /// ```
    ///
    /// [`PrivateChannel`]: ../model/struct.PrivateChannel.html
    /// [`User::dm`]: ../model/struct.User.html#method.dm
    pub fn dm<C: Into<ChannelId>>(&self, target_id: C, content: &str)
        -> Result<Message> {
        self.send_message(target_id.into(), |m| m.content(content))
    }

    pub fn edit_channel<C, F>(&self, channel_id: C, f: F)
        -> Result<GuildChannel> where C: Into<ChannelId>,
                                       F: FnOnce(EditChannel) -> EditChannel {
        let channel_id = channel_id.into();

        let map = match try!(self.get_channel(channel_id)) {
            Channel::Guild(channel) => {
                let map = ObjectBuilder::new()
                    .insert("name", channel.name)
                    .insert("position", channel.position);

                match channel.kind {
                    ChannelType::Text => map.insert("topic", channel.topic),
                    ChannelType::Voice => {
                        map.insert("bitrate", channel.bitrate)
                            .insert("user_limit", channel.user_limit)
                    },
                    kind => return Err(Error::Client(ClientError::UnexpectedChannelType(kind))),
                }
            },
            Channel::Private(channel) => {
                return Err(Error::Client(ClientError::UnexpectedChannelType(channel.kind)));
            },
            Channel::Group(_group) => {
                return Err(Error::Client(ClientError::UnexpectedChannelType(ChannelType::Group)));
            },
        };

        let edited = f(EditChannel(map)).0.build();

        rest::edit_channel(channel_id.0, edited)
    }

    pub fn edit_emoji<E, G>(&self, guild_id: G, emoji_id: E, name: &str)
        -> Result<Emoji> where E: Into<EmojiId>, G: Into<GuildId> {
        let map = ObjectBuilder::new()
            .insert("name", name)
            .build();

        rest::edit_emoji(guild_id.into().0, emoji_id.into().0, map)
    }

    pub fn edit_guild<F, G>(&self, guild_id: G, f: F) -> Result<PartialGuild>
        where F: FnOnce(EditGuild) -> EditGuild, G: Into<GuildId> {
        let map = f(EditGuild::default()).0.build();

        rest::edit_guild(guild_id.into().0, map)
    }

    pub fn edit_member<F, G, U>(&self, guild_id: G, user_id: U, f: F)
        -> Result<()> where F: FnOnce(EditMember) -> EditMember,
                            G: Into<GuildId>,
                            U: Into<UserId> {
        let map = f(EditMember::default()).0.build();

        rest::edit_member(guild_id.into().0, user_id.into().0, map)
    }

    /// Edits the current user's nickname for the provided [`Guild`] via its Id.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// **Note**: Requires the [Change Nickname] permission.
    ///
    /// [`Guild`]: ../../model/struct.Guild.html
    /// [Change Nickname]: permissions/constant.CHANGE_NICKNAME.html
    #[inline]
    pub fn edit_nickname<G>(&self, guild_id: G, new_nickname: Option<&str>)
        -> Result<()> where G: Into<GuildId> {
        rest::edit_nickname(guild_id.into().0, new_nickname)
    }

    pub fn edit_profile<F: FnOnce(EditProfile) -> EditProfile>(&mut self, f: F)
        -> Result<CurrentUser> {
        let user = try!(rest::get_current_user());

        let mut map = ObjectBuilder::new()
            .insert("avatar", user.avatar)
            .insert("username", user.name);

        if let Some(email) = user.email.as_ref() {
            map = map.insert("email", email);
        }

        let edited = f(EditProfile(map)).0.build();

        rest::edit_profile(edited)
    }

    pub fn edit_role<F, G, R>(&self, guild_id: G, role_id: R, f: F)
        -> Result<Role> where F: FnOnce(EditRole) -> EditRole,
                              G: Into<GuildId>,
                              R: Into<GuildId> {
        let guild_id = guild_id.into();
        let role_id = role_id.into();

        feature_cache! {{
            let cache = CACHE.read().unwrap();

            let role = if let Some(role) = {
                cache.get_role(guild_id.0, role_id.0)
            } {
                role
            } else {
                return Err(Error::Client(ClientError::RecordNotFound));
            };

            let map = f(EditRole::new(role)).0.build();

            rest::edit_role(guild_id.0, role_id.0, map)
        } else {
            let map = f(EditRole::default()).0.build();

            rest::edit_role(guild_id.0, role_id.0, map)
        }}
    }

    /// Edit a message given its Id and the Id of the channel it belongs to.
    ///
    /// Pass an empty string (`""`) to `text` if you are editing a message with
    /// an embed but no content. Otherwise, `text` must be given.
    pub fn edit_message<C, F, M>(&self, channel_id: C, message_id: M, text: &str, f: F)
        -> Result<Message> where C: Into<ChannelId>,
                                 F: FnOnce(CreateEmbed) -> CreateEmbed,
                                 M: Into<MessageId> {
        let mut map = ObjectBuilder::new()
            .insert("content", text);

        let embed = f(CreateEmbed::default()).0;

        if embed.len() > 1 {
            map = map.insert("embed", Value::Object(embed));
        }

        rest::edit_message(channel_id.into().0, message_id.into().0, map.build())
    }

    pub fn edit_note<U: Into<UserId>>(&self, user_id: U, note: &str)
        -> Result<()> {
        let map = ObjectBuilder::new()
            .insert("note", note)
            .build();

        rest::edit_note(user_id.into().0, map)
    }

    pub fn get_bans<G: Into<GuildId>>(&self, guild_id: G) -> Result<Vec<Ban>> {
        rest::get_bans(guild_id.into().0)
    }

    pub fn get_channel_invites<C: Into<ChannelId>>(&self, channel_id: C)
        -> Result<Vec<RichInvite>> {
        rest::get_channel_invites(channel_id.into().0)
    }

    pub fn get_channel<C>(&self, channel_id: C) -> Result<Channel>
        where C: Into<ChannelId> {
        let channel_id = channel_id.into();

        feature_cache_enabled! {{
            if let Some(channel) = CACHE.read().unwrap().get_channel(channel_id) {
                return Ok(channel.clone_inner());
            }
        }}

        rest::get_channel(channel_id.0)
    }

    pub fn get_channels<G>(&self, guild_id: G)
        -> Result<HashMap<ChannelId, GuildChannel>> where G: Into<GuildId> {
        let guild_id = guild_id.into();

        feature_cache_enabled! {{
            let cache = CACHE.read().unwrap();

            if let Some(guild) = cache.get_guild(guild_id) {
                return Ok(guild.channels.clone());
            }
        }}

        let mut channels = HashMap::new();

        for channel in try!(rest::get_channels(guild_id.0)) {
            channels.insert(channel.id, channel);
        }

        Ok(channels)
    }

    pub fn get_emoji<E, G>(&self, guild_id: G, emoji_id: E) -> Result<Emoji>
        where E: Into<EmojiId>, G: Into<GuildId> {
        rest::get_emoji(guild_id.into().0, emoji_id.into().0)
    }

    pub fn get_emojis<G: Into<GuildId>>(&self, guild_id: G)
        -> Result<Vec<Emoji>> {
        rest::get_emojis(guild_id.into().0)
    }

    pub fn get_guild<G: Into<GuildId>>(&self, guild_id: G)
        -> Result<PartialGuild> {
        rest::get_guild(guild_id.into().0)
    }

    pub fn get_guild_invites<G>(&self, guild_id: G) -> Result<Vec<RichInvite>>
        where G: Into<GuildId> {
        rest::get_guild_invites(guild_id.into().0)
    }

    pub fn get_guild_prune_count<G>(&self, guild_id: G, days: u16)
        -> Result<GuildPrune> where G: Into<GuildId> {
        let map = ObjectBuilder::new()
            .insert("days", days)
            .build();

        rest::get_guild_prune_count(guild_id.into().0, map)
    }

    pub fn get_guilds(&self) -> Result<Vec<GuildInfo>> {
        rest::get_guilds()
    }

    pub fn get_integrations<G: Into<GuildId>>(&self, guild_id: G)
        -> Result<Vec<Integration>> {
        rest::get_guild_integrations(guild_id.into().0)
    }

    pub fn get_invite(&self, invite: &str) -> Result<Invite> {
        let code = utils::parse_invite(invite);

        rest::get_invite(code)
    }

    pub fn get_member<G, U>(&self, guild_id: G, user_id: U) -> Result<Member>
        where G: Into<GuildId>, U: Into<UserId> {
        let guild_id = guild_id.into();
        let user_id = user_id.into();

        feature_cache_enabled! {{
            let cache = CACHE.read().unwrap();

            if let Some(member) = cache.get_member(guild_id, user_id) {
                return Ok(member.clone());
            }
        }}

        rest::get_member(guild_id.0, user_id.0)
    }

    /// Retrieves a single [`Message`] from a [`Channel`].
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidOperationAsUser`] if this is a user.
    ///
    /// [`Channel`]: ../model/struct.Channel.html
    /// [`ClientError::InvalidOperationAsUser`]: ../enum.ClientError.html#variant.InvalidOperationAsUser
    /// [`Message`]: ../model/struct.Message.html
    /// [Read Message History]: ../model/permissions/constant.READ_MESSAGE_HISTORY.html
    pub fn get_message<C, M>(&self, channel_id: C, message_id: M)
        -> Result<Message> where C: Into<ChannelId>, M: Into<MessageId> {
        if self.login_type == LoginType::User {
            return Err(Error::Client(ClientError::InvalidOperationAsUser))
        }

        rest::get_message(channel_id.into().0, message_id.into().0)
    }

    pub fn get_messages<C, F>(&self, channel_id: C, f: F) -> Result<Vec<Message>>
        where C: Into<ChannelId>, F: FnOnce(GetMessages) -> GetMessages {
        let query = {
            let mut map = f(GetMessages::default()).0;
            let mut query = format!("?limit={}",
                                    map.remove("limit").unwrap_or(50));

            if let Some(after) = map.remove("after") {
                query.push_str("&after=");
                query.push_str(&after.to_string());
            }

            if let Some(around) = map.remove("around") {
                query.push_str("&around=");
                query.push_str(&around.to_string());
            }

            if let Some(before) = map.remove("before") {
                query.push_str("&before=");
                query.push_str(&before.to_string());
            }

            query
        };

        rest::get_messages(channel_id.into().0, &query)
    }

    /// Retrieves the list of [`User`]s who have reacted to a [`Message`] with a
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
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required [permissions].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Emoji`]: struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: struct.User.html
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    pub fn get_reaction_users<C, M, R, U>(&self,
                                          channel_id: C,
                                          message_id: M,
                                          reaction_type: R,
                                          limit: Option<u8>,
                                          after: Option<U>)
                                          -> Result<Vec<User>>
                                          where C: Into<ChannelId>,
                                                M: Into<MessageId>,
                                                R: Into<ReactionType>,
                                                U: Into<UserId> {
        let limit = limit.map(|x| if x > 100 { 100 } else { x }).unwrap_or(50);

        rest::get_reaction_users(channel_id.into().0,
                                 message_id.into().0,
                                 reaction_type.into(),
                                 limit,
                                 after.map(|u| u.into().0))
    }

    /// Kicks a [`Member`] from the specified [`Guild`] if they are in it.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`Member`]: ../model/struct.Member.html
    /// [Kick Members]: ../model/permissions/constant.KICK_MEMBERS.html
    pub fn kick_member<G, U>(&self, guild_id: G, user_id: U) -> Result<()>
        where G: Into<GuildId>, U: Into<UserId> {
        rest::kick_member(guild_id.into().0, user_id.into().0)
    }

    pub fn leave_guild<G: Into<GuildId>>(&self, guild_id: G)
        -> Result<PartialGuild> {
        rest::leave_guild(guild_id.into().0)
    }

    pub fn move_member<C, G, U>(&self, guild_id: G, user_id: U, channel_id: C)
        -> Result<()> where C: Into<ChannelId>,
                            G: Into<ChannelId>,
                            U: Into<ChannelId> {
        let map = ObjectBuilder::new()
            .insert("channel_id", channel_id.into().0)
            .build();

        rest::edit_member(guild_id.into().0, user_id.into().0, map)
    }

    /// Retrieves the list of [`Message`]s which are pinned to the specified
    /// [`Channel`].
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`Message`]: ../model/struct.Message.html
    pub fn get_pins<C>(&self, channel_id: C) -> Result<Vec<Message>>
        where C: Into<ChannelId> {
        rest::get_pins(channel_id.into().0)
    }

    pub fn pin<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        rest::pin_message(channel_id.into().0, message_id.into().0)
    }

    /// Sends a message with just the given message content in the channel that
    /// a message was received from.
    ///
    /// **Note**: This will only work when a [`Message`] is received.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// Returns a [`ClientError::NoChannelId`] when there is no [`ChannelId`]
    /// directly available.
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    /// [`ClientError::MessageTooLong`]: enum.ClientError.html#variant.MessageTooLong
    /// [`ClientError::NoChannelId`]: ../enum.ClientError.html#NoChannelId
    /// [`Message`]: ../model/struct.Message.html
    pub fn say(&self, content: &str) -> Result<Message> {
        if let Some(channel_id) = self.channel_id {
            self.send_message(channel_id, |m| m.content(content))
        } else {
            Err(Error::Client(ClientError::NoChannelId))
        }
    }

    /// Sends a file along with optional message contents. The filename _must_
    /// be specified.
    ///
    /// Pass an empty string to send no message contents.
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
    pub fn send_file<C, R>(&self,
                           channel_id: C,
                           content: &str,
                           file: R,
                           filename: &str)
                           -> Result<Message> where C: Into<ChannelId>,
                                                    R: Read {
        if let Some(length_over) = Message::overflow_length(content) {
            return Err(Error::Client(ClientError::MessageTooLong(length_over)));
        }

        rest::send_file(channel_id.into().0, content, file, filename)
    }

    /// Sends a message to a [`Channel`].
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Example
    ///
    /// Send a message with just the content `test`:
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    /// let _ = context.send_message(message.channel_id, |f| f.content("test"));
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
    /// ```rust,no_run
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
    /// fn ping(context: Context, message: Message, _arguments: Vec<String>) {
    ///     let cache = CACHE.read().unwrap();
    ///     let channel = cache.get_guild_channel(message.channel_id);
    ///
    ///     let _ = context.send_message(message.channel_id, |m| m
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
    /// }
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
    /// [author structure]: ../utils/builder/struct.CreateEmbedAuthor.html
    /// [field structure]: ../utils/builder/struct.CreateEmbedField.html
    pub fn send_message<C, F>(&self, channel_id: C, f: F) -> Result<Message>
        where C: Into<ChannelId>, F: FnOnce(CreateMessage) -> CreateMessage {
        let map = f(CreateMessage::default()).0;

        if let Some(content) = map.get(&"content".to_owned()) {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Client(ClientError::MessageTooLong(length_over)));
                }
            }
        }

        rest::send_message(channel_id.into().0, Value::Object(map))
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

    pub fn start_guild_prune<G>(&self, guild_id: G, days: u16)
        -> Result<GuildPrune> where G: Into<GuildId> {
        let map = ObjectBuilder::new()
            .insert("days", days)
            .build();

        rest::start_guild_prune(guild_id.into().0, map)
    }

    pub fn start_integration_sync<G, I>(&self, guild_id: G, integration_id: I)
        -> Result<()> where G: Into<GuildId>, I: Into<IntegrationId> {
        rest::start_integration_sync(guild_id.into().0, integration_id.into().0)
    }

    /// Unbans a [`User`] from a [`Guild`].
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`User`]: ../model/struct.User.html
    /// [Ban Members]: ../model/permissions/constant.BAN_MEMBERS.html
    pub fn unban<G, U>(&self, guild_id: G, user_id: U) -> Result<()>
        where G: Into<GuildId>, U: Into<UserId> {
        rest::remove_ban(guild_id.into().0, user_id.into().0)
    }

    pub fn unpin<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        rest::unpin_message(channel_id.into().0, message_id.into().0)
    }
}
