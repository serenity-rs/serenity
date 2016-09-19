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
use ::prelude::*;
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

    pub fn accept_invite(&self, invite: &str) -> Result<Invite> {
        let code = utils::parse_invite(invite);

        http::accept_invite(code)
    }

    /// This is an alias of [ack_message](#method.ack_message).
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
    /// Returns a
    /// [ClientError::InvalidOperationAsBot](../enum.ClientError.html#InvalidOperationAsUser.v)
    /// if this is a bot.
    pub fn ack_message<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        if self.login_type == LoginType::User {
            return Err(Error::Client(ClientError::InvalidOperationAsUser))
        }

        http::ack_message(channel_id.into().0, message_id.into().0)
    }

    /// This is an alias of [ban](#method.ban).
    pub fn ban<G, U>(&self, guild_id: G, user_id: U, delete_message_days: u8)
        -> Result<()> where G: Into<GuildId>, U: Into<UserId> {
        self.ban_user(guild_id.into(), user_id.into(), delete_message_days)
    }

    /// Ban a user from a guild, removing their messages sent in the last X
    /// number of days.
    ///
    /// 0 days is equivilant to not removing any messages. Up to 7 days' worth
    /// of messages may be deleted.
    ///
    /// Requires that you have the
    /// [Ban Members](../model/permissions/constant.BAN_MEMBERS.html)
    /// permission.
    ///
    /// # Examples
    ///
    /// Ban the user that sent a message for 7 days:
    ///
    /// ```rust,ignore
    /// context.ban_user(context.guild_id, context.message.author, 7);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a
    /// [ClientError::DeleteMessageDaysAmount](./enum.ClientError.html#DeleteMessageDaysAmount.v)
    /// if the number of days given is over the maximum allowed.
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
    /// context.broadcast_typing(context.channel_id);
    /// ```
    pub fn broadcast_typing<C>(&self, channel_id: C) -> Result<()>
        where C: Into<ChannelId> {
        http::broadcast_typing(channel_id.into().0)
    }

    /// Creates a [PublicChannel](../model/struct.PublicChannel.html) in the
    /// given [Guild](../model/struct.Guild.html).
    ///
    /// Requires that you have the
    /// [Manage Channels](../model/permissions/constant.MANAGE_CHANNELS.html)
    /// permission.
    ///
    /// # Examples
    ///
    /// Create a voice channel in a guild with the name "test":
    ///
    /// ```rust,ignore
    /// use serenity::model::ChannelType;
    ///
    /// context.create_channel(context.guild_id, "test", ChannelType::Voice);
    /// ```
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

    /// Creates a [Guild](../model/struct.Guild.html) with the data provided.
    ///
    /// # Examples
    ///
    /// Create a guild called "test" in the US West region with no icon:
    ///
    /// ```rust,ignore
    /// use serenity::model::Region;
    ///
    /// context.create_guild("test", Region::UsWest, None);
    /// ```
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

    pub fn create_role<F, G>(&self, guild_id: G, f: F) -> Result<Role>
        where F: FnOnce(EditRole) -> EditRole, G: Into<GuildId> {
        let id = guild_id.into().0;

        // The API only allows creating an empty role.
        let role = try!(http::create_role(id));
        let map = f(EditRole::default()).0.build();

        http::edit_role(id, role.id.0, map)
    }

    /// Deletes a [Channel](../model/enum.Channel.html) based on the id given.
    ///
    /// If the channel being deleted is a
    /// [PublicChannel](../model/struct.PublicChannel.html) (a guild's channel),
    /// then the
    /// [Manage Channels](../model/permissions/constant.MANAGE_CHANNELS.html)
    /// permission is required.
    pub fn delete_channel<C>(&self, channel_id: C) -> Result<Channel>
        where C: Into<ChannelId> {
        http::delete_channel(channel_id.into().0)
    }

    pub fn delete_emoji<E, G>(&self, guild_id: G, emoji_id: E) -> Result<()>
        where E: Into<EmojiId>, G: Into<GuildId> {
        http::delete_emoji(guild_id.into().0, emoji_id.into().0)
    }

    /// Deletes a [Guild](../model/struct.Guild.html). You must be the guild
    /// owner to be able to delete the guild.
    pub fn delete_guild<G: Into<GuildId>>(&self, guild_id: G) -> Result<Guild> {
        http::delete_guild(guild_id.into().0)
    }

    pub fn delete_integration<G, I>(&self, guild_id: G, integration_id: I)
        -> Result<()> where G: Into<GuildId>, I: Into<IntegrationId> {
        http::delete_guild_integration(guild_id.into().0,
                                       integration_id.into().0)
    }

    /*
    pub fn delete_invite(&self, invite: &str) -> Result<Invite> {
        let code = utils::parse_invite(invite);

        http::delete_invite(code)
    }
    */

    /// Deletes a [Message](../model/struct.Message.html) given its ID.
    ///
    /// # Examples
    ///
    /// Deleting a message that was received by its ID:
    ///
    /// ```rust,ignore
    /// context.delete_message(context.message.id);
    /// ```
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

    pub fn delete_role<G, R>(&self, guild_id: G, role_id: R) -> Result<()>
        where G: Into<GuildId>, R: Into<RoleId> {
        http::delete_role(guild_id.into().0, role_id.into().0)
    }

    /// Sends a message to a user through a direct message channel. This is a
    /// channel that can only be accessed by you and the recipient.
    ///
    /// # Examples
    ///
    /// There are three ways to send a direct message to someone, the first being
    /// an unrelated, although equally helpful method.
    ///
    /// Sending a message via a [User](../../model/struct.User.html):
    ///
    /// ```rust,ignore
    /// context.message.author.dm("Hello!");
    /// ```
    ///
    /// Sending a message to a PrivateChannel:
    ///
    /// ```rust,ignore
    /// let private_channel = context.create_private_channel(context.message.author.id);
    ///
    /// context.direct_message(private_channel, "Test!");
    /// ```
    ///
    /// Sending a message to a PrivateChannel given its ID:
    ///
    /// ```rust,ignore
    /// let private_channel = context.create_private_channel(context.message.author.id);
    ///
    /// context.direct_message(private_channel.id, "Test!");
    /// ```
    pub fn direct_message<C>(&self, target_id: C, content: &str)
        -> Result<Message> where C: Into<ChannelId> {
        self.send_message(target_id.into(), content, "", false)
    }

    /// This is an alias of [direct_message](#method.direct_message).
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

    /// Retrieves a single [Message](../../model/struct.Message.html) from a
    /// [Channel](../../model/struct.Channel.html).
    ///
    /// Requires the
    /// [Read Message History](../../model/permissions/constant.READ_MESSAGE_HISTORY.html)
    /// permission.
    ///
    /// # Errors
    ///
    /// Returns a
    /// [ClientError::InvalidOperationAsUser](../enum.ClientError.html#InvalidOperationAsUser.v)
    /// if this is a user.
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

    pub fn get_voice_regions(&self) -> Result<Vec<VoiceRegion>> {
        http::get_voice_regions()
    }

    /// Kicks a [Member](../../model/struct.Member.html) from the specified
    /// [Guild](../../model/struct.Guild.html) if they are in it.
    ///
    /// Requires the
    /// [Kick Members](../../model/permissions/constant.KICK_MEMBERS.html)
    /// permission.
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

    /// This is an alias of [get_pins](#method.get_pins).
    pub fn pins<C>(&self, channel_id: C) -> Result<Vec<Message>>
        where C: Into<ChannelId> {
        self.get_pins(channel_id.into())
    }

    /// Retrieves the list of [Message](../../model/struct.Message.html)s which
    /// are pinned to the specified [Channel](../../model/enum.Channel.html).
    pub fn get_pins<C>(&self, channel_id: C) -> Result<Vec<Message>>
        where C: Into<ChannelId> {
        http::get_pins(channel_id.into().0)
    }

    /// This is an alias of [pin_message](#method.pin_message).
    pub fn pin<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        self.pin_message(channel_id.into(), message_id.into())
    }

    pub fn pin_message<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        http::pin_message(channel_id.into().0, message_id.into().0)
    }

    /// This is an alias of [direct_message](#method.direct_message).
    pub fn pm<C: Into<ChannelId>>(&self, target_id: C, content: &str)
        -> Result<Message> {
        self.direct_message(target_id.into(), content)
    }

    /// Unbans a [User](../../model/struct.User.html) from a guild.
    ///
    /// Requires the
    /// [Ban Members](../../model/permissions/constant.BAN_MEMBERS.html)
    /// permission.
    pub fn remove_ban<G, U>(&self, guild_id: G, user_id: U) -> Result<()>
        where G: Into<GuildId>, U: Into<UserId> {
        http::remove_ban(guild_id.into().0, user_id.into().0)
    }

    /// Sends a message with just the given message content in the channel that
    /// a message was received from.
    ///
    /// **Note**: This will only work when a Message is received.
    ///
    /// # Errors
    ///
    /// Returns a
    /// [ClientError::NoChannelId](../enum.ClientError.html#NoChannelId) when
    /// there is no [ChannelId](../../models/struct.ChannelId.html) directly
    /// available.
    pub fn say(&self, text: &str) -> Result<Message> {
        if let Some(channel_id) = self.channel_id {
            self.send_message(channel_id, text, "", false)
        } else {
            Err(Error::Client(ClientError::NoChannelId))
        }
    }

    /// This is an alias of [send_message](#method.send_message).
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

    /// Sends a message to a [Channel](../../model/enum.Channel.html).
    ///
    /// Note that often a nonce is not required and can be omitted in most
    /// situations.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let _ = context.send_message(message.channel_id, "Hello!", "", false);
    /// ```
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
        self.connection.lock().unwrap().set_game(game)
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

    /// This is an alias of [broadcast_typing](#method.broadcast_typing).
    pub fn typing<C>(&self, channel_id: C) -> Result<()>
        where C: Into<ChannelId> {
        self.broadcast_typing(channel_id.into().0)
    }

    /// This is an alias of [remove_ban](#method.remove_ban).
    pub fn unban<G, U>(&self, guild_id: G, user_id: U) -> Result<()>
        where G: Into<GuildId>, U: Into<UserId> {
        self.remove_ban(guild_id.into().0, user_id.into().0)
    }

    /// This is an alias of [unpin_message](#method.unpin_message).
    pub fn unpin<C, M>(&self, channel_id: C, message_id: M) -> Result<()>
        where C: Into<ChannelId>, M: Into<MessageId> {
        self.unpin_message(channel_id.into().0, message_id.into().0)
    }

    pub fn unpin_message<C, M>(&self, channel_id: C, message_id: M)
        -> Result<()> where C: Into<ChannelId>, M: Into<MessageId> {
        http::unpin_message(channel_id.into().0, message_id.into().0)
    }
}
