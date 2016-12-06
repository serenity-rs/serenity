use super::*;
use std::fmt;

#[cfg(all(feature = "cache", feature = "methods"))]
use ::client::CACHE;
#[cfg(feature = "methods")]
use ::client::rest;
#[cfg(feature = "methods")]
use ::internal::prelude::*;

impl ChannelId {
    /// Search the cache for the channel with the Id.
    #[cfg(all(feature = "cache", feature = "methods"))]
    pub fn find(&self) -> Option<Channel> {
        CACHE.read().unwrap().get_channel(*self).map(|x| x.clone_inner())
    }

    /// Search the cache for the channel. If it can't be found, the channel is
    /// requested over REST.
    #[cfg(feature="methods")]
    pub fn get(&self) -> Result<Channel> {
        feature_cache_enabled! {{
            if let Some(channel) = CACHE.read().unwrap().get_channel(*self) {
                return Ok(channel.clone_inner());
            }
        }}

        rest::get_channel(self.0)
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[cfg(feature="methods")]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> {
        rest::get_channel_webhooks(self.0)
    }
}

impl From<Channel> for ChannelId {
    /// Gets the Id of a `Channel`.
    fn from(channel: Channel) -> ChannelId {
        match channel {
            Channel::Group(group) => group.channel_id,
            Channel::Guild(channel) => channel.id,
            Channel::Private(channel) => channel.id,
        }
    }
}

impl From<PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl From<GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: GuildChannel) -> ChannelId {
        public_channel.id
    }
}

impl From<Emoji> for EmojiId {
    /// Gets the Id of an `Emoji`.
    fn from(emoji: Emoji) -> EmojiId {
        emoji.id
    }
}

impl GuildId {
    /// Search the cache for the guild.
    #[cfg(all(feature = "cache", feature="methods"))]
    pub fn find(&self) -> Option<Guild> {
        CACHE.read().unwrap().get_guild(*self).cloned()
    }

    /// Requests the guild over REST.
    ///
    /// Note that this will not be a complete guild, as REST does not send
    /// all data with a guild retrieval.
    #[cfg(feature="methods")]
    pub fn get(&self) -> Result<PartialGuild> {
        rest::get_guild(self.0)
    }

    /// Returns this Id as a `ChannelId`, which is useful when needing to use
    /// the guild Id to send a message to the default channel.
    #[cfg(feature = "methods")]
    pub fn to_channel(&self) -> ChannelId {
        ChannelId(self.0)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[cfg(feature="methods")]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> {
        rest::get_guild_webhooks(self.0)
    }
}

impl From<PartialGuild> for GuildId {
    /// Gets the Id of a partial guild.
    fn from(guild: PartialGuild) -> GuildId {
        guild.id
    }
}

impl From<GuildInfo> for GuildId {
    /// Gets the Id of Guild information struct.
    fn from(guild_info: GuildInfo) -> GuildId {
        guild_info.id
    }
}

impl From<InviteGuild> for GuildId {
    /// Gets the Id of Invite Guild struct.
    fn from(invite_guild: InviteGuild) -> GuildId {
        invite_guild.id
    }
}

impl From<Guild> for GuildId {
    /// Gets the Id of Guild.
    fn from(live_guild: Guild) -> GuildId {
        live_guild.id
    }
}

impl From<Integration> for IntegrationId {
    /// Gets the Id of integration.
    fn from(integration: Integration) -> IntegrationId {
        integration.id
    }
}

impl From<Message> for MessageId {
    /// Gets the Id of a `Message`.
    fn from(message: Message) -> MessageId {
        message.id
    }
}

impl From<Role> for RoleId {
    /// Gets the Id of a `Role`.
    fn from(role: Role) -> RoleId {
        role.id
    }
}

impl RoleId {
    /// Search the cache for the role.
    #[cfg(all(feature = "cache", feature = "methods"))]
    pub fn find(&self) -> Option<Role> {
        CACHE.read()
            .unwrap()
            .guilds
            .values()
            .find(|guild| guild.roles.contains_key(self))
            .map(|guild| guild.roles.get(self))
            .and_then(|v| match v {
                Some(v) => Some(v),
                None => None,
            })
            .cloned()
    }
}

impl From<CurrentUser> for UserId {
    /// Gets the Id of a `CurrentUser` struct.
    fn from(current_user: CurrentUser) -> UserId {
        current_user.id
    }
}

impl From<Member> for UserId {
    /// Gets the Id of a `Member`.
    fn from(member: Member) -> UserId {
        member.user.id
    }
}

impl From<User> for UserId {
    /// Gets the Id of a `User`.
    fn from(user: User) -> UserId {
        user.id
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}

impl fmt::Display for RoleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}

impl fmt::Display for GuildId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Display for EmojiId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl WebhookId {
    /// Retrieves the webhook by the Id.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[cfg(feature="methods")]
    pub fn webhooks(&self) -> Result<Webhook> {
        rest::get_webhook(self.0)
    }
}
