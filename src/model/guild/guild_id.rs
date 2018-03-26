use model::prelude::*;

impl GuildId {
    /// Converts the guild Id into the default channel's Id.
    #[inline]
    #[deprecated(note = "The concept of default channels is no more, use \
                         `Guild::default_channel{_guaranteed}` to simulate the
                         concept.")]
    pub fn as_channel_id(&self) -> ChannelId {
        ChannelId(self.0)
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total
    /// number of shards.
    ///
    /// When the cache is not enabled, the total number of shards being used
    /// will need to be passed.
    ///
    /// # Examples
    ///
    /// Retrieve the Id of the shard for a guild with Id `81384788765712384`,
    /// using 17 shards:
    ///
    /// ```rust
    /// use serenity::model::GuildId;
    /// use serenity::utils;
    ///
    /// let guild_id = GuildId(81384788765712384);
    ///
    /// assert_eq!(guild_id.shard_id(17), 7);
    /// ```
    #[cfg(feature = "utils")]
    #[inline]
    pub fn shard_id(&self, shard_count: u64) -> u64 {
        ::utils::shard_id(self.0, shard_count)
    }
}

impl From<PartialGuild> for GuildId {
    /// Gets the Id of a partial guild.
    fn from(guild: PartialGuild) -> GuildId { guild.id }
}

impl<'a> From<&'a PartialGuild> for GuildId {
    /// Gets the Id of a partial guild.
    fn from(guild: &PartialGuild) -> GuildId { guild.id }
}

impl From<GuildInfo> for GuildId {
    /// Gets the Id of Guild information struct.
    fn from(guild_info: GuildInfo) -> GuildId { guild_info.id }
}

impl<'a> From<&'a GuildInfo> for GuildId {
    /// Gets the Id of Guild information struct.
    fn from(guild_info: &GuildInfo) -> GuildId { guild_info.id }
}

impl From<InviteGuild> for GuildId {
    /// Gets the Id of Invite Guild struct.
    fn from(invite_guild: InviteGuild) -> GuildId { invite_guild.id }
}

impl<'a> From<&'a InviteGuild> for GuildId {
    /// Gets the Id of Invite Guild struct.
    fn from(invite_guild: &InviteGuild) -> GuildId { invite_guild.id }
}

impl From<Guild> for GuildId {
    /// Gets the Id of Guild.
    fn from(live_guild: Guild) -> GuildId { live_guild.id }
}

impl<'a> From<&'a Guild> for GuildId {
    /// Gets the Id of Guild.
    fn from(live_guild: &Guild) -> GuildId { live_guild.id }
}
