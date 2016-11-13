use hyper::Client as HyperClient;
use serde_json::builder::ObjectBuilder;
use std::borrow::Cow;
use std::fmt::{self, Write};
use std::fs::File;
use std::io::{Read, Write as IoWrite};
use std::mem;
use std::path::{Path, PathBuf};
use super::utils::{
    decode_id,
    into_map,
    into_string,
    opt,
    remove,
    warn_field,
};
use super::*;
use super::utils;
use ::utils::builder::{CreateEmbed, CreateInvite, EditChannel};
use ::client::{STATE, http};
use ::constants;
use ::prelude_internal::*;
use ::utils::decode_array;

impl Attachment {
    /// If this attachment is an image, then a tuple of the width and height
    /// in pixels is returned.
    pub fn dimensions(&self) -> Option<(u64, u64)> {
        if let (Some(width), Some(height)) = (self.width, self.height) {
            Some((width, height))
        } else {
            None
        }
    }

    /// Downloads the attachment, returning back a vector of bytes.
    ///
    /// # Examples
    ///
    /// Download all of the attachments associated with a [`Message`]:
    ///
    /// ```rust,no_run
    /// use serenity::Client;
    /// use std::env;
    /// use std::fs::File;
    /// use std::io::Write;
    /// use std::path::Path;
    ///
    /// let token = env::var("DISCORD_TOKEN").expect("token in environment");
    /// let mut client = Client::login_bot(&token);
    ///
    /// client.on_message(|context, message| {
    ///     for attachment in message.attachments {
    ///         let content = match attachment.download() {
    ///             Ok(content) => content,
    ///             Err(why) => {
    ///                 println!("Error downloading attachment: {:?}", why);
    ///                 let _ = context.say("Error downloading attachment");
    ///
    ///                 return;
    ///             },
    ///         };
    ///
    ///         let mut file = match File::create(&attachment.filename) {
    ///             Ok(file) => file,
    ///             Err(why) => {
    ///                 println!("Error creating file: {:?}", why);
    ///                 let _ = context.say("Error creating file");
    ///
    ///                 return;
    ///             },
    ///         };
    ///
    ///         if let Err(why) = file.write(&content) {
    ///             println!("Error writing to file: {:?}", why);
    ///
    ///             return;
    ///         }
    ///
    ///         let _ = context.say(&format!("Saved {:?}", attachment.filename));
    ///     }
    /// });
    ///
    /// client.on_ready(|_context, ready| {
    ///     println!("{} is connected!", ready.user.name);
    /// });
    ///
    /// let _ = client.start();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Io`] when there is a problem reading the contents
    /// of the HTTP response.
    ///
    /// Returns an [`Error::Hyper`] when there is a problem retrieving the
    /// attachment.
    ///
    /// [`Error::Hyper`]: ../enum.Error.html#variant.Hyper
    /// [`Error::Io`]: ../enum.Error.html#variant.Io
    /// [`Message`]: struct.Message.html
    pub fn download(&self) -> Result<Vec<u8>> {
        let hyper = HyperClient::new();
        let mut response = try!(hyper.get(&self.url).send());

        let mut bytes = vec![];
        try!(response.read_to_end(&mut bytes));

        Ok(bytes)
    }

    /// Downloads the attachment, saving it to the provided directory path.
    /// Returns a path to the saved file.
    ///
    /// # Examples
    ///
    /// Download all of the attachments associated with a [`Message`] to a
    /// given folder:
    ///
    /// ```rust,no_run
    /// use serenity::Client;
    /// use std::env;
    /// use std::fs;
    ///
    /// // Make sure that the directory to store images in exists.
    /// fs::create_dir_all("./attachment_downloads")
    ///     .expect("err making directory");
    ///
    /// let token = env::var("DISCORD_TOKEN").expect("token in environment");
    /// let mut client = Client::login_bot(&token);
    ///
    /// client.on_message(|context, message| {
    ///     for attachment in message.attachments {
    ///         let dir = "./attachment_downloads";
    ///
    ///         let _ = match attachment.download_to_directory(dir) {
    ///             Ok(_saved_filepath) => {
    ///                 context.say(&format!("Saved {:?}", attachment.filename))
    ///             },
    ///             Err(why) => {
    ///                 println!("Error saving attachment: {:?}", why);
    ///                 context.say("Error saving attachment")
    ///             },
    ///         };
    ///     }
    /// });
    ///
    /// client.on_ready(|_context, ready| {
    ///     println!("{} is connected!", ready.user.name);
    /// });
    ///
    /// let _ = client.start();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Io`] when there is a problem reading the contents of
    /// the HTTP response, creating the file, or writing to the file.
    ///
    /// Returns an [`Error::Hyper`] when there is a problem retrieving the
    /// attachment.
    ///
    /// [`Error::Hyper`]: ../enum.Error.html#variant.Hyper
    /// [`Error::Io`]: ../enum.Error.html#variant.Io
    /// [`Message`]: struct.Message.html
    pub fn download_to_directory<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let bytes = try!(self.download());

        let filepath: PathBuf = path.as_ref().join(&self.filename);
        let mut file = try!(File::create(&filepath));
        try!(file.write(&bytes));

        Ok(filepath)
    }
}

impl Channel {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Channel> {
        let map = try!(into_map(value));
        match req!(map.get("type").and_then(|x| x.as_u64())) {
            0 | 2 => PublicChannel::decode(Value::Object(map))
                .map(Channel::Public),
            1 => PrivateChannel::decode(Value::Object(map))
                .map(Channel::Private),
            3 => Group::decode(Value::Object(map))
                .map(Channel::Group),
            other => Err(Error::Decode("Expected value Channel type",
                                       Value::U64(other))),
        }
    }

    /// Deletes the inner channel.
    ///
    /// **Note**: There is no real function as _deleting_ a [`Group`]. The
    /// closest functionality is leaving it.
    ///
    /// [`Group`]: struct.Group.html
    pub fn delete(&self) -> Result<()> {
        match *self {
            Channel::Group(ref group) => {
                let _ = try!(group.leave());
            },
            Channel::Private(ref private_channel) => {
                let _ = try!(private_channel.delete());
            },
            Channel::Public(ref public_channel) => {
                let _ = try!(public_channel.delete());
            },
        }

        Ok(())
    }

    /// Retrieves the Id of the inner [`Group`], [`PublicChannel`], or
    /// [`PrivateChannel`].
    ///
    /// [`Group`]: struct.Group.html
    /// [`PublicChannel`]: struct.PublicChannel.html
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    pub fn id(&self) -> ChannelId {
        match *self {
            Channel::Group(ref group) => group.channel_id,
            Channel::Private(ref channel) => channel.id,
            Channel::Public(ref channel) => channel.id,
        }
    }
}

impl fmt::Display for Channel {
    /// Formats the channel into a "mentioned" string.
    ///
    /// This will return a different format for each type of channel:
    ///
    /// - [`Group`]s: the generated name retrievable via [`Group::name`];
    /// - [`PrivateChannel`]s: the recipient's name;
    /// - [`PublicChannel`]s: a string mentioning the channel that users who can
    /// see the channel can click on.
    ///
    /// [`Group`]: struct.Group.html
    /// [`Group::name`]: struct.Group.html#method.name
    /// [`PublicChannel`]: struct.PublicChannel.html
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match *self {
            Channel::Group(ref group) => group.name().to_owned(),
            Channel::Private(ref channel) => Cow::Owned(channel.recipient.name.clone()),
            Channel::Public(ref channel) => Cow::Owned(format!("{}", channel)),
        };

        fmt::Display::fmt(&out, f)
    }
}

impl Embed {
    /// Creates a fake Embed, giving back a `serde_json` map.
    ///
    /// This should only be useful in conjunction with [`Webhook::execute`].
    ///
    /// [`Webhook::execute`]: struct.Webhook.html
    #[inline(always)]
    pub fn fake<F>(f: F) -> Value where F: FnOnce(CreateEmbed) -> CreateEmbed {
        f(CreateEmbed::default()).0.build()
    }
}

impl Group {
    /// Adds the given user to the group. If the user is already in the group,
    /// then nothing is done.
    ///
    /// **Note**: Groups have a limit of 10 recipients, including the current
    /// user.
    pub fn add_recipient<U: Into<UserId>>(&self, user: U) -> Result<()> {
        let user = user.into();

        // If the group already contains the recipient, do nothing.
        if self.recipients.contains_key(&user) {
            return Ok(());
        }

        http::add_group_recipient(self.channel_id.0, user.0)
    }

    /// Broadcasts that the current user is typing in the group.
    pub fn broadcast_typing(&self) -> Result<()> {
        http::broadcast_typing(self.channel_id.0)
    }

    /// Deletes multiple messages in the group.
    ///
    /// Refer to
    /// [`Context::delete_messages`] for more information.
    ///
    /// **Note**: Only 2 to 100 messages may be deleted in a single request.
    ///
    /// # Errors
    ///
    /// Returns a
    /// [`ClientError::DeleteMessageDaysAmount`] if the number of messages to
    /// delete is not within the valid range.
    ///
    /// [`ClientError::DeleteMessageDaysAmount`]: ../client/enum.ClientError.html#variant.DeleteMessageDaysAmount
    /// [`Context::delete_messages`]: ../client/struct.Context.html#delete_messages
    pub fn delete_messages(&self, message_ids: &[MessageId]) -> Result<()> {
        if message_ids.len() < 2 || message_ids.len() > 100 {
            return Err(Error::Client(ClientError::BulkDeleteAmount));
        }

        let ids: Vec<u64> = message_ids.into_iter()
            .map(|message_id| message_id.0)
            .collect();

        let map = ObjectBuilder::new()
            .insert("messages", ids)
            .build();

        http::delete_messages(self.channel_id.0, map)
    }

    /// Returns the formatted URI of the group's icon if one exists.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn_concat!("/channel-icons/{}/{}.jpg"), self.channel_id, icon))
    }

    /// Leaves the group.
    pub fn leave(&self) -> Result<Group> {
        http::leave_group(self.channel_id.0)
    }

    /// Generates a name for the group.
    ///
    /// If there are no recipients in the group, the name will be "Empty Group".
    /// Otherwise, the name is generated in a Comma Separated Value list, such
    /// as "person 1, person 2, person 3".
    pub fn name(&self) -> Cow<str> {
        match self.name {
            Some(ref name) => Cow::Borrowed(name),
            None => {
                let mut name = match self.recipients.values().nth(0) {
                    Some(recipient) => recipient.name.clone(),
                    None => return Cow::Borrowed("Empty Group"),
                };

                for recipient in self.recipients.values().skip(1) {
                    let _ = write!(name, ", {}", recipient.name);
                }

                Cow::Owned(name)
            }
        }
    }

    /// Retrieves the list of messages that have been pinned in the group.
    pub fn pins(&self) -> Result<Vec<Message>> {
        http::get_pins(self.channel_id.0)
    }

    /// Removes a recipient from the group. If the recipient is already not in
    /// the group, then nothing is done.
    ///
    /// **Note**: This is only available to the group owner.
    pub fn remove_recipient<U: Into<UserId>>(&self, user: U) -> Result<()> {
        let user = user.into();

        // If the group does not contain the recipient already, do nothing.
        if !self.recipients.contains_key(&user) {
            return Ok(());
        }

        http::remove_group_recipient(self.channel_id.0, user.0)
    }

    /// Sends a message to the group with the given content.
    ///
    /// Note that an @everyone mention will not be applied.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn send_message(&self, content: &str) -> Result<Message> {
        let map = ObjectBuilder::new()
            .insert("content", content)
            .insert("nonce", "")
            .insert("tts", false)
            .build();

        http::send_message(self.channel_id.0, map)
    }
}

impl Message {
    /// Deletes the message.
    ///
    /// **Note**: The logged in user must either be the author of the message or
    /// have the [Manage Messages] permission.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidUser`] if the current user is not the
    /// author.
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`ClientError::InvalidUser`]: ../client/enum.ClientError.html#variant.InvalidUser
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn delete(&self) -> Result<()> {
        let req = permissions::MANAGE_MESSAGES;
        let is_author = self.author.id != STATE.lock().unwrap().user.id;

        if is_author {
            return Err(Error::Client(ClientError::InvalidUser));
        }

        if !try!(utils::user_has_perms(self.channel_id, req)) && !is_author {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::delete_message(self.channel_id.0, self.id.0)
    }

    /// Deletes all of the [`Reaction`]s associated with the message.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn delete_reactions(&self) -> Result<()> {
        let req = permissions::MANAGE_MESSAGES;

        if !try!(utils::user_has_perms(self.channel_id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::delete_message_reactions(self.channel_id.0, self.id.0)
    }

    /// Edits this message, replacing the original content with new content.
    ///
    /// **Note**: You must be the author of the message to be able to do this.
    ///
    /// **Note**: Messages must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidUser`] if the current user is not the
    /// author.
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ClientError::InvalidUser`]: ../client/enum.ClientError.html#variant.InvalidUser
    /// [`ClientError::MessageTooLong`]: enum.ClientError.html#variant.MessageTooLong
    pub fn edit(&mut self, new_content: &str) -> Result<()> {
        if let Some(length_over) = Message::overflow_length(new_content) {
            return Err(Error::Client(ClientError::MessageTooLong(length_over)));
        }

        if self.author.id != STATE.lock().unwrap().user.id {
            return Err(Error::Client(ClientError::InvalidUser));
        }

        let map = ObjectBuilder::new()
            .insert("content", new_content)
            .build();

        match http::edit_message(self.channel_id.0, self.id.0, map) {
            Ok(edited) => {
                mem::replace(self, edited);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Checks the length of a string to ensure that it is within Discord's
    /// maximum message length limit.
    ///
    /// Returns `None` if the message is within the limit, otherwise returns
    /// `Some` with an inner value of how many unicode code points the message
    /// is over.
    pub fn overflow_length(content: &str) -> Option<u64> {
        // Check if the content is over the maximum number of unicode code
        // points.
        let count = content.chars().count() as i64;
        let diff = count - (constants::MESSAGE_CODE_LIMIT as i64);

        if diff > 0 {
            Some(diff as u64)
        } else {
            None
        }
    }

    /// Pins this message to its channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn pin(&self) -> Result<()> {
        let req = permissions::MANAGE_MESSAGES;

        if !try!(utils::user_has_perms(self.channel_id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::pin_message(self.channel_id.0, self.id.0)
    }

    /// React to the message with a custom [`Emoji`] or unicode character.
    ///
    /// **Note**: Requires the [Add Reactions] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required [permissions].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Emoji`]: struct.Emoji.html
    /// [Add Reactions]: permissions/constant.ADD_REACTIONS.html
    /// [permissions]: permissions
    pub fn react<R: Into<ReactionType>>(&self, reaction_type: R) -> Result<()> {
        let req = permissions::ADD_REACTIONS;

        if !try!(utils::user_has_perms(self.channel_id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::create_reaction(self.channel_id.0,
                              self.id.0,
                              reaction_type.into())
    }

    /// Replies to the user, mentioning them prior to the content in the form
    /// of: `@<USER_ID>: YOUR_CONTENT`.
    ///
    /// User mentions are generally around 20 or 21 characters long.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required permissions.
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`ClientError::MessageTooLong`]: enum.ClientError.html#variant.MessageTooLong
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn reply(&self, content: &str) -> Result<Message> {
        if let Some(length_over) = Message::overflow_length(content) {
            return Err(Error::Client(ClientError::MessageTooLong(length_over)));
        }

        let req = permissions::SEND_MESSAGES;

        if !try!(utils::user_has_perms(self.channel_id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        let mut gen = format!("{}", self.author.mention());
        gen.push(':');
        gen.push(' ');
        gen.push_str(content);

        let map = ObjectBuilder::new()
            .insert("content", gen)
            .insert("nonce", "")
            .insert("tts", false)
            .build();

        http::send_message(self.channel_id.0, map)
    }

    /// Unpins the message from its channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn unpin(&self) -> Result<()> {
        let req = permissions::MANAGE_MESSAGES;

        if !try!(utils::user_has_perms(self.channel_id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::unpin_message(self.channel_id.0, self.id.0)
    }
}

impl PermissionOverwrite {
    pub fn decode(value: Value) -> Result<PermissionOverwrite> {
        let mut map = try!(into_map(value));
        let id = try!(remove(&mut map, "id").and_then(decode_id));
        let kind = try!(remove(&mut map, "type").and_then(into_string));
        let kind = match &*kind {
            "member" => PermissionOverwriteType::Member(UserId(id)),
            "role" => PermissionOverwriteType::Role(RoleId(id)),
            _ => return Err(Error::Decode("Expected valid PermissionOverwrite type", Value::String(kind))),
        };

        missing!(map, PermissionOverwrite {
            kind: kind,
            allow: try!(remove(&mut map, "allow").and_then(Permissions::decode)),
            deny: try!(remove(&mut map, "deny").and_then(Permissions::decode)),
        })
    }
}

impl PrivateChannel {
    /// Broadcasts that the current user is typing to the recipient.
    pub fn broadcast_typing(&self) -> Result<()> {
        http::broadcast_typing(self.id.0)
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<PrivateChannel> {
        let mut map = try!(into_map(value));
        let mut recipients = try!(decode_array(try!(remove(&mut map, "recipients")),
                                  User::decode));

        missing!(map, PrivateChannel {
            id: try!(remove(&mut map, "id").and_then(ChannelId::decode)),
            kind: try!(remove(&mut map, "type").and_then(ChannelType::decode)),
            last_message_id: try!(opt(&mut map, "last_message_id", MessageId::decode)),
            last_pin_timestamp: try!(opt(&mut map, "last_pin_timestamp", into_string)),
            recipient: recipients.remove(0),
        })
    }

    /// Deletes the given message Ids from the private channel.
    ///
    /// **Note**: You can only delete your own messages.
    ///
    /// **Note** This method is only available to bot users.
    ///
    /// # Errors
    ///
    /// Returns a
    /// [`ClientError::InvalidUser`] if the current user is not a bot user.
    ///
    /// [`ClientError::InvalidUser`]: ../client/enum.ClientError.html#variant.InvalidOperationAsUser
    pub fn delete_messages(&self, message_ids: &[MessageId]) -> Result<()> {
        if !STATE.lock().unwrap().user.bot {
            return Err(Error::Client(ClientError::InvalidOperationAsUser));
        }

        let ids: Vec<u64> = message_ids.into_iter()
            .map(|message_id| message_id.0)
            .collect();

        let map = ObjectBuilder::new()
            .insert("messages", ids)
            .build();

        http::delete_messages(self.id.0, map)
    }

    /// Deletes the channel. This does not delete the contents of the channel,
    /// and is equivilant to closing a private channel on the client, which can
    /// be re-opened.
    pub fn delete(&self) -> Result<Channel> {
        http::delete_channel(self.id.0)
    }

    /// Retrieves the list of messages that have been pinned in the private
    /// channel.
    pub fn pins(&self) -> Result<Vec<Message>> {
        http::get_pins(self.id.0)
    }

    /// Sends a message to the channel with the given content.
    ///
    /// **Note**: This will only work when a [`Message`] is received.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    pub fn send_message(&self, content: &str) -> Result<Message> {
        if let Some(length_over) = Message::overflow_length(content) {
            return Err(Error::Client(ClientError::MessageTooLong(length_over)));
        }

        let map = ObjectBuilder::new()
            .insert("content", content)
            .insert("nonce", "")
            .insert("tts", false)
            .build();

        http::send_message(self.id.0, map)
    }
}

impl fmt::Display for PrivateChannel {
    /// Formats the private channel, displaying the recipient's username.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.recipient.name)
    }
}

impl PublicChannel {
    /// Broadcasts to the channel that the current user is typing.
    ///
    /// For bots, this is a good indicator for long-running commands.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a
    /// [ClientError::InvalidPermissions] if the current user does not have the
    /// required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Send Messages]: permissions/constants.SEND_MESSAGES.html
    pub fn broadcast_typing(&self) -> Result<()> {
        http::broadcast_typing(self.id.0)
    }

    pub fn create_invite<F>(&self, f: F) -> Result<RichInvite>
        where F: FnOnce(CreateInvite) -> CreateInvite {
        let req = permissions::CREATE_INVITE;

        if !try!(utils::user_has_perms(self.id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        let map = f(CreateInvite::default()).0.build();

        http::create_invite(self.id.0, map)
    }

    pub fn decode(value: Value) -> Result<PublicChannel> {
        let mut map = try!(into_map(value));

        let id = try!(remove(&mut map, "guild_id").and_then(GuildId::decode));

        PublicChannel::decode_guild(Value::Object(map), id)
    }

    pub fn decode_guild(value: Value, guild_id: GuildId) -> Result<PublicChannel> {
        let mut map = try!(into_map(value));
        missing!(map, PublicChannel {
            id: try!(remove(&mut map, "id").and_then(ChannelId::decode)),
            name: try!(remove(&mut map, "name").and_then(into_string)),
            guild_id: guild_id,
            topic: try!(opt(&mut map, "topic", into_string)),
            position: req!(try!(remove(&mut map, "position")).as_i64()),
            kind: try!(remove(&mut map, "type").and_then(ChannelType::decode)),
            last_message_id: try!(opt(&mut map, "last_message_id", MessageId::decode)),
            permission_overwrites: try!(decode_array(try!(remove(&mut map, "permission_overwrites")), PermissionOverwrite::decode)),
            bitrate: remove(&mut map, "bitrate").ok().and_then(|v| v.as_u64()),
            user_limit: remove(&mut map, "user_limit").ok().and_then(|v| v.as_u64()),
            last_pin_timestamp: try!(opt(&mut map, "last_pin_timestamp", into_string)),
        })
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    pub fn delete(&self) -> Result<Channel> {
        let req = permissions::MANAGE_CHANNELS;

        if !try!(utils::user_has_perms(self.id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::delete_channel(self.id.0)
    }
    pub fn edit<F>(&mut self, f: F) -> Result<()>

        where F: FnOnce(EditChannel) -> EditChannel {
        let req = permissions::MANAGE_CHANNELS;

        if !try!(utils::user_has_perms(self.id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        let map = ObjectBuilder::new()
            .insert("name", &self.name)
            .insert("position", self.position)
            .insert("type", self.kind.name());

        let edited = f(EditChannel(map)).0.build();

        match http::edit_channel(self.id.0, edited) {
            Ok(channel) => {
                mem::replace(self, channel);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Return a [`Mention`] which will link to this channel.
    ///
    /// [`Mention`]: struct.Mention.html
    pub fn mention(&self) -> Mention {
        self.id.mention()
    }

    pub fn pins(&self) -> Result<Vec<Message>> {
        http::get_pins(self.id.0)
    }

    /// Sends a message to the channel with the given content.
    ///
    /// **Note**: This will only work when a [`Message`] is received.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn send_message(&self, content: &str) -> Result<Message> {
        if let Some(length_over) = Message::overflow_length(content) {
            return Err(Error::Client(ClientError::MessageTooLong(length_over)));
        }

        let req = permissions::SEND_MESSAGES;

        if !try!(utils::user_has_perms(self.id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        let map = ObjectBuilder::new()
            .insert("content", content)
            .insert("nonce", "")
            .insert("tts", false)
            .build();

        http::send_message(self.id.0, map)
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    pub fn webhooks(&self) -> Result<Vec<Webhook>> {
        http::get_channel_webhooks(self.id.0)
    }
}

impl fmt::Display for PublicChannel {
    /// Formas the channel, creating a mention of it.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}

impl Reaction {
    /// Deletes the reaction, but only if the current user is the user who made
    /// the reaction or has permission to.
    ///
    /// **Note**: Requires the [`Manage Messages`] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required [permissions].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    /// [permissions]: permissions
    pub fn delete(&self) -> Result<()> {
        let user = if self.user_id == STATE.lock().unwrap().user.id {
            None
        } else {
            Some(self.user_id.0)
        };

        // If the reaction is one _not_ made by the current user, then ensure
        // that the current user has permission* to delete the reaction.
        //
        // Normally, users can only delete their own reactions.
        //
        // * The `Manage Messages` permission.
        if user.is_some() {
            let req = permissions::MANAGE_MESSAGES;

            if !utils::user_has_perms(self.channel_id, req).unwrap_or(true) {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        http::delete_reaction(self.channel_id.0,
                              self.message_id.0,
                              user,
                              self.emoji.clone())
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
    /// [permissions]: permissions
    pub fn users<R, U>(&self,
                       reaction_type: R,
                       limit: Option<u8>,
                       after: Option<U>)
                       -> Result<Vec<User>>
                       where R: Into<ReactionType>,
                             U: Into<UserId> {
        http::get_reaction_users(self.channel_id.0,
                                 self.message_id.0,
                                 reaction_type.into(),
                                 limit.unwrap_or(50),
                                 after.map(|u| u.into().0))
    }
}

/// The type of a [`Reaction`] sent.
///
/// [`Reaction`]: struct.Reaction.html
#[derive(Clone, Debug)]
pub enum ReactionType {
    /// A reaction with a [`Guild`]s custom [`Emoji`], which is unique to the
    /// guild.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [`Guild`]: struct.Guild.html
    Custom {
        /// The Id of the custom [`Emoji`].
        ///
        /// [`Emoji`]: struct.Emoji.html
        id: EmojiId,
        /// The name of the custom emoji. This is primarily used for decoration
        /// and distinguishing the emoji client-side.
        name: String,
    },
    /// A reaction with a twemoji.
    Unicode(String),
}

impl ReactionType {
    /// Creates a data-esque display of the type. This is not very useful for
    /// displaying, as the primary client can not render it, but can be useful
    /// for debugging.
    ///
    /// **Note**: This is mainly for use internally. There is otherwise most
    /// likely little use for it.
    #[inline(always)]
    pub fn as_data(&self) -> String {
        match *self {
            ReactionType::Custom { id, ref name } => {
                format!("{}:{}", id, name)
            },
            ReactionType::Unicode(ref unicode) => unicode.clone(),
        }
    }

    pub fn decode(value: Value) -> Result<Self> {
        let mut map = try!(into_map(value));
        let name = try!(remove(&mut map, "name").and_then(into_string));

        // Only custom emoji reactions (`ReactionType::Custom`) have an Id.
        Ok(match try!(opt(&mut map, "id", EmojiId::decode)) {
            Some(id) => ReactionType::Custom {
                id: id,
                name: name,
            },
            None => ReactionType::Unicode(name),
        })
    }
}

impl From<Emoji> for ReactionType {
    fn from(emoji: Emoji) -> ReactionType {
        ReactionType::Custom {
            id: emoji.id,
            name: emoji.name,
        }
    }
}

impl From<String> for ReactionType {
    fn from(unicode: String) -> ReactionType {
        ReactionType::Unicode(unicode)
    }
}

impl fmt::Display for ReactionType {
    /// Formats the reaction type, displaying the associated emoji in a
    /// way that clients can understand.
    ///
    /// If the type is a [custom][`ReactionType::Custom`] emoji, then refer to
    /// the documentation for [emoji's formatter][`Emoji::fmt`] on how this is
    /// displayed. Otherwise, if the type is a
    /// [unicode][`ReactionType::Unicode`], then the inner unicode is displayed.
    ///
    /// [`Emoji::fmt`]: struct.Emoji.html#method.fmt
    /// [`ReactionType::Custom`]: enum.ReactionType.html#variant.Custom
    /// [`ReactionType::Unicode`]: enum.ReactionType.html#variant.Unicode
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReactionType::Custom { id, ref name } => {
                try!(f.write_char('<'));
                try!(f.write_char(':'));
                try!(f.write_str(&name));
                try!(f.write_char(':'));
                try!(fmt::Display::fmt(&id, f));
                f.write_char('>')
            },
            ReactionType::Unicode(ref unicode) => f.write_str(unicode),
        }
    }
}
