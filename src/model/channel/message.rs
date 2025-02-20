//! Models relating to Discord channels.

#[cfg(feature = "model")]
use std::borrow::Cow;

use nonmax::NonMaxU64;

#[cfg(all(feature = "model", feature = "utils"))]
use crate::builder::{CreateAllowedMentions, CreateMessage, EditMessage};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::{Cache, GuildRef};
#[cfg(feature = "model")]
use crate::constants;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;
use crate::model::utils::{StrOrInt, discord_colours};

/// A representation of a message over a guild's text channel, a group, or a private channel.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#message-object) with some
/// [extra fields](https://discord.com/developers/docs/topics/gateway-events#message-create-message-create-extra-fields).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct Message {
    /// The unique Id of the message. Can be used to calculate the creation date of the message.
    pub id: MessageId,
    /// The Id of the [`Channel`] that the message was sent to.
    pub channel_id: ChannelId,
    /// The user that sent the message.
    pub author: User,
    /// The content of the message.
    pub content: FixedString<u16>,
    /// Initial message creation timestamp, calculated from its Id.
    pub timestamp: Timestamp,
    /// The timestamp of the last time the message was updated, if it was.
    pub edited_timestamp: Option<Timestamp>,
    /// Indicator of whether the command is to be played back via text-to-speech.
    ///
    /// In the client, this is done via the `/tts` slash command.
    pub tts: bool,
    /// Indicator of whether the message mentions everyone.
    pub mention_everyone: bool,
    /// Array of users mentioned in the message.
    pub mentions: FixedArray<User>,
    /// Array of [`Role`]s' Ids mentioned in the message.
    pub mention_roles: FixedArray<RoleId>,
    /// Channels specifically mentioned in this message.
    ///
    /// **Note**: Not all channel mentions in a message will appear in [`Self::mention_channels`].
    /// Only textual channels that are visible to everyone in a lurkable guild will ever be
    /// included.
    ///
    /// A lurkable guild is one that allows users to read public channels in a server without
    /// actually joining the server. It also allows users to look at these channels without being
    /// logged in to Discord.
    ///
    /// Only crossposted messages (via Channel Following) currently include
    /// [`Self::mention_channels`] at all. If no mentions in the message meet these requirements,
    /// this field will not be sent.
    ///
    /// [Refer to Discord's documentation for more information][discord-docs].
    ///
    /// [discord-docs]: https://discord.com/developers/docs/resources/channel#message-object
    #[serde(default)]
    pub mention_channels: FixedArray<ChannelMention>,
    /// An vector of the files attached to a message.
    pub attachments: FixedArray<Attachment>,
    /// Array of embeds sent with the message.
    pub embeds: FixedArray<Embed>,
    /// Array of reactions performed on the message.
    #[serde(default)]
    pub reactions: FixedArray<MessageReaction>,
    /// Non-repeating number used for ensuring message order.
    #[serde(default)]
    pub nonce: Option<Nonce>,
    /// Indicator of whether the message is pinned.
    pub pinned: bool,
    /// The Id of the webhook that sent this message, if one did.
    pub webhook_id: Option<WebhookId>,
    /// Indicator of the type of message this is, i.e. whether it is a regular message or a system
    /// message.
    #[serde(rename = "type")]
    pub kind: MessageType,
    /// Sent with Rich Presence-related chat embeds.
    pub activity: Option<MessageActivity>,
    /// Sent with Rich Presence-related chat embeds.
    pub application: Option<MessageApplication>,
    /// If the message is an Interaction or application-owned webhook, this is the id of the
    /// application.
    pub application_id: Option<ApplicationId>,
    /// Reference data sent with crossposted messages.
    pub message_reference: Option<MessageReference>,
    /// Bit flags describing extra features of the message.
    pub flags: Option<MessageFlags>,
    /// The message that was replied to using this message.
    pub referenced_message: Option<Box<Message>>, // Boxed to avoid recursion
    /// An array of message snapshots, known as forwarded messages.
    #[serde(default, deserialize_with = "deserialize_snapshots")]
    pub message_snapshots: Vec<MessageSnapshot>,
    #[cfg(not(feature = "unstable"))]
    pub interaction: Option<Box<MessageInteraction>>,
    /// Sent if the message is a response to an [`Interaction`].
    ///
    /// [`Interaction`]: crate::model::application::Interaction
    pub interaction_metadata: Option<Box<MessageInteractionMetadata>>,
    /// The thread that was started from this message, includes thread member object.
    pub thread: Option<Box<GuildChannel>>,
    /// The components of this message
    #[serde(default)]
    pub components: FixedArray<ActionRow>,
    /// Array of message sticker item objects.
    #[serde(default)]
    pub sticker_items: FixedArray<StickerItem>,
    /// A generally increasing integer (there may be gaps or duplicates) that represents the
    /// approximate position of the message in a thread, it can be used to estimate the relative
    /// position of the message in a thread in company with total_message_sent on parent thread.
    pub position: Option<NonMaxU64>,
    /// Data of the role subscription purchase or renewal that prompted this
    /// [`MessageType::RoleSubscriptionPurchase`] message.
    pub role_subscription_data: Option<RoleSubscriptionData>,
    // Field omitted: stickers (it's deprecated by Discord)
    /// The Id of the [`Guild`] that the message was sent in. This value will only be present if
    /// this message was received over the gateway, therefore **do not use this to check if message
    /// is in DMs**, it is not a reliable method.
    pub guild_id: Option<GuildId>,
    /// A partial amount of data about the user's member data
    ///
    /// Only present in [`MessageCreateEvent`].
    pub member: Option<Box<PartialMember>>,
    /// A poll that may be attached to a message.
    ///
    /// This is often omitted, so is boxed to improve memory usage.
    ///
    /// Only present in [`MessageCreateEvent`].
    pub poll: Option<Box<Poll>>,
}

#[cfg(feature = "model")]
impl Message {
    /// Crossposts this message.
    ///
    /// Requires either to be the message author or to have manage [Manage Messages] permissions on
    /// this channel.
    ///
    /// **Note**: Only available on news channels.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::MessageAlreadyCrossposted`] if the message has already been
    /// crossposted.
    ///
    /// Returns a [`ModelError::CannotCrosspostMessage`] if the message cannot be crossposted.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn crosspost(&self, http: &Http) -> Result<Message> {
        if let Some(flags) = self.flags {
            if flags.contains(MessageFlags::CROSSPOSTED) {
                return Err(Error::Model(ModelError::MessageAlreadyCrossposted));
            } else if flags.contains(MessageFlags::IS_CROSSPOST)
                || self.kind != MessageType::Regular
            {
                return Err(Error::Model(ModelError::CannotCrosspostMessage));
            }
        }

        self.channel_id.crosspost(http, self.id).await
    }

    /// First attempts to find a [`Channel`] by its Id in the cache, upon failure requests it via
    /// HTTP.
    ///
    /// # Errors
    ///
    /// Can return an error if the HTTP request fails.
    pub async fn channel(&self, cache_http: impl CacheHttp) -> Result<Channel> {
        self.channel_id.to_channel(cache_http, self.guild_id).await
    }

    /// First attempts to find the [`GuildChannel`] by it's Id in the cache, upon failure requests
    /// it via HTTP.
    ///
    /// # Errors
    ///
    /// Can return an error if the HTTP request fails, or this is executed in a DM channel.
    pub async fn guild_channel(&self, cache_http: impl CacheHttp) -> Result<GuildChannel> {
        self.channel_id.to_guild_channel(cache_http, self.guild_id).await
    }

    /// Calculates the permissions of the message author in the current channel.
    ///
    /// This handles the [`Permissions::SEND_MESSAGES_IN_THREADS`] permission for threads, setting
    /// [`Permissions::SEND_MESSAGES`] accordingly if this message was sent in a thread.
    ///
    /// This may return `None` if:
    /// - The [`Cache`] does not have the current [`Guild`]
    /// - The [`Guild`] does not have the current channel cached (should never happen).
    /// - This message is not from [`MessageCreateEvent`] and the author's [`Member`] cannot be
    ///   found in [`Guild#structfield.members`].
    #[cfg(feature = "cache")]
    pub fn author_permissions(&self, cache: &Cache) -> Option<Permissions> {
        let Some(guild_id) = self.guild_id else {
            return Some(Permissions::dm_permissions());
        };

        let guild = cache.guild(guild_id)?;
        let (channel, is_thread) = if let Some(channel) = guild.channels.get(&self.channel_id) {
            (channel, false)
        } else if let Some(thread) = guild.threads.iter().find(|th| th.id == self.channel_id) {
            (thread, true)
        } else {
            return None;
        };

        let mut permissions = if let Some(member) = &self.member {
            guild.partial_member_permissions_in(channel, self.author.id, member)
        } else {
            guild.user_permissions_in(channel, guild.members.get(&self.author.id)?)
        };

        if is_thread {
            permissions.set(Permissions::SEND_MESSAGES, permissions.send_messages_in_threads());
        }

        Some(permissions)
    }

    /// Deletes the message.
    ///
    /// **Note**: The logged in user must either be the author of the message or have the [Manage
    /// Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete(&self, http: &Http, reason: Option<&str>) -> Result<()> {
        self.channel_id.delete_message(http, self.id, reason).await
    }

    /// Deletes all of the [`Reaction`]s associated with the message.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete_reactions(&self, http: &Http) -> Result<()> {
        self.channel_id.delete_reactions(http, self.id).await
    }

    /// Deletes the given [`Reaction`] from the message.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current user did not perform
    /// the reaction.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user did not perform the reaction, or lacks
    /// permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete_reaction(
        &self,
        http: &Http,
        user_id: Option<UserId>,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        self.channel_id.delete_reaction(http, self.id, user_id, reaction_type).await
    }

    /// Deletes all of the [`Reaction`]s of a given emoji associated with the message.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn delete_reaction_emoji(
        &self,
        http: &Http,
        reaction_type: impl Into<ReactionType>,
    ) -> Result<()> {
        http.delete_message_reaction_emoji(self.channel_id, self.id, &reaction_type.into()).await
    }

    /// Edits this message, replacing the original content with new content.
    ///
    /// Message editing preserves all unchanged message data, with some exceptions for embeds and
    /// attachments.
    ///
    /// **Note**: In most cases requires that the current user be the author of the message.
    ///
    /// Refer to the documentation for [`EditMessage`] for information regarding content
    /// restrictions and requirements.
    ///
    /// # Examples
    ///
    /// Edit a message with new content:
    ///
    /// ```rust,no_run
    /// # use serenity::builder::EditMessage;
    /// # use serenity::model::channel::Message;
    /// # use serenity::model::id::ChannelId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let mut message: Message = unimplemented!();
    /// // assuming a `message` has already been bound
    /// let builder = EditMessage::new().content("new content");
    /// message.edit(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidUser`] if the current user is not
    /// the author. Otherwise returns [`Error::Http`] if the user lacks permission, as well as if
    /// invalid data is given.
    ///
    /// Returns a [`ModelError::TooLarge`] if the message contents are too long.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn edit(
        &mut self,
        cache_http: impl CacheHttp,
        builder: EditMessage<'_>,
    ) -> Result<()> {
        if let Some(flags) = self.flags {
            if flags.contains(MessageFlags::IS_VOICE_MESSAGE) {
                return Err(Error::Model(ModelError::CannotEditVoiceMessage));
            }
        }

        *self = builder.execute(cache_http, self.channel_id, self.id, Some(self.author.id)).await?;
        Ok(())
    }

    /// Returns message content, but with user and role mentions replaced with
    /// names and everyone/here mentions cancelled.
    #[cfg(all(feature = "cache", feature = "utils"))]
    pub fn content_safe(&self, cache: &Cache) -> String {
        let Some(guild) = self.guild(cache) else { return self.content.to_string() };

        let options = crate::utils::ContentSafeOptions::new()
            .clean_user(true)
            .clean_role(true)
            .clean_everyone(true);

        crate::utils::content_safe(&guild, &self.content, options, &self.mentions)
    }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different maximum number of
    /// users. The maximum that may be retrieve at a time is `100`, if a greater number is provided
    /// then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain user. This is
    /// useful for pagination.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// **Note**: If the passed reaction_type is a custom guild emoji, it must contain the name.
    /// So, [`Emoji`] or [`EmojiIdentifier`] will always work, [`ReactionType`] only if
    /// [`ReactionType::Custom::name`] is Some, and **[`EmojiId`] will never work**.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn reaction_users(
        &self,
        http: &Http,
        reaction_type: impl Into<ReactionType>,
        limit: Option<u8>,
        after: Option<UserId>,
    ) -> Result<Vec<User>> {
        self.channel_id.reaction_users(http, self.id, reaction_type, limit, after).await
    }

    /// Returns the associated [`Guild`] for the message if one is in the cache.
    ///
    /// Returns [`None`] if the guild's Id could not be found via [`Self::guild_id`] or if the
    /// Guild itself is not cached.
    ///
    /// Requires the `cache` feature be enabled.
    #[cfg(feature = "cache")]
    pub fn guild<'a>(&self, cache: &'a Cache) -> Option<GuildRef<'a>> {
        cache.guild(self.guild_id?)
    }

    /// Retrieves a clone of the author's Member instance, if this message was sent in a guild.
    ///
    /// If the instance cannot be found in the cache, or the `cache` feature is disabled, a HTTP
    /// request is performed to retrieve it from Discord's API.
    ///
    /// # Errors
    ///
    /// [`ModelError::ItemMissing`] is returned if [`Self::guild_id`] is [`None`].
    pub async fn member(&self, cache_http: impl CacheHttp) -> Result<Member> {
        match self.guild_id {
            Some(guild_id) => guild_id.member(cache_http, self.author.id).await,
            None => Err(Error::Model(ModelError::ItemMissing)),
        }
    }

    /// Checks the length of a message to ensure that it is within Discord's maximum length limit.
    ///
    /// Returns [`None`] if the message is within the limit, otherwise returns [`Some`] with an
    /// inner value of how many unicode code points the message is over.
    #[must_use]
    pub fn overflow_length(content: &str) -> Option<usize> {
        let char_count = content.chars().count();
        if char_count > constants::MESSAGE_CODE_LIMIT {
            Some(constants::MESSAGE_CODE_LIMIT - char_count)
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
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn pin(&self, http: &Http, reason: Option<&str>) -> Result<()> {
        self.channel_id.pin(http, self.id, reason).await
    }

    /// React to the message with a custom [`Emoji`] or unicode character.
    ///
    /// **Note**: Requires the [Add Reactions] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Add Reactions]: Permissions::ADD_REACTIONS
    pub async fn react(&self, http: &Http, reaction_type: impl Into<ReactionType>) -> Result<()> {
        http.create_reaction(self.channel_id, self.id, &reaction_type.into()).await
    }

    /// Uses Discord's inline reply to a user without pinging them.
    ///
    /// Refer to the documentation for [`CreateMessage`] for information regarding content
    /// restrictions and requirements.
    ///
    /// # Errors
    ///
    /// See the documentation of [`CreateMessage::execute`] for possible errors.
    pub async fn reply(&self, http: &Http, content: impl Into<Cow<'_, str>>) -> Result<Message> {
        self.reply_(http, content.into(), false).await
    }

    /// Uses Discord's inline reply to a user with a ping.
    ///
    /// Refer to the documentation for [`CreateMessage`] for information regarding content
    /// restrictions and requirements.
    ///
    /// # Errors
    ///
    /// See the documentation of [`CreateMessage::execute`] for possible errors.
    pub async fn reply_ping(
        &self,
        http: &Http,
        content: impl Into<Cow<'_, str>>,
    ) -> Result<Message> {
        self.reply_(http, content.into(), true).await
    }

    async fn reply_(&self, http: &Http, content: Cow<'_, str>, ping_user: bool) -> Result<Message> {
        let default_allowed_mentions = http.default_allowed_mentions.clone();
        let allowed_mentions = default_allowed_mentions.unwrap_or_else(|| {
            CreateAllowedMentions::new().everyone(true).all_users(true).all_roles(true)
        });

        let builder = CreateMessage::new()
            .content(content)
            .reference_message(self)
            .allowed_mentions(allowed_mentions.replied_user(ping_user));

        self.channel_id.send_message(http, builder).await
    }

    /// Checks whether the message mentions passed [`UserId`].
    #[must_use]
    pub fn mentions_user_id(&self, id: UserId) -> bool {
        self.mentions.iter().any(|mentioned_user| mentioned_user.id == id)
    }

    /// Checks whether the message mentions passed [`User`].
    #[must_use]
    pub fn mentions_user(&self, user: &User) -> bool {
        self.mentions_user_id(user.id)
    }

    /// Checks whether the message mentions the current user.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the `cache` feature is not enabled, or if the cache is
    /// otherwise unavailable.
    pub async fn mentions_me(&self, cache_http: impl CacheHttp) -> Result<bool> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                return Ok(self.mentions_user_id(cache.current_user().id));
            }
        }

        let current_user = cache_http.http().get_current_user().await?;
        Ok(self.mentions_user_id(current_user.id))
    }

    /// Unpins the message from its channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if invalid data is given.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    pub async fn unpin(&self, http: &Http, reason: Option<&str>) -> Result<()> {
        http.unpin_message(self.channel_id, self.id, reason).await
    }

    /// Ends the [`Poll`] on this message, if there is one.
    ///
    /// # Errors
    ///
    /// See [`ChannelId::end_poll`] for more information.
    pub async fn end_poll(&self, http: &Http) -> Result<Self> {
        self.channel_id.end_poll(http, self.id).await
    }

    /// Tries to return author's nickname in the current channel's guild.
    ///
    /// Refer to [`User::nick_in()`] inside and [`None`] outside of a guild.
    pub async fn author_nick(&self, cache_http: impl CacheHttp) -> Option<String> {
        self.author.nick_in(cache_http, self.guild_id?).await
    }

    /// Returns a link referencing this message. When clicked, users will jump to the message. The
    /// link will be valid for messages in either private channels or guilds.
    #[must_use]
    pub fn link(&self) -> String {
        self.id.link(self.channel_id, self.guild_id)
    }

    /// Retrieves the message channel's category ID if the channel has one.
    pub async fn category_id(&self, cache_http: impl CacheHttp) -> Option<ChannelId> {
        #[cfg(feature = "cache")]
        if let Some(cache) = cache_http.cache() {
            if let Some(guild) = cache.guild(self.guild_id?) {
                let channel = guild.channels.get(&self.channel_id)?;
                return if channel.thread_metadata.is_some() {
                    let thread_parent = guild.channels.get(&channel.parent_id?)?;
                    thread_parent.parent_id
                } else {
                    channel.parent_id
                };
            }
        }

        let http = cache_http.http();
        let channel = http.get_channel(self.channel_id).await.ok()?.guild()?;
        if channel.thread_metadata.is_some() {
            let thread_parent = http.get_channel(channel.parent_id?).await.ok()?.guild()?;
            thread_parent.parent_id
        } else {
            channel.parent_id
        }
    }
}

impl ExtractKey<MessageId> for Message {
    fn extract_key(&self) -> &MessageId {
        &self.id
    }
}

impl From<Message> for MessageId {
    /// Gets the Id of a [`Message`].
    fn from(message: Message) -> MessageId {
        message.id
    }
}

impl From<&Message> for MessageId {
    /// Gets the Id of a [`Message`].
    fn from(message: &Message) -> MessageId {
        message.id
    }
}

/// A representation of a reaction to a message.
///
/// Multiple of the same [reaction type] are sent into one [`MessageReaction`], with an associated
/// [`Self::count`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#reaction-object).
///
/// [reaction type]: ReactionType
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageReaction {
    /// The amount of the type of reaction that have been sent for the associated message
    /// including super reactions.
    pub count: u64,
    /// A breakdown of what reactions were from regular reactions and super reactions.
    pub count_details: CountDetails,
    /// Indicator of whether the current user has sent this type of reaction.
    pub me: bool,
    /// Indicator of whether the current user has sent the type of super-reaction.
    pub me_burst: bool,
    /// The type of reaction.
    #[serde(rename = "emoji")]
    pub reaction_type: ReactionType,
    // The colours used for super reactions.
    #[serde(rename = "burst_colors", deserialize_with = "discord_colours")]
    pub burst_colours: Vec<Colour>,
}

/// A representation of reaction count details.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#reaction-count-details-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CountDetails {
    pub burst: u64,
    pub normal: u64,
}

enum_number! {
    /// Differentiates between regular and different types of system messages.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#message-object-message-types).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
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
        /// An indicator that someone has boosted the guild.
        NitroBoost = 8,
        /// An indicator that the guild has reached nitro tier 1
        NitroTier1 = 9,
        /// An indicator that the guild has reached nitro tier 2
        NitroTier2 = 10,
        /// An indicator that the guild has reached nitro tier 3
        NitroTier3 = 11,
        /// An indicator that the channel is now following a news channel
        ChannelFollowAdd = 12,
        /// An indicator that the guild is disqualified for Discovery Feature
        GuildDiscoveryDisqualified = 14,
        /// An indicator that the guild is requalified for Discovery Feature
        GuildDiscoveryRequalified = 15,
        /// The first warning before guild discovery removal.
        GuildDiscoveryGracePeriodInitialWarning = 16,
        /// The last warning before guild discovery removal.
        GuildDiscoveryGracePeriodFinalWarning = 17,
        /// Message sent to inform users that a thread was created.
        ThreadCreated = 18,
        /// A message reply.
        InlineReply = 19,
        /// A slash command.
        ChatInputCommand = 20,
        /// A thread start message.
        ThreadStarterMessage = 21,
        /// Server setup tips.
        GuildInviteReminder = 22,
        /// A context menu command.
        ContextMenuCommand = 23,
        /// A message from an auto moderation action.
        AutoModAction = 24,
        RoleSubscriptionPurchase = 25,
        InteractionPremiumUpsell = 26,
        StageStart = 27,
        StageEnd = 28,
        StageSpeaker = 29,
        StageTopic = 31,
        GuildApplicationPremiumSubscription = 32,
        GuildIncidentAlertModeEnabled = 36,
        GuildIncidentAlertModeDisabled = 37,
        GuildIncidentReportRaid = 38,
        GuildIncidentReportFalseAlarm = 39,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#message-object-message-activity-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum MessageActivityKind {
        Join = 1,
        Spectate = 2,
        Listen = 3,
        JoinRequest = 5,
        _ => Unknown(u8),
    }
}

/// Rich Presence application information.
///
/// [Discord docs](https://discord.com/developers/docs/resources/application#application-object),
/// [subset undocumented](https://discord.com/developers/docs/resources/channel#message-object-message-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageApplication {
    /// ID of the application.
    pub id: ApplicationId,
    /// ID of the embed's image asset.
    pub cover_image: Option<ImageHash>,
    /// Application's description.
    pub description: FixedString,
    /// ID of the application's icon.
    pub icon: Option<ImageHash>,
    /// Name of the application.
    pub name: FixedString,
}

/// Rich Presence activity information.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#message-object-message-activity-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageActivity {
    /// Kind of message activity.
    #[serde(rename = "type")]
    pub kind: MessageActivityKind,
    /// `party_id` from a Rich Presence event.
    pub party_id: Option<FixedString>,
}

enum_number! {
    /// Message Reference Type information
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/message#message-reference-types)
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    #[<default> = 0]
    pub enum MessageReferenceKind {
        Default = 0,
        Forward = 1,
        _ => Unknown(u8),
    }
}

/// Reference data sent with crossposted messages.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#message-reference-object-message-reference-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MessageReference {
    /// The Type of Message Reference
    #[serde(rename = "type", default = "MessageReferenceKind::default")]
    pub kind: MessageReferenceKind,
    /// ID of the originating message.
    pub message_id: Option<MessageId>,
    /// ID of the originating message's channel.
    pub channel_id: ChannelId,
    /// ID of the originating message's guild.
    pub guild_id: Option<GuildId>,
    /// When sending, whether to error if the referenced message doesn't exist instead of sending
    /// as a normal (non-reply) message, default true.
    pub fail_if_not_exists: Option<bool>,
}

impl MessageReference {
    #[must_use]
    pub fn new(kind: MessageReferenceKind, channel_id: ChannelId) -> Self {
        Self {
            kind,
            channel_id,
            message_id: None,
            guild_id: None,
            fail_if_not_exists: None,
        }
    }

    #[must_use]
    pub fn message_id(mut self, message_id: MessageId) -> Self {
        self.message_id = Some(message_id);
        self
    }

    #[must_use]
    pub fn guild_id(mut self, guild_id: GuildId) -> Self {
        self.guild_id = Some(guild_id);
        self
    }

    #[must_use]
    pub fn fail_if_not_exists(mut self, fail_if_not_exists: bool) -> Self {
        self.fail_if_not_exists = Some(fail_if_not_exists);
        self
    }
}

impl From<&Message> for MessageReference {
    fn from(m: &Message) -> Self {
        Self {
            kind: MessageReferenceKind::default(),
            message_id: Some(m.id),
            channel_id: m.channel_id,
            guild_id: m.guild_id,
            fail_if_not_exists: None,
        }
    }
}

/// [Discord docs](https://discord.com/developers/docs/resources/channel#channel-mention-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChannelMention {
    /// ID of the channel.
    pub id: ChannelId,
    /// ID of the guild containing the channel.
    pub guild_id: GuildId,
    /// The kind of channel
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The name of the channel
    pub name: FixedString,
}

/// [Discord docs](https://discord.com/developers/docs/resources/message#message-snapshot-structure)
///
/// For field documentation, see [`Message`].
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MessageSnapshot {
    pub content: String,
    pub timestamp: Timestamp,
    pub edited_timestamp: Option<Timestamp>,
    pub mentions: Vec<User>,
    #[serde(default)]
    pub mention_roles: Vec<RoleId>,
    pub attachments: Vec<Attachment>,
    pub embeds: Vec<Embed>,
    #[serde(rename = "type")]
    pub kind: MessageType,
    pub flags: Option<MessageFlags>,
    #[serde(default)]
    pub components: Vec<ActionRow>,
    #[serde(default)]
    pub sticker_items: Vec<StickerItem>,
}

/// Custom deserialization function to handle the nested "message" field
fn deserialize_snapshots<'de, D>(deserializer: D) -> Result<Vec<MessageSnapshot>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct MessageSnapshotWrapper {
        pub message: MessageSnapshot,
    }

    let snapshots: Vec<MessageSnapshotWrapper> = Deserialize::deserialize(deserializer)?;

    let result = snapshots.into_iter().map(|wrapper| wrapper.message).collect();

    Ok(result)
}

bitflags! {
    /// Describes extra features of the message.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/channel#message-object-message-flags).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct MessageFlags: u64 {
        /// This message has been published to subscribed channels (via Channel Following).
        const CROSSPOSTED = 1 << 0;
        /// This message originated from a message in another channel (via Channel Following).
        const IS_CROSSPOST = 1 << 1;
        /// Do not include any embeds when serializing this message.
        const SUPPRESS_EMBEDS = 1 << 2;
        /// The source message for this crosspost has been deleted (via Channel Following).
        const SOURCE_MESSAGE_DELETED = 1 << 3;
        /// This message came from the urgent message system.
        const URGENT = 1 << 4;
        /// This message has an associated thread, with the same id as the message.
        const HAS_THREAD = 1 << 5;
        /// This message is only visible to the user who invoked the Interaction.
        const EPHEMERAL = 1 << 6;
        /// This message is an Interaction Response and the bot is "thinking".
        const LOADING = 1 << 7;
        /// This message failed to mention some roles and add their members to the thread.
        const FAILED_TO_MENTION_SOME_ROLES_IN_THREAD = 1 << 8;
        /// This message will not trigger push and desktop notifications.
        const SUPPRESS_NOTIFICATIONS = 1 << 12;
        /// This message is a voice message.
        ///
        /// Voice messages have the following properties:
        /// - They cannot be edited.
        /// - Only a single audio attachment is allowed. No content, stickers, etc...
        /// - The [`Attachment`] has additional fields: `duration_secs` and `waveform`.
        ///
        /// As of 2023-04-14, clients upload a 1 channel, 48000 Hz, 32kbps Opus stream in an OGG container.
        /// The encoding is a Discord implementation detail and may change without warning or documentation.
        ///
        /// As of 2023-04-20, bots are currently not able to send voice messages
        /// ([source](https://github.com/discord/discord-api-docs/pull/6082)).
        const IS_VOICE_MESSAGE = 1 << 13;
    }
}

#[cfg(feature = "model")]
impl MessageId {
    /// Returns a link referencing this message. When clicked, users will jump to the message. The
    /// link will be valid for messages in either private channels or guilds.
    #[must_use]
    pub fn link(self, channel_id: ChannelId, guild_id: Option<GuildId>) -> String {
        if let Some(guild_id) = guild_id {
            format!("https://discord.com/channels/{guild_id}/{channel_id}/{self}")
        } else {
            format!("https://discord.com/channels/@me/{channel_id}/{self}")
        }
    }
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Nonce {
    String(FixedString),
    Number(u64),
}

impl<'de> serde::Deserialize<'de> for Nonce {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        Ok(StrOrInt::deserialize(deserializer)?.into_enum(Self::String, Self::Number))
    }
}

/// [Discord docs](https://discord.com/developers/docs/resources/channel#role-subscription-data-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoleSubscriptionData {
    /// The id of the sku and listing that the user is subscribed to.
    pub role_subscription_listing_id: SkuId,
    /// The name of the tier that the user is subscribed to.
    pub tier_name: FixedString,
    /// The cumulative number of months that the user has been subscribed for.
    pub total_months_subscribed: u16,
    /// Whether this notification is for a renewal rather than a new purchase.
    pub is_renewal: bool,
}

/// A poll that has been attached to a [`Message`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/poll#poll-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Poll {
    pub question: PollMedia,
    pub answers: Vec<PollAnswer>,
    pub expiry: Option<Timestamp>,
    pub allow_multiselect: bool,
    pub layout_type: PollLayoutType,
    /// The results of the Poll.
    ///
    /// None does **not** mean that there are no results, simply that Discord has not provide them.
    /// See the discord docs for a more detailed explaination.
    pub results: Option<PollResults>,
}

/// A piece of data used in mutliple parts of the [`Poll`] structure.
///
/// Currently holds text and an optional emoji, but this is expected to change in future
///
/// [Discord docs](https://discord.com/developers/docs/resources/poll#poll-media-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollMedia {
    pub text: Option<String>,
    pub emoji: Option<PollMediaEmoji>,
}

/// The "Partial Emoji" attached to a [`PollMedia`] model.
///
/// [Discord docs](https://discord.com/developers/docs/resources/poll#poll-media-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PollMediaEmoji {
    Name(String),
    Id(EmojiId),
}

impl<'de> serde::Deserialize<'de> for PollMediaEmoji {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct RawPollMediaEmoji {
            name: Option<String>,
            id: Option<EmojiId>,
        }

        let raw = RawPollMediaEmoji::deserialize(deserializer)?;
        if let Some(name) = raw.name {
            Ok(PollMediaEmoji::Name(name))
        } else if let Some(id) = raw.id {
            Ok(PollMediaEmoji::Id(id))
        } else {
            Err(serde::de::Error::duplicate_field("emoji"))
        }
    }
}

impl From<String> for PollMediaEmoji {
    fn from(value: String) -> Self {
        Self::Name(value)
    }
}

impl From<EmojiId> for PollMediaEmoji {
    fn from(value: EmojiId) -> Self {
        Self::Id(value)
    }
}

/// A possible answer for a [`Poll`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/poll#poll-answer-object)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollAnswer {
    pub answer_id: AnswerId,
    pub poll_media: PollMedia,
}

enum_number! {
    /// Represents the different layouts that a [`Poll`] may have.
    ///
    /// Currently, there is only the one option.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/poll#layout-type)
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    #[<default> = 1]
    pub enum PollLayoutType {
        Default = 1,
        _ => Unknown(u8),
    }
}

/// The model for the results of a [`Poll`].
///
/// If `is_finalized` is `false`, `answer_counts` will be inaccurate due to Discord's scale.
///
/// [Discord docs](https://discord.com/developers/docs/resources/poll#poll-results-object-poll-results-object-structure)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollResults {
    pub is_finalized: bool,
    pub answer_counts: Vec<PollAnswerCount>,
}

/// The count of a single [`PollAnswer`]'s results.
///
/// [Discord docs](https://discord.com/developers/docs/resources/poll#poll-results-object-poll-answer-count-object-structure)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PollAnswerCount {
    pub id: AnswerId,
    pub count: u64,
    pub me_voted: bool,
}

// all tests here require cache, move if non-cache test is added
#[cfg(all(test, feature = "cache"))]
mod tests {
    use dashmap::DashMap;
    use extract_map::ExtractMap;
    use small_fixed_array::FixedArray;

    use super::{
        Guild,
        GuildChannel,
        Member,
        Message,
        PermissionOverwrite,
        PermissionOverwriteType,
        Permissions,
        User,
        UserId,
    };
    use crate::cache::Cache;
    use crate::cache::wrappers::MaybeMap;

    fn new_extract_map<K, T>(val: T) -> ExtractMap<K, T>
    where
        K: std::hash::Hash + Eq,
        T: extract_map::ExtractKey<K>,
    {
        let mut map = ExtractMap::new();
        map.insert(val);
        map
    }

    /// Test that author_permissions checks the permissions in a channel, not just the guild.
    #[test]
    fn author_permissions_respects_overwrites() {
        // Author of the message, with a random ID that won't collide with defaults.
        let author = User {
            id: UserId::new(50778944701071),
            ..Default::default()
        };

        // Channel with the message, with SEND_MESSAGES on.
        let channel = GuildChannel {
            permission_overwrites: FixedArray::from_vec_trunc(vec![PermissionOverwrite {
                allow: Permissions::SEND_MESSAGES,
                deny: Permissions::default(),
                kind: PermissionOverwriteType::Member(author.id),
            }]),
            ..Default::default()
        };
        let channel_id = channel.id;

        // Guild with the author and channel cached, default (empty) permissions.
        let guild = Guild {
            channels: new_extract_map(channel),
            members: new_extract_map(Member {
                user: author.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };

        // Message, tied to the guild and the channel.
        let message = Message {
            author,
            channel_id,
            guild_id: Some(guild.id),
            ..Default::default()
        };

        // Cache, with the guild setup.
        let mut cache = Cache::new();
        cache.guilds = MaybeMap(Some({
            let guilds = DashMap::default();
            guilds.insert(guild.id, guild);
            guilds
        }));

        // The author should only have the one permission, SEND_MESSAGES.
        assert_eq!(message.author_permissions(&cache), Some(Permissions::SEND_MESSAGES));
    }
}
