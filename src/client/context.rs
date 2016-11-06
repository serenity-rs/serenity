use serde_json::builder::ObjectBuilder;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use super::connection::Connection;
use super::{STATE, http};
use super::login_type::LoginType;
use ::builder::{
    CreateInvite,
    EditChannel,
    EditGuild,
    EditMember,
    EditProfile,
    EditRole,
    GetMessages
};
use ::model::*;
use ::prelude_internal::*;
use ::utils;

#[derive(Clone)]
pub struct Context {
    channel_id: Option<ChannelId>,
    pub connection: Arc<Mutex<Connection>>,
    login_type: LoginType,
}

impl Context {
    /// Create a new Context to be passed to an event handler.
    #[doc(hidden)]
    pub fn new(channel_id: Option<ChannelId>,
               connection: Arc<Mutex<Connection>>,
               login_type: LoginType) -> Context {
        Context {
            channel_id: channel_id,
            connection: connection,
            login_type: login_type,
        }
    }

    /// Accepts the given invite.
    ///
    /// Refer to the documentation for [`Invite::accept`] for restrictions on
    /// accepting an invite.
    ///
    /// [`Invite::accept`]: ../model/struct.Invite.html#method.accept
    pub fn accept_invite(&self, invite: &str) -> Result<Invite> {
        let code = utils::parse_invite(invite);

        http::accept_invite(code)
    }

    /// This is an alias of [`ack_message`].
    ///
    /// [`ack_message`]: #method.ack_message
    pub fn ack<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        self.ack_message(channel_id.into(), message_id.into())
    }

    /// Mark a message as being read in a channel. This will mark up to the
    /// given message as read. Any messages created after that message will not
    /// be marked as read.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidOperationAsBot`] if this is a bot.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../enum.ClientError.html#variant.InvalidOperationAsUser
    pub fn ack_message<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        if self.login_type == LoginType::User {
            return Err(Error::Client(ClientError::InvalidOperationAsUser))
        }

        http::ack_message(channel_id.into().0, message_id.into().0)
    }

    /// This is an alias of [`ban`].
    ///
    /// [`ban`]: #method.ban
    pub fn ban<G, U>(&self, guild_id: G, user_id: U, delete_message_days: u8)
        -> Result<()> where G: Into<GuildId>, U: Into<UserId> {
        self.ban_user(guild_id.into(), user_id.into(), delete_message_days)
    }

    /// Ban a [`User`] from a [`Guild`], removing their messages sent in the
    /// last X number of days.
    ///
    /// `0` days is equivilant to not removing any messages. Up to `7` days'
    /// worth of messages may be deleted.
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
    pub fn ban_user<G, U>(&self, guild_id: G, user_id: U, delete_message_days: u8)
        -> Result<()> where G: Into<GuildId>, U: Into<UserId> {
        if delete_message_days > 7 {
            return Err(Error::Client(ClientError::DeleteMessageDaysAmount(delete_message_days)));
        }

        http::ban_user(guild_id.into().0, user_id.into().0, delete_message_days)
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
        http::broadcast_typing(channel_id.into().0)
    }

    /// Creates a [`PublicChannel`] in the given [`Guild`].
    ///
    /// Requires that you have the [Manage Channels] permission.
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
    /// [`PublicChannel`]: ../model/struct.PublicChannel.html
    /// [Manage Channels]: ../model/permissions/constant.MANAGE_CHANNELS.html
    pub fn create_channel<G>(&self, guild_id: G, name: &str, kind: ChannelType)
        -> Result<Channel> where G: Into<GuildId> {
        let map = ObjectBuilder::new()
            .insert("name", name)
            .insert("type", kind.name())
            .build();

        http::create_channel(guild_id.into().0, map)
    }

    pub fn create_emoji<G>(&self, guild_id: G, name: &str, image: &str)
        -> Result<Emoji> where G: Into<GuildId> {
        let map = ObjectBuilder::new()
            .insert("name", name)
            .insert("image", image)
            .build();

        http::create_emoji(guild_id.into().0, map)
    }

    /// Creates a [`Guild`] with the data provided.
    ///
    /// # Examples
    ///
    /// Create a guild called `test` in the [US West region] with no icon:
    ///
    /// ```rust,ignore
    /// use serenity::model::Region;
    ///
    /// context.create_guild("test", Region::UsWest, None);
    /// ```
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    /// [US West region]: ../model/enum.Region.html#variant.UsWest
    pub fn create_guild(&self, name: &str, region: Region, icon: Option<&str>)
        -> Result<Guild> {
        let map = ObjectBuilder::new()
            .insert("icon", icon)
            .insert("name", name)
            .insert("region", region.name())
            .build();

        http::create_guild(map)
    }

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

        http::create_guild_integration(guild_id.into().0, integration_id.0, map)
    }

    pub fn create_invite<C, F>(&self, channel_id: C, f: F) -> Result<RichInvite>
        where C: Into<ChannelId>, F: FnOnce(CreateInvite) -> CreateInvite {
        let map = f(CreateInvite::default()).0.build();

        http::create_invite(channel_id.into().0, map)
    }

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

        http::create_permission(channel_id.into().0, id, map)
    }

    pub fn create_private_channel<U>(&self, user_id: U)
        -> Result<PrivateChannel> where U: Into<UserId> {
        let map = ObjectBuilder::new()
            .insert("recipient_id", user_id.into().0)
            .build();

        http::create_private_channel(map)
    }

    /// React to a [`Message`] with a custom [`Emoji`] or unicode character.
    ///
    /// **Note**: Requires the [Add Reactions] permission.
    ///
    /// [`Emoji`]: ../models/struct.Emoji.html
    /// [`Message`]: ../models/struct.Message.html
    /// [Add Reactions]: ../models/permissions/constant.ADD_REACTIONS.html
    pub fn create_reaction<C, M, R>(&self,
                                    channel_id: C,
                                    message_id: M,
                                    reaction_type: R)
                                    -> Result<()>
                                    where C: Into<ChannelId>,
                                          M: Into<MessageId>,
                                          R: Into<ReactionType> {
        http::create_reaction(channel_id.into().0,
                              message_id.into().0,
                              reaction_type.into())
    }

    pub fn create_role<F, G>(&self, guild_id: G, f: F) -> Result<Role>
        where F: FnOnce(EditRole) -> EditRole, G: Into<GuildId> {
        let id = guild_id.into().0;

        // The API only allows creating an empty role.
        let role = try!(http::create_role(id));
        let map = f(EditRole::default()).0.build();

        http::edit_role(id, role.id.0, map)
    }

    /// Deletes a [`Channel`] based on the Id given.
    ///
    /// If the channel being deleted is a [`PublicChannel`] (a [`Guild`]'s
    /// channel), then the [Manage Channels] permission is required.
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`PublicChannel`]: ../model/struct.PublicChannel.html
    /// [Manage Messages]: ../model/permissions/constant.MANAGE_CHANNELS.html
    pub fn delete_channel<C>(&self, channel_id: C) -> Result<Channel>
        where C: Into<ChannelId> {
        http::delete_channel(channel_id.into().0)
    }

    pub fn delete_emoji<E, G>(&self, guild_id: G, emoji_id: E) -> Result<()>
        where E: Into<EmojiId>, G: Into<GuildId> {
        http::delete_emoji(guild_id.into().0, emoji_id.into().0)
    }

    /// Deletes a [`Guild`]. You must be the guild owner to be able to delete
    /// the guild.
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    pub fn delete_guild<G: Into<GuildId>>(&self, guild_id: G) -> Result<Guild> {
        http::delete_guild(guild_id.into().0)
    }

    pub fn delete_integration<G, I>(&self, guild_id: G, integration_id: I)
        -> Result<()> where G: Into<GuildId>, I: Into<IntegrationId> {
        http::delete_guild_integration(guild_id.into().0,
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

        http::delete_invite(code)
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
        http::delete_message(channel_id.into().0, message_id.into().0)
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

        http::delete_messages(channel_id.into().0, map)
    }

    pub fn delete_note<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        let map = ObjectBuilder::new()
            .insert("note", "")
            .build();

        http::edit_note(user_id.into().0, map)
    }

    pub fn delete_permission<C>(&self,
                                channel_id: C,
                                permission_type: PermissionOverwriteType)
                                -> Result<()> where C: Into<ChannelId> {
        let id = match permission_type {
            PermissionOverwriteType::Member(id) => id.0,
            PermissionOverwriteType::Role(id) => id.0,
        };

        http::delete_permission(channel_id.into().0, id)
    }


    /// Deletes the given [`Reaction`], but only if the current user is the user
    /// who made the reaction or has permission to.
    ///
    /// **Note**: Requires the [`Manage Messages`] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: ../models/struct.Reaction.html
    /// [Manage Messages]: ../models/permissions/constant.MANAGE_MESSAGES.html
    pub fn delete_reaction<C, M, R>(&self,
                                    channel_id: C,
                                    message_id: M,
                                    user_id: Option<UserId>,
                                    reaction_type: R)
                                    -> Result<()>
                                    where C: Into<ChannelId>,
                                          M: Into<MessageId>,
                                          R: Into<ReactionType> {
        http::delete_reaction(channel_id.into().0,
                              message_id.into().0,
                              user_id.map(|uid| uid.0),
                              reaction_type.into())
    }

    pub fn delete_role<G, R>(&self, guild_id: G, role_id: R) -> Result<()>
        where G: Into<GuildId>, R: Into<RoleId> {
        http::delete_role(guild_id.into().0, role_id.into().0)
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
    pub fn direct_message<C>(&self, target_id: C, content: &str)
        -> Result<Message> where C: Into<ChannelId> {
        self.send_message(target_id.into(), content, "", false)
    }

    /// This is an alias of [`direct_message`].
    ///
    /// [`direct_message`]: #method.direct_message
    pub fn dm<C: Into<ChannelId>>(&self, target_id: C, content: &str)
        -> Result<Message> {
        self.direct_message(target_id.into(), content)
    }

    pub fn edit_channel<C, F>(&self, channel_id: C, f: F)
        -> Result<PublicChannel> where C: Into<ChannelId>,
                                       F: FnOnce(EditChannel) -> EditChannel {
        let channel_id = channel_id.into();

        let map = match try!(self.get_channel(channel_id)) {
            Channel::Public(channel) => {
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

        http::edit_channel(channel_id.0, edited)
    }

    pub fn edit_emoji<E, G>(&self, guild_id: G, emoji_id: E, name: &str)
        -> Result<Emoji> where E: Into<EmojiId>, G: Into<GuildId> {
        let map = ObjectBuilder::new()
            .insert("name", name)
            .build();

        http::edit_emoji(guild_id.into().0, emoji_id.into().0, map)
    }

    pub fn edit_guild<F, G>(&self, guild_id: G, f: F) -> Result<Guild>
        where F: FnOnce(EditGuild) -> EditGuild, G: Into<GuildId> {
        let map = f(EditGuild::default()).0.build();

        http::edit_guild(guild_id.into().0, map)
    }

    pub fn edit_member<F, G, U>(&self, guild_id: G, user_id: U, f: F)
        -> Result<()> where F: FnOnce(EditMember) -> EditMember,
                            G: Into<GuildId>,
                            U: Into<UserId> {
        let map = f(EditMember::default()).0.build();

        http::edit_member(guild_id.into().0, user_id.into().0, map)
    }

    pub fn edit_profile<F: FnOnce(EditProfile) -> EditProfile>(&mut self, f: F)
        -> Result<CurrentUser> {
        let user = try!(http::get_current_user());

        let mut map = ObjectBuilder::new()
            .insert("avatar", user.avatar)
            .insert("username", user.name);

        if let Some(email) = user.email.as_ref() {
            map = map.insert("email", email);
        }

        let edited = f(EditProfile(map)).0.build();

        http::edit_profile(edited)
    }

    pub fn edit_role<F, G, R>(&self, guild_id: G, role_id: R, f: F)
        -> Result<Role> where F: FnOnce(EditRole) -> EditRole,
                              G: Into<GuildId>,
                              R: Into<GuildId> {
        let guild_id = guild_id.into();
        let role_id = role_id.into();

        let map = {
            let state = STATE.lock().unwrap();

            let role = if let Some(role) = {
                state.find_role(guild_id.0, role_id.0)
            } {
                role
            } else {
                return Err(Error::Client(ClientError::RecordNotFound));
            };

            f(EditRole::new(role)).0.build()
        };

        http::edit_role(guild_id.0, role_id.0, map)
    }

    pub fn edit_message<C, M>(&self, channel_id: C, message_id: M, text: &str)
        -> Result<Message> where C: Into<ChannelId>, M: Into<MessageId> {
        let map = ObjectBuilder::new()
            .insert("content", text)
            .build();

        http::edit_message(channel_id.into().0, message_id.into().0, map)
    }

    pub fn edit_note<U: Into<UserId>>(&self, user_id: U, note: &str)
        -> Result<()> {
        let map = ObjectBuilder::new()
            .insert("note", note)
            .build();

        http::edit_note(user_id.into().0, map)
    }

    pub fn get_application_info(&self) -> Result<CurrentApplicationInfo> {
        http::get_application_info()
    }

    pub fn get_applications(&self) -> Result<Vec<ApplicationInfo>> {
        http::get_applications()
    }

    pub fn get_bans<G: Into<GuildId>>(&self, guild_id: G) -> Result<Vec<Ban>> {
        http::get_bans(guild_id.into().0)
    }

    pub fn get_channel_invites<C: Into<ChannelId>>(&self, channel_id: C)
        -> Result<Vec<RichInvite>> {
        http::get_channel_invites(channel_id.into().0)
    }

    pub fn get_channel<C>(&self, channel_id: C) -> Result<Channel>
        where C: Into<ChannelId> {
        let channel_id = channel_id.into();

        if let Some(channel) = STATE.lock().unwrap().find_channel(channel_id) {
            return Ok(channel.clone())
        }

        http::get_channel(channel_id.0)
    }

    pub fn get_channels<G>(&self, guild_id: G)
        -> Result<HashMap<ChannelId, PublicChannel>> where G: Into<GuildId> {
        let guild_id = guild_id.into();

        {
            let state = STATE.lock().unwrap();

            if let Some(guild) = state.find_guild(guild_id) {
                return Ok(guild.channels.clone());
            }
        }

        let mut channels = HashMap::new();

        for channel in try!(http::get_channels(guild_id.0)) {
            channels.insert(channel.id, channel);
        }

        Ok(channels)
    }

    pub fn get_emoji<E, G>(&self, guild_id: G, emoji_id: E) -> Result<Emoji>
        where E: Into<EmojiId>, G: Into<GuildId> {
        http::get_emoji(guild_id.into().0, emoji_id.into().0)
    }

    pub fn get_emojis<G: Into<GuildId>>(&self, guild_id: G)
        -> Result<Vec<Emoji>> {
        http::get_emojis(guild_id.into().0)
    }

    pub fn get_guild<G: Into<GuildId>>(&self, guild_id: G) -> Result<Guild> {
        http::get_guild(guild_id.into().0)
    }

    pub fn get_guild_invites<G>(&self, guild_id: G) -> Result<Vec<RichInvite>>
        where G: Into<GuildId> {
        http::get_guild_invites(guild_id.into().0)
    }

    pub fn get_guild_prune_count<G>(&self, guild_id: G, days: u16)
        -> Result<GuildPrune> where G: Into<GuildId> {
        let map = ObjectBuilder::new()
            .insert("days", days)
            .build();

        http::get_guild_prune_count(guild_id.into().0, map)
    }

    pub fn get_guilds(&self) -> Result<Vec<GuildInfo>> {
        http::get_guilds()
    }

    pub fn get_integrations<G: Into<GuildId>>(&self, guild_id: G)
        -> Result<Vec<Integration>> {
        http::get_guild_integrations(guild_id.into().0)
    }

    pub fn get_invite(&self, invite: &str) -> Result<Invite> {
        let code = utils::parse_invite(invite);

        http::get_invite(code)
    }

    pub fn get_member<G, U>(&self, guild_id: G, user_id: U) -> Result<Member>
        where G: Into<GuildId>, U: Into<UserId> {
        let guild_id = guild_id.into();
        let user_id = user_id.into();

        {
            let state = STATE.lock().unwrap();

            if let Some(member) = state.find_member(guild_id, user_id) {
                return Ok(member.clone());
            }
        }

        http::get_member(guild_id.0, user_id.0)
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

        http::get_message(channel_id.into().0, message_id.into().0)
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

        http::get_messages(channel_id.into().0, &query)
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

        http::get_reaction_users(channel_id.into().0,
                                 message_id.into().0,
                                 reaction_type.into(),
                                 limit,
                                 after.map(|u| u.into().0))
    }

    pub fn get_voice_regions(&self) -> Result<Vec<VoiceRegion>> {
        http::get_voice_regions()
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
        http::kick_member(guild_id.into().0, user_id.into().0)
    }

    pub fn leave_guild<G: Into<GuildId>>(&self, guild_id: G) -> Result<Guild> {
        http::leave_guild(guild_id.into().0)
    }

    pub fn move_member<C, G, U>(&self, guild_id: G, user_id: U, channel_id: C)
        -> Result<()> where C: Into<ChannelId>,
                            G: Into<ChannelId>,
                            U: Into<ChannelId> {
        let map = ObjectBuilder::new()
            .insert("channel_id", channel_id.into().0)
            .build();

        http::edit_member(guild_id.into().0, user_id.into().0, map)
    }

    /// This is an alias of [`get_pins`].
    ///
    /// [`get_pins`]: #method.get_pins
    pub fn pins<C>(&self, channel_id: C) -> Result<Vec<Message>>
        where C: Into<ChannelId> {
        self.get_pins(channel_id.into())
    }

    /// Retrieves the list of [`Message`]s which are pinned to the specified
    /// [`Channel`].
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    /// [`Message`]: ../model/struct.Message.html
    pub fn get_pins<C>(&self, channel_id: C) -> Result<Vec<Message>>
        where C: Into<ChannelId> {
        http::get_pins(channel_id.into().0)
    }

    /// This is an alias of [`pin_message`].
    ///
    /// [`pin_message`]: #method.pin_message
    pub fn pin<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        self.pin_message(channel_id.into(), message_id.into())
    }

    pub fn pin_message<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        http::pin_message(channel_id.into().0, message_id.into().0)
    }

    /// This is an alias of [`direct_message`].
    ///
    /// [`direct_message`]: #method.direct_message
    pub fn pm<C: Into<ChannelId>>(&self, target_id: C, content: &str)
        -> Result<Message> {
        self.direct_message(target_id.into(), content)
    }

    /// Unbans a [`User`] from a [`Guild`].
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`User`]: ../model/struct.User.html
    /// [Ban Members]: ../model/permissions/constant.BAN_MEMBERS.html
    pub fn remove_ban<G, U>(&self, guild_id: G, user_id: U) -> Result<()>
        where G: Into<GuildId>, U: Into<UserId> {
        http::remove_ban(guild_id.into().0, user_id.into().0)
    }

    /// Sends a message with just the given message content in the channel that
    /// a message was received from.
    ///
    /// **Note**: This will only work when a [`Message`] is received.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::NoChannelId`] when there is no [`ChannelId`]
    /// directly available.
    ///
    /// [`ChannelId`]: ../../models/struct.ChannelId.html
    /// [`ClientError::NoChannelId`]: ../enum.ClientError.html#NoChannelId
    /// [`Message`]: ../model/struct.Message.html
    pub fn say(&self, text: &str) -> Result<Message> {
        if let Some(channel_id) = self.channel_id {
            self.send_message(channel_id, text, "", false)
        } else {
            Err(Error::Client(ClientError::NoChannelId))
        }
    }

    /// This is an alias of [`send_message`].
    ///
    /// [`send_message`]: #method.send_message
    pub fn send<C>(&self, channel_id: C, content: &str, nonce: &str, tts: bool)
        -> Result<Message> where C: Into<ChannelId> {
        self.send_message(channel_id.into(),
                          content,
                          nonce,
                          tts)
    }

    pub fn send_file<C, R>(&self,
                           channel_id: C,
                           content: &str,
                           file: R,
                           filename: &str)
                           -> Result<Message> where C: Into<ChannelId>,
                                                    R: Read {
        http::send_file(channel_id.into().0, content, file, filename)
    }

    /// Sends a message to a [`Channel`].
    ///
    /// Note that often a nonce is not required and can be omitted in most
    /// situations.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // assuming you are in a context
    /// let _ = context.send_message(message.channel_id, "Hello!", "", false);
    /// ```
    ///
    /// [`Channel`]: ../model/enum.Channel.html
    pub fn send_message<C>(&self, channel_id: C, content: &str, nonce: &str, tts: bool)
        -> Result<Message> where C: Into<ChannelId> {
        let map = ObjectBuilder::new()
            .insert("content", content)
            .insert("nonce", nonce)
            .insert("tts", tts)
            .build();

        http::send_message(channel_id.into().0, map)
    }

    pub fn set_game(&self, game: Option<Game>) {
        self.connection.lock()
            .unwrap()
            .set_presence(game, OnlineStatus::Online, false);
    }

    pub fn set_presence(&self,
                        game: Option<Game>,
                        status: OnlineStatus,
                        afk: bool) {
        self.connection.lock()
            .unwrap()
            .set_presence(game, status, afk)
    }

    pub fn start_guild_prune<G>(&self, guild_id: G, days: u16)
        -> Result<GuildPrune> where G: Into<GuildId> {
        let map = ObjectBuilder::new()
            .insert("days", days)
            .build();

        http::start_guild_prune(guild_id.into().0, map)
    }

    pub fn start_integration_sync<G, I>(&self, guild_id: G, integration_id: I)
        -> Result<()> where G: Into<GuildId>, I: Into<IntegrationId> {
        http::start_integration_sync(guild_id.into().0, integration_id.into().0)
    }

    /// This is an alias of [`broadcast_typing`].
    ///
    /// [`broadcast_typing`]: #method.broadcast_typing
    pub fn typing<C>(&self, channel_id: C) -> Result<()>
        where C: Into<ChannelId> {
        self.broadcast_typing(channel_id.into().0)
    }

    /// This is an alias of [`remove_ban`].
    ///
    /// [`#method.remove_ban`]: #method.remove_ban
    pub fn unban<G, U>(&self, guild_id: G, user_id: U) -> Result<()>
        where G: Into<GuildId>, U: Into<UserId> {
        self.remove_ban(guild_id.into().0, user_id.into().0)
    }

    /// This is an alias of [`unpin_message`].
    ///
    /// [`unpin_message`]: #method.unpin_message
    pub fn unpin<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        self.unpin_message(channel_id.into().0, message_id.into().0)
    }

    pub fn unpin_message<C, M>(&self, channel_id: C, message_id: M)
        -> Result<()> where C: Into<ChannelId>, M: Into<MessageId> {
        http::unpin_message(channel_id.into().0, message_id.into().0)
    }
}
