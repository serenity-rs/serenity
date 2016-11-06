use super::*;
use ::client::{STATE, http};
use ::prelude::*;

impl ChannelId {
    /// Search the state for the channel with the Id.
    pub fn find(&self) -> Option<Channel> {
        STATE.lock().unwrap().find_channel(*self)
    }

    /// Search the state for the channel. If it can't be found, the channel is
    /// requested over REST.
    pub fn get(&self) -> Result<Channel> {
        if let Some(channel) = STATE.lock().unwrap().find_channel(*self) {
            return Ok(channel);
        }

        http::get_channel(self.0)
    }

    /// Returns a [`Mention`] which will link to the [`Channel`].
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`Mention`]: struct.Mention.html
    pub fn mention(&self) -> Mention {
        Mention {
            id: self.0,
            prefix: "<#",
        }
    }
}

impl From<Channel> for ChannelId {
    fn from(channel: Channel) -> ChannelId {
        match channel {
            Channel::Group(group) => group.channel_id,
            Channel::Private(channel) => channel.id,
            Channel::Public(channel) => channel.id,
        }
    }
}

impl From<PrivateChannel> for ChannelId {
    fn from(private_channel: PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl From<PublicChannel> for ChannelId {
    fn from(public_channel: PublicChannel) -> ChannelId {
        public_channel.id
    }
}

impl From<Emoji> for EmojiId {
    fn from(emoji: Emoji) -> EmojiId {
        emoji.id
    }
}

impl GuildId {
    /// Search the state for the guild.
    pub fn find(&self) -> Option<LiveGuild> {
        STATE.lock().unwrap().find_guild(*self).cloned()
    }

    /// Requests the guild over REST.
    ///
    /// Note that this will not be a complete guild, as REST does not send
    /// all data with a guild retrieval.
    pub fn get(&self) -> Result<Guild> {
        http::get_guild(self.0)
    }

    /// Mentions the [`Guild`]'s default channel.
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn mention(&self) -> Mention {
        Mention {
            id: self.0,
            prefix: "<#",
        }
    }
}

impl From<Guild> for GuildId {
    fn from(guild: Guild) -> GuildId {
        guild.id
    }
}

impl From<GuildInfo> for GuildId {
    fn from(guild_info: GuildInfo) -> GuildId {
        guild_info.id
    }
}

impl From<InviteGuild> for GuildId {
    fn from(invite_guild: InviteGuild) -> GuildId {
        invite_guild.id
    }
}

impl From<LiveGuild> for GuildId {
    fn from(live_guild: LiveGuild) -> GuildId {
        live_guild.id
    }
}

impl From<Integration> for IntegrationId {
    fn from(integration: Integration) -> IntegrationId {
        integration.id
    }
}

impl From<Message> for MessageId {
    fn from(message: Message) -> MessageId {
        message.id
    }
}

impl From<Role> for RoleId {
    fn from(role: Role) -> RoleId {
        role.id
    }
}

impl RoleId {
    /// Search the state for the role.
    pub fn find(&self) -> Option<Role> {
        STATE.lock()
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

    /// Returns a [`Mention`] which will ping members of the role.
    ///
    /// [`Mention`]: struct.Mention.html
    pub fn mention(&self) -> Mention {
        Mention {
            id: self.0,
            prefix: "<@&",
        }
    }
}

impl From<CurrentUser> for UserId {
    fn from(current_user: CurrentUser) -> UserId {
        current_user.id
    }
}

impl From<Member> for UserId {
    fn from(member: Member) -> UserId {
        member.user.id
    }
}

impl From<User> for UserId {
    fn from(user: User) -> UserId {
        user.id
    }
}

impl UserId {
    /// Returns a [`Mention`] which will ping the user.
    ///
    /// [`Mention`]: struct.Mention.html
    pub fn mention(&self) -> Mention {
        Mention {
            id: self.0,
            prefix: "<@",
        }
    }
}
