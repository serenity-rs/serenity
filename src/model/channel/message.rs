use chrono::{DateTime, FixedOffset};
use serde_json::Value;
use model::*;

#[cfg(feature = "cache")]
use std::fmt::Write;
#[cfg(feature = "model")]
use std::mem;
#[cfg(feature = "model")]
use builder::{CreateEmbed, CreateMessage};
#[cfg(feature = "model")]
use constants;
#[cfg(feature = "cache")]
use CACHE;
#[cfg(feature = "model")]
use http;

/// A representation of a message over a guild's text channel, a group, or a
/// private channel.
#[derive(Clone, Debug, Deserialize)]
pub struct Message {
    /// The unique Id of the message. Can be used to calculate the creation date
    /// of the message.
    pub id: MessageId,
    /// An vector of the files attached to a message.
    pub attachments: Vec<Attachment>,
    /// The user that sent the message.
    pub author: User,
    /// The Id of the [`Channel`] that the message was sent to.
    ///
    /// [`Channel`]: enum.Channel.html
    pub channel_id: ChannelId,
    /// The content of the message.
    pub content: String,
    /// The timestamp of the last time the message was updated, if it was.
    pub edited_timestamp: Option<DateTime<FixedOffset>>,
    /// Array of embeds sent with the message.
    pub embeds: Vec<Embed>,
    /// Indicator of the type of message this is, i.e. whether it is a regular
    /// message or a system message.
    #[serde(rename = "type")]
    pub kind: MessageType,
    /// Indicator of whether the message mentions everyone.
    pub mention_everyone: bool,
    /// Array of [`Role`]s' Ids mentioned in the message.
    ///
    /// [`Role`]: struct.Role.html
    pub mention_roles: Vec<RoleId>,
    /// Array of users mentioned in the message.
    pub mentions: Vec<User>,
    /// Non-repeating number used for ensuring message order.
    #[serde(default)]
    pub nonce: Value,
    /// Indicator of whether the message is pinned.
    pub pinned: bool,
    /// Array of reactions performed on the message.
    #[serde(default)]
    pub reactions: Vec<MessageReaction>,
    /// Initial message creation timestamp, calculated from its Id.
    pub timestamp: DateTime<FixedOffset>,
    /// Indicator of whether the command is to be played back via
    /// text-to-speech.
    ///
    /// In the client, this is done via the `/tts` slash command.
    pub tts: bool,
    /// The Id of the webhook that sent this message, if one did.
    pub webhook_id: Option<WebhookId>,
}

#[cfg(feature = "model")]
impl Message {
    /// Retrieves the related channel located in the cache.
    ///
    /// Returns `None` if the channel is not in the cache.
    ///
    /// # Examples
    ///
    /// On command, print the name of the channel that a message took place in:
    ///
    /// ```rust,no_run
    /// # #[macro_use] extern crate serenity;
    /// #
    /// # fn main() {
    /// #   use serenity::prelude::*;
    /// #   use serenity::framework::standard::Args;
    /// #   struct Handler;
    /// #
    /// #   impl EventHandler for Handler {}
    /// #   let mut client = Client::new("token", Handler);
    /// #
    /// use serenity::model::Channel;
    /// use serenity::framework::StandardFramework;
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .configure(|c| c.prefix("~"))
    ///     .command("channelname", |c| c.exec(channel_name)));
    ///
    /// command!(channel_name(_ctx, msg) {
    ///     let _ = match msg.channel() {
    ///         Some(Channel::Group(c)) => msg.reply(&c.read().unwrap().name()),
    ///         Some(Channel::Guild(c)) => msg.reply(&c.read().unwrap().name),
    ///         Some(Channel::Private(c)) => {
    ///             let channel = c.read().unwrap();
    ///             let user = channel.recipient.read().unwrap();
    ///
    ///             msg.reply(&format!("DM with {}", user.name.clone()))
    ///         },
    ///         None => msg.reply("Unknown"),
    ///     };
    /// });
    /// # }
    /// ```
    #[cfg(feature = "cache")]
    #[inline]
    pub fn channel(&self) -> Option<Channel> { CACHE.read().unwrap().channel(self.channel_id) }

    /// A util function for determining whether this message was sent by someone else, or the
    /// bot.
    #[cfg(all(feature = "cache", feature = "utils"))]
    pub fn is_own(&self) -> bool { self.author.id == CACHE.read().unwrap().user.id }

    /// Deletes the message.
    ///
    /// **Note**: The logged in user must either be the author of the message or
    /// have the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`ModelError::InvalidUser`]: enum.ModelError.html#variant.InvalidUser
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn delete(&self) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::MANAGE_MESSAGES;
            let is_author = self.author.id == CACHE.read().unwrap().user.id;
            let has_perms = utils::user_has_perms(self.channel_id, req)?;

            if !is_author && !has_perms {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.channel_id.delete_message(self.id)
    }

    /// Deletes all of the [`Reaction`]s associated with the message.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn delete_reactions(&self) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::MANAGE_MESSAGES;

            if !utils::user_has_perms(self.channel_id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        http::delete_message_reactions(self.channel_id.0, self.id.0)
    }

    /// Edits this message, replacing the original content with new content.
    ///
    /// Message editing preserves all unchanged message data.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Requires that the current user be the author of the message.
    ///
    /// # Examples
    ///
    /// Edit a message with new content:
    ///
    /// ```rust,ignore
    /// // assuming a `message` has already been bound
    ///
    /// message.edit(|m| m.content("new content"));
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidUser`] if the
    /// current user is not the author.
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over [`the limit`], containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ModelError::InvalidUser`]: enum.ModelError.html#variant.InvalidUser
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    /// [`CreateMessage`]: ../builder/struct.CreateMessage.html
    /// [`the limit`]: ../builder/struct.CreateMessage.html#method.content
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        #[cfg(feature = "cache")]
        {
            if self.author.id != CACHE.read().unwrap().user.id {
                return Err(Error::Model(ModelError::InvalidUser));
            }
        }

        let mut builder = CreateMessage::default();

        if !self.content.is_empty() {
            builder = builder.content(&self.content);
        }

        if let Some(embed) = self.embeds.get(0) {
            builder = builder.embed(|_| CreateEmbed::from(embed.clone()));
        }

        if self.tts {
            builder = builder.tts(true);
        }

        let map = f(builder).0;

        match http::edit_message(self.channel_id.0, self.id.0, &Value::Object(map)) {
            Ok(edited) => {
                mem::replace(self, edited);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    pub(crate) fn transform_content(&mut self) {
        match self.kind {
            MessageType::PinsAdd => {
                self.content = format!(
                    "{} pinned a message to this channel. See all the pins.",
                    self.author
                );
            },
            MessageType::MemberJoin => {
                let sec = self.timestamp.timestamp() as usize;
                let chosen = constants::JOIN_MESSAGES[sec % constants::JOIN_MESSAGES.len()];

                self.content = if chosen.contains("$user") {
                    chosen.replace("$user", &self.author.mention())
                } else {
                    chosen.to_owned()
                };
            },
            _ => {},
        }
    }

    /// Returns message content, but with user and role mentions replaced with
    /// names and everyone/here mentions cancelled.
    #[cfg(feature = "cache")]
    pub fn content_safe(&self) -> String {
        let mut result = self.content.clone();

        // First replace all user mentions.
        for u in &self.mentions {
            let mut at_distinct = String::with_capacity(38);
            at_distinct.push('@');
            at_distinct.push_str(&u.name);
            at_distinct.push('#');
            let _ = write!(at_distinct, "{}", u.discriminator);
            result = result.replace(&u.mention(), &at_distinct);
        }

        // Then replace all role mentions.
        for id in &self.mention_roles {
            let mention = id.mention();

            if let Some(role) = id.find() {
                result = result.replace(&mention, &format!("@{}", role.name));
            } else {
                result = result.replace(&mention, "@deleted-role");
            }
        }

        // And finally replace everyone and here mentions.
        result.replace("@everyone", "@\u{200B}everyone").replace(
            "@here",
            "@\u{200B}here",
        )
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
    #[inline]
    pub fn reaction_users<R, U>(&self,
                                reaction_type: R,
                                limit: Option<u8>,
                                after: Option<U>)
                                -> Result<Vec<User>>
        where R: Into<ReactionType>, U: Into<UserId> {
        self.channel_id.reaction_users(
            self.id,
            reaction_type,
            limit,
            after,
        )
    }

    /// Returns the associated `Guild` for the message if one is in the cache.
    ///
    /// Returns `None` if the guild's Id could not be found via [`guild_id`] or
    /// if the Guild itself is not cached.
    ///
    /// Requires the `cache` feature be enabled.
    ///
    /// [`guild_id`]: #method.guild_id
    #[cfg(feature = "cache")]
    pub fn guild(&self) -> Option<Arc<RwLock<Guild>>> {
        self.guild_id().and_then(|guild_id| {
            CACHE.read().unwrap().guild(guild_id)
        })
    }

    /// Retrieves the Id of the guild that the message was sent in, if sent in
    /// one.
    ///
    /// Returns `None` if the channel data or guild data does not exist in the
    /// cache.
    #[cfg(feature = "cache")]
    pub fn guild_id(&self) -> Option<GuildId> {
        match CACHE.read().unwrap().channel(self.channel_id) {
            Some(Channel::Guild(ch)) => Some(ch.read().unwrap().guild_id),
            _ => None,
        }
    }

    /// True if message was sent using direct messages.
    #[cfg(feature = "cache")]
    pub fn is_private(&self) -> bool {
        match CACHE.read().unwrap().channel(self.channel_id) {
            Some(Channel::Group(_)) |
            Some(Channel::Private(_)) => true,
            _ => false,
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

        if diff > 0 { Some(diff as u64) } else { None }
    }

    /// Pins this message to its channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn pin(&self) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::MANAGE_MESSAGES;

            if !utils::user_has_perms(self.channel_id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.channel_id.pin(self.id.0)
    }

    /// React to the message with a custom [`Emoji`] or unicode character.
    ///
    /// **Note**: Requires the [Add Reactions] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have the
    /// required [permissions].
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`Emoji`]: struct.Emoji.html
    /// [Add Reactions]: permissions/constant.ADD_REACTIONS.html
    /// [permissions]: permissions
    pub fn react<R: Into<ReactionType>>(&self, reaction_type: R) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::ADD_REACTIONS;

            if !utils::user_has_perms(self.channel_id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        http::create_reaction(self.channel_id.0, self.id.0, &reaction_type.into())
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
    /// If the `cache` is enabled, returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// Returns a [`ModelError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`ModelError::MessageTooLong`]: enum.ModelError.html#variant.MessageTooLong
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn reply(&self, content: &str) -> Result<Message> {
        if let Some(length_over) = Message::overflow_length(content) {
            return Err(Error::Model(ModelError::MessageTooLong(length_over)));
        }

        #[cfg(feature = "cache")]
        {
            let req = permissions::SEND_MESSAGES;

            if !utils::user_has_perms(self.channel_id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        let mut gen = self.author.mention();
        gen.push_str(": ");
        gen.push_str(content);

        let map = json!({
            "content": gen,
            "tts": false,
        });

        http::send_message(self.channel_id.0, &map)
    }

    /// Unpins the message from its channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    pub fn unpin(&self) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::MANAGE_MESSAGES;

            if !utils::user_has_perms(self.channel_id, req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        http::unpin_message(self.channel_id.0, self.id.0)
    }

    pub(crate) fn check_content_length(map: &JsonMap) -> Result<()> {
        if let Some(content) = map.get("content") {
            if let Value::String(ref content) = *content {
                if let Some(length_over) = Message::overflow_length(content) {
                    return Err(Error::Model(ModelError::MessageTooLong(length_over)));
                }
            }
        }

        Ok(())
    }

    pub(crate) fn check_embed_length(map: &JsonMap) -> Result<()> {
        let embed = match map.get("embed") {
            Some(&Value::Object(ref value)) => value,
            _ => return Ok(()),
        };

        let mut total: usize = 0;

        if let Some(&Value::Object(ref author)) = embed.get("author") {
            if let Some(&Value::Object(ref name)) = author.get("name") {
                total += name.len();
            }
        }

        if let Some(&Value::String(ref description)) = embed.get("description") {
            total += description.len();
        }

        if let Some(&Value::Array(ref fields)) = embed.get("fields") {
            for field_as_value in fields {
                if let Value::Object(ref field) = *field_as_value {
                    if let Some(&Value::String(ref field_name)) = field.get("name") {
                        total += field_name.len();
                    }

                    if let Some(&Value::String(ref field_value)) = field.get("value") {
                        total += field_value.len();
                    }
                }
            }
        }

        if let Some(&Value::Object(ref footer)) = embed.get("footer") {
            if let Some(&Value::String(ref text)) = footer.get("text") {
                total += text.len();
            }
        }

        if let Some(&Value::String(ref title)) = embed.get("title") {
            total += title.len();
        }

        if total <= constants::EMBED_MAX_LENGTH as usize {
            Ok(())
        } else {
            let overflow = total as u64 - constants::EMBED_MAX_LENGTH as u64;

            Err(Error::Model(ModelError::EmbedTooLarge(overflow)))
        }
    }
}

impl From<Message> for MessageId {
    /// Gets the Id of a `Message`.
    fn from(message: Message) -> MessageId { message.id }
}

impl<'a> From<&'a Message> for MessageId {
    /// Gets the Id of a `Message`.
    fn from(message: &Message) -> MessageId { message.id }
}

/// A representation of a reaction to a message.
///
/// Multiple of the same [reaction type] are sent into one `MessageReaction`,
/// with an associated [`count`].
///
/// [`count`]: #structfield.count
/// [reaction type]: enum.ReactionType.html
#[derive(Clone, Debug, Deserialize)]
pub struct MessageReaction {
    /// The amount of the type of reaction that have been sent for the
    /// associated message.
    pub count: u64,
    /// Indicator of whether the current user has sent the type of reaction.
    pub me: bool,
    /// The type of reaction.
    #[serde(rename = "emoji")]
    pub reaction_type: ReactionType,
}

enum_number!(
    /// Differentiates between regular and different types of system messages.
    MessageType {
        /// A regular message.
        Regular = 0,
        /// An indicator that a recipient was added by the author.
        GroupRecipientAddition = 1,
        /// An indicator that a recipient was removed by the author.
        GroupRecipientRemoval = 2,
        /// An indicator that a call was started by the author.
        GroupCallCreation = 3,
        /// An indicator that the group name was modified by the author.
        GroupNameUpdate = 4,
        /// An indicator that the group icon was modified by the author.
        GroupIconUpdate = 5,
        /// An indicator that a message was pinned by the author.
        PinsAdd = 6,
        /// An indicator that a member joined the guild.
        MemberJoin = 7,
    }
);
