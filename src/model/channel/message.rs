//! Models relating to Discord channels.

#[cfg(feature = "http")]
use crate::http::CacheHttp;
use chrono::{DateTime, FixedOffset};
use crate::{model::prelude::*};
use serde_json::Value;

#[cfg(feature = "model")]
use crate::builder::{CreateEmbed, EditMessage};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::CacheRwLock;
#[cfg(all(feature = "cache", feature = "model"))]
use parking_lot::RwLock;
#[cfg(all(feature = "client", feature = "model"))]
use serde_json::json;
#[cfg(all(feature = "cache", feature = "model"))]
use std::sync::Arc;
#[cfg(all(feature = "cache", feature = "model"))]
use std::fmt::Write;
#[cfg(feature = "model")]
use std::mem;
#[cfg(feature = "model")]
use crate::{constants, utils as serenity_utils};
#[cfg(feature = "http")]
use crate::http::Http;

/// A representation of a message over a guild's text channel, a group, or a
/// private channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    /// The Id of the [`Guild`] that the message was sent in. This value will
    /// only be present if this message was received over the gateway.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    pub guild_id: Option<GuildId>,
    /// Indicator of the type of message this is, i.e. whether it is a regular
    /// message or a system message.
    #[serde(rename = "type")]
    pub kind: MessageType,
    /// A partial amount of data about the user's member data, if this message
    /// was sent in a guild.
    pub member: Option<PartialMember>,
    /// Indicator of whether the message mentions everyone.
    pub mention_everyone: bool,
    /// Array of [`Role`]s' Ids mentioned in the message.
    ///
    /// [`Role`]: ../guild/struct.Role.html
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
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

#[cfg(feature = "model")]
impl Message {
    /// Retrieves the related channel located in the cache.
    ///
    /// Returns `None` if the channel is not in the cache.
    #[cfg(feature = "cache")]
    #[inline]
    pub fn channel(&self, cache: impl AsRef<CacheRwLock>) -> Option<Channel> { cache.as_ref().read().channel(self.channel_id) }

    /// A util function for determining whether this message was sent by someone else, or the
    /// bot.
    #[cfg(all(feature = "cache", feature = "utils"))]
    pub fn is_own(&self, cache: impl AsRef<CacheRwLock>) -> bool { self.author.id == cache.as_ref().read().user.id }

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
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`ModelError::InvalidUser`]: ../error/enum.Error.html#variant.InvalidUser
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[cfg(feature = "http")]
    pub fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_MESSAGES;
                let is_author = self.author.id == cache.read().user.id;
                let has_perms = utils::user_has_perms(&cache, self.channel_id, req)?;

                if !is_author && !has_perms {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        self.channel_id.delete_message(&cache_http.http(), self.id)
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
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[cfg(feature = "http")]
    pub fn delete_reactions(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let req = Permissions::MANAGE_MESSAGES;

                if !utils::user_has_perms(cache, self.channel_id, req)? {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }
        }

        cache_http.http().as_ref().delete_message_reactions(self.channel_id.0, self.id.0)
    }

    /// Edits this message, replacing the original content with new content.
    ///
    /// Message editing preserves all unchanged message data.
    ///
    /// Refer to the documentation for [`EditMessage`] for more information
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
    /// message.edit(&context, |m| m.content("new content"));
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
    /// [`ModelError::InvalidUser`]: ../error/enum.Error.html#variant.InvalidUser
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    /// [`EditMessage`]: ../../builder/struct.EditMessage.html
    /// [`the limit`]: ../../builder/struct.EditMessage.html#method.content
    #[cfg(feature = "client")]
    pub fn edit<F>(&mut self, cache_http: impl CacheHttp, f: F) -> Result<()>
        where F: FnOnce(&mut EditMessage) -> &mut EditMessage {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {

                if self.author.id != cache.read().user.id {
                    return Err(Error::Model(ModelError::InvalidUser));
                }
            }
        }

        let mut builder = EditMessage::default();

        if !self.content.is_empty() {
            builder.content(&self.content);
        }

        if let Some(embed) = self.embeds.get(0) {
            let embed = CreateEmbed::from(embed.clone());
            builder.embed( |e| {
                *e = embed;
                e
            });
        }

        f(&mut builder);

        let map = serenity_utils::hashmap_to_json_map(builder.0);

        match cache_http.http().edit_message(self.channel_id.0, self.id.0, &Value::Object(map)) {
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
                    chosen.to_string()
                };
            },
            _ => {},
        }
    }

    /// Returns message content, but with user and role mentions replaced with
    /// names and everyone/here mentions cancelled.
    #[cfg(feature = "cache")]
    pub fn content_safe(&self, cache: impl AsRef<CacheRwLock>) -> String {
        let mut result = self.content.clone();

        // First replace all user mentions.
        for u in &self.mentions {
            let mut at_distinct = String::with_capacity(38);
            at_distinct.push('@');
            at_distinct.push_str(&u.name);
            at_distinct.push('#');
            let _ = write!(at_distinct, "{:04}", u.discriminator);
            result = result.replace(&u.mention(), &at_distinct);
        }

        // Then replace all role mentions.
        for id in &self.mention_roles {
            let mention = id.mention();

            if let Some(role) = id.to_role_cached(&cache) {
                result = result.replace(&mention, &format!("@{}", role.name));
            } else {
                result = result.replace(&mention, "@deleted-role");
            }
        }

        // And finally replace everyone and here mentions.
        result
            .replace("@everyone", "@\u{200B}everyone")
            .replace("@here", "@\u{200B}here")
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
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: ../user/struct.User.html
    /// [Read Message History]: ../permissions/struct.Permissions.html#associatedconstant.READ_MESSAGE_HISTORY
    #[cfg(feature = "http")]
    #[inline]
    pub fn reaction_users<R, U>(
        &self,
        http: impl AsRef<Http>,
        reaction_type: R,
        limit: Option<u8>,
        after: U,
    ) -> Result<Vec<User>> where R: Into<ReactionType>,
                                 U: Into<Option<UserId>> {
        self.channel_id.reaction_users(&http, self.id, reaction_type, limit, after)
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
    pub fn guild(&self, cache: impl AsRef<CacheRwLock>) -> Option<Arc<RwLock<Guild>>> {
       cache.as_ref().read().guild(self.guild_id?)
    }

    /// True if message was sent using direct messages.
    pub fn is_private(&self) -> bool {
        self.guild_id.is_none()
    }

    /// Retrieves a clone of the author's Member instance, if this message was
    /// sent in a guild.
    ///
    /// Note that since this clones, it is preferable performance-wise to
    /// manually retrieve the guild from the cache and access
    /// [`Guild::members`].
    ///
    /// [`Guild::members`]: ../guild/struct.Guild.html#structfield.members
    #[cfg(feature = "cache")]
    pub fn member(&self, cache: impl AsRef<CacheRwLock>) -> Option<Member> {
        self.guild(&cache).and_then(|g| g.read().members.get(&self.author.id).cloned())
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
        let diff = count - i64::from(constants::MESSAGE_CODE_LIMIT);

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
    /// If the `cache` is enabled, returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have
    /// the required permissions.
    ///
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES.html
    #[cfg(feature = "http")]
    pub fn pin(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {

                if self.guild_id.is_some() {
                    let req = Permissions::MANAGE_MESSAGES;

                    if !utils::user_has_perms(&cache, self.channel_id, req)? {
                        return Err(Error::Model(ModelError::InvalidPermissions(req)));
                    }
                }
            }
        }

        self.channel_id.pin(cache_http.http(), self.id.0)
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
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [Add Reactions]:
    /// ../permissions/struct.Permissions.html#associatedconstant.ADD_REACTIONS
    /// [permissions]: ../permissions/index.html
    #[inline]
    #[cfg(feature = "client")]
    pub fn react<R: Into<ReactionType>>(&self, cache_http: impl CacheHttp, reaction_type: R) -> Result<()> {
        self._react(cache_http, &reaction_type.into())
    }

    #[cfg(feature = "client")]
    fn _react(&self, cache_http: impl CacheHttp, reaction_type: &ReactionType) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {

                if self.guild_id.is_some() {
                    let req = Permissions::ADD_REACTIONS;

                    if !utils::user_has_perms(cache, self.channel_id, req)? {
                        return Err(Error::Model(ModelError::InvalidPermissions(req)));
                    }
                }
            }
        }

        cache_http.http().create_reaction(self.channel_id.0, self.id.0, reaction_type)
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
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [`ModelError::MessageTooLong`]: ../error/enum.Error.html#variant.MessageTooLong
    /// [Send Messages]: ../permissions/struct.Permissions.html#associatedconstant.SEND_MESSAGES
    #[cfg(feature = "client")]
    pub fn reply(&self, cache_http: impl CacheHttp, content: &str) -> Result<Message> {
        if let Some(length_over) = Message::overflow_length(content) {
            return Err(Error::Model(ModelError::MessageTooLong(length_over)));
        }

        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {

                if self.guild_id.is_some() {
                    let req = Permissions::SEND_MESSAGES;

                    if !utils::user_has_perms(cache, self.channel_id, req)? {
                        return Err(Error::Model(ModelError::InvalidPermissions(req)));
                    }
                }
            }
        }

        let mut gen = self.author.mention();
        gen.push_str(": ");
        gen.push_str(content);

        let map = json!({
            "content": gen,
            "tts": false,
        });

        cache_http.http().send_message(self.channel_id.0, &map)
    }

    /// Checks whether the message mentions passed [`UserId`].
    ///
    /// [`UserId`]: ../id/struct.UserId.html
    #[inline]
    pub fn mentions_user_id<I: Into<UserId>>(&self, id: I) -> bool {
        self._mentions_user_id(id.into())
    }

    fn _mentions_user_id(&self, id: UserId) -> bool {
        self.mentions.iter().any(|mentioned_user| mentioned_user.id.0 == id.0)
    }

    /// Checks whether the message mentions passed [`User`].
    ///
    /// [`User`]: ../user/struct.User.html
    pub fn mentions_user(&self, user: &User) -> bool {
        self.mentions_user_id(user.id)
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
    /// [`ModelError::InvalidPermissions`]: ../error/enum.Error.html#variant.InvalidPermissions
    /// [Manage Messages]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_MESSAGES
    #[cfg(feature = "http")]
    pub fn unpin(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {

                if self.guild_id.is_some() {
                    let req = Permissions::MANAGE_MESSAGES;

                    if !utils::user_has_perms(cache, self.channel_id, req)? {
                        return Err(Error::Model(ModelError::InvalidPermissions(req)));
                    }
                }
            }
        }

        cache_http.http().unpin_message(self.channel_id.0, self.id.0)
    }

    /// Tries to return author's nickname in the current channel's guild.
    ///
    /// **Note**:
    /// If message was sent in a private channel, then the function will return
    /// `None`.
    #[cfg(feature = "http")]
    pub fn author_nick(&self, cache_http: impl CacheHttp) -> Option<String> {
        self.guild_id.as_ref().and_then(|guild_id| self.author.nick_in(cache_http, *guild_id))
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
            let overflow = total as u64 - u64::from(constants::EMBED_MAX_LENGTH);

            Err(Error::Model(ModelError::EmbedTooLarge(overflow)))
        }
    }
}

impl AsRef<MessageId> for Message {
    fn as_ref(&self) -> &MessageId {
        &self.id
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageReaction {
    /// The amount of the type of reaction that have been sent for the
    /// associated message.
    pub count: u64,
    /// Indicator of whether the current user has sent the type of reaction.
    pub me: bool,
    /// The type of reaction.
    #[serde(rename = "emoji")]
    pub reaction_type: ReactionType,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

/// Differentiates between regular and different types of system messages.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum MessageType {
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
    #[doc(hidden)]
    __Nonexhaustive,
}

enum_number!(
    MessageType {
        Regular,
        GroupRecipientAddition,
        GroupRecipientRemoval,
        GroupCallCreation,
        GroupNameUpdate,
        GroupIconUpdate,
        PinsAdd,
        MemberJoin,
    }
);

impl MessageType {
    pub fn num(self) -> u64 {
        use self::MessageType::*;

        match self {
            Regular => 0,
            GroupRecipientAddition => 1,
            GroupRecipientRemoval => 2,
            GroupCallCreation => 3,
            GroupNameUpdate => 4,
            GroupIconUpdate => 5,
            PinsAdd => 6,
            MemberJoin => 7,
            __Nonexhaustive => unreachable!(),
        }
    }
}
