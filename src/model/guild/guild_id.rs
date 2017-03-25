use serde_json::builder::ObjectBuilder;
use std::fmt::{Display, Formatter, Result as FmtResult};
use ::client::{CACHE, rest};
use ::internal::prelude::*;
use ::model::*;
use ::utils::builder::{EditGuild, EditMember, EditRole, Search};

impl GuildId {
    /// Converts the guild Id into the default channel's Id.
    #[inline]
    pub fn as_channel_id(&self) -> ChannelId {
        ChannelId(self.0)
    }

    /// Ban a [`User`] from the guild. All messages by the
    /// user within the last given number of days given will be deleted.
    ///
    /// Refer to the documentation for [`Guild::ban`] for more information.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Examples
    ///
    /// Ban a member and remove all messages they've sent in the last 4 days:
    ///
    /// ```rust,ignore
    /// use serenity::model::GuildId;
    ///
    /// // assuming a `user` has already been bound
    /// let _ = GuildId(81384788765712384).ban(user, 4);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::DeleteMessageDaysAmount`] if the number of
    /// days' worth of messages to delete is over the maximum.
    ///
    /// [`ClientError::DeleteMessageDaysAmount`]: ../client/enum.ClientError.html#variant.DeleteMessageDaysAmount
    /// [`Guild::ban`]: struct.Guild.html#method.ban
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    pub fn ban<U: Into<UserId>>(&self, user: U, delete_message_days: u8)
        -> Result<()> {
        if delete_message_days > 7 {
            return Err(Error::Client(ClientError::DeleteMessageDaysAmount(delete_message_days)));
        }

        rest::ban_user(self.0, user.into().0, delete_message_days)
    }

    /// Creates a [`GuildChannel`] in the the guild.
    ///
    /// Refer to [`rest::create_channel`] for more information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Create a voice channel in a guild with the name `test`:
    ///
    /// ```rust,ignore
    /// use serenity::model::{ChannelType, GuildId};
    ///
    /// let _channel = GuildId(7).create_channel("test", ChannelType::Voice);
    /// ```
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`rest::create_channel`]: ../client/rest/fn.create_channel.html
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    pub fn create_channel(&self, name: &str, kind: ChannelType) -> Result<GuildChannel> {
        let map = ObjectBuilder::new()
            .insert("name", name)
            .insert("type", kind.name())
            .build();

        rest::create_channel(self.0, &map)
    }

    /// Creates an emoji in the guild with a name and base64-encoded image.
    ///
    /// Refer to the documentation for [`Guild::create_emoji`] for more
    /// information.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// # Examples
    ///
    /// See the [`EditProfile::avatar`] example for an in-depth example as to
    /// how to read an image from the filesystem and encode it as base64. Most
    /// of the example can be applied similarly for this method.
    ///
    /// [`EditProfile::avatar`]: ../utils/builder/struct.EditProfile.html#method.avatar
    /// [`Guild::create_emoji`]: struct.Guild.html#method.create_emoji
    /// [`utils::read_image`]: ../utils/fn.read_image.html
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    pub fn create_emoji(&self, name: &str, image: &str) -> Result<Emoji> {
        let map = ObjectBuilder::new()
            .insert("name", name)
            .insert("image", image)
            .build();

        rest::create_emoji(self.0, &map)
    }

    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn create_integration<I>(&self, integration_id: I, kind: &str)
        -> Result<()> where I: Into<IntegrationId> {
        let integration_id = integration_id.into();
        let map = ObjectBuilder::new()
            .insert("id", integration_id.0)
            .insert("type", kind)
            .build();

        rest::create_guild_integration(self.0, integration_id.0, &map)
    }

    /// Creates a new role in the guild with the data set, if any.
    ///
    /// See the documentation for [`Guild::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Guild::create_role`]: struct.Guild.html#method.create_role
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[inline]
    pub fn create_role<F: FnOnce(EditRole) -> EditRole>(&self, f: F) -> Result<Role> {
        rest::create_role(self.0, &f(EditRole::default()).0.build())
    }

    /// Deletes the current guild if the current account is the owner of the
    /// guild.
    ///
    /// Refer to [`Guild::delete`] for more information.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    ///
    /// [`Guild::delete`]: struct.Guild.html#method.delete
    #[inline]
    pub fn delete(&self) -> Result<PartialGuild> {
        rest::delete_guild(self.0)
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn delete_emoji<E: Into<EmojiId>>(&self, emoji_id: E) -> Result<()> {
        rest::delete_emoji(self.0, emoji_id.into().0)
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn delete_integration<I: Into<IntegrationId>>(&self, integration_id: I) -> Result<()> {
        rest::delete_guild_integration(self.0, integration_id.into().0)
    }

    /// Deletes a [`Role`] by Id from the guild.
    ///
    /// Also see [`Role::delete`] if you have the `cache` and `methods` features
    /// enabled.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [`Role::delete`]: struct.Role.html#method.delete
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[inline]
    pub fn delete_role<R: Into<RoleId>>(&self, role_id: R) -> Result<()> {
        rest::delete_role(self.0, role_id.into().0)
    }

    /// Edits the current guild with new data where specified.
    ///
    /// Refer to [`Guild::edit`] for more information.
    ///
    /// **Note**: Requires the current user to have the [Manage Guild]
    /// permission.
    ///
    /// [`Guild::edit`]: struct.Guild.html#method.edit
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn edit<F: FnOnce(EditGuild) -> EditGuild>(&mut self, f: F) -> Result<PartialGuild> {
        rest::edit_guild(self.0, &f(EditGuild::default()).0.build())
    }

    /// Edits an [`Emoji`]'s name in the guild.
    ///
    /// Also see [`Emoji::edit`] if you have the `cache` and `methods` features
    /// enabled.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [`Emoji::edit`]: struct.Emoji.html#method.edit
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    pub fn edit_emoji<E: Into<EmojiId>>(&self, emoji_id: E, name: &str) -> Result<Emoji> {
        let map = ObjectBuilder::new().insert("name", name).build();

        rest::edit_emoji(self.0, emoji_id.into().0, &map)
    }

    /// Edits the properties of member of the guild, such as muting or
    /// nicknaming them.
    ///
    /// Refer to `EditMember`'s documentation for a full list of methods and
    /// permission restrictions.
    ///
    /// # Examples
    ///
    /// Mute a member and set their roles to just one role with a predefined Id:
    ///
    /// ```rust,ignore
    /// guild.edit_member(user_id, |m| m.mute(true).roles(&vec![role_id]));
    /// ```
    #[inline]
    pub fn edit_member<F, U>(&self, user_id: U, f: F) -> Result<()>
        where F: FnOnce(EditMember) -> EditMember, U: Into<UserId> {
        rest::edit_member(self.0, user_id.into().0, &f(EditMember::default()).0.build())
    }

    /// Edits the current user's nickname for the guild.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// Requires the [Change Nickname] permission.
    ///
    /// [Change Nickname]: permissions/constant.CHANGE_NICKNAME.html
    #[inline]
    pub fn edit_nickname(&self, new_nickname: Option<&str>) -> Result<()> {
        rest::edit_nickname(self.0, new_nickname)
    }

    /// Edits a [`Role`], optionally setting its new fields.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Make a role hoisted:
    ///
    /// ```rust,ignore
    /// use serenity::model::{GuildId, RoleId};
    ///
    /// GuildId(7).edit_role(RoleId(8), |r| r.hoist(true));
    /// ```
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[inline]
    pub fn edit_role<F, R>(&self, role_id: R, f: F) -> Result<Role>
        where F: FnOnce(EditRole) -> EditRole, R: Into<RoleId> {
        rest::edit_role(self.0, role_id.into().0, &f(EditRole::default()).0.build())
    }

    /// Search the cache for the guild.
    #[cfg(feature="cache")]
    pub fn find(&self) -> Option<Arc<RwLock<Guild>>> {
        CACHE.read().unwrap().get_guild(*self)
    }

    /// Requests the guild over REST.
    ///
    /// Note that this will not be a complete guild, as REST does not send
    /// all data with a guild retrieval.
    #[inline]
    pub fn get(&self) -> Result<PartialGuild> {
        rest::get_guild(self.0)
    }

    /// Gets a list of the guild's bans.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[inline]
    pub fn get_bans(&self) -> Result<Vec<Ban>> {
        rest::get_bans(self.0)
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn get_channels(&self) -> Result<HashMap<ChannelId, GuildChannel>> {
        let mut channels = HashMap::new();

        for channel in rest::get_channels(self.0)? {
            channels.insert(channel.id, channel);
        }

        Ok(channels)
    }

    /// Gets an emoji in the guild by Id.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn get_emoji<E: Into<EmojiId>>(&self, emoji_id: E) -> Result<Emoji> {
        rest::get_emoji(self.0, emoji_id.into().0)
    }

    /// Gets a list of all of the guild's emojis.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn get_emojis(&self) -> Result<Vec<Emoji>> {
        rest::get_emojis(self.0)
    }

    /// Gets all integration of the guild.
    ///
    /// This performs a request over the REST API.
    #[inline]
    pub fn get_integrations(&self) -> Result<Vec<Integration>> {
        rest::get_guild_integrations(self.0)
    }

    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/struct.MANAGE_GUILD.html
    #[inline]
    pub fn get_invites(&self) -> Result<Vec<RichInvite>> {
        rest::get_guild_invites(self.0)
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`Member`]: struct.Member.html
    #[inline]
    pub fn get_member<U: Into<UserId>>(&self, user_id: U) -> Result<Member> {
        rest::get_member(self.0, user_id.into().0)
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Maximum
    /// value is 1000. Optionally pass in `after` to offset the results by a
    /// [`User`]'s Id.
    ///
    /// [`User`]: struct.User.html
    #[inline]
    pub fn get_members<U>(&self, limit: Option<u64>, after: Option<U>)
        -> Result<Vec<Member>> where U: Into<UserId> {
        rest::get_guild_members(self.0, limit, after.map(|x| x.into().0))
    }

    /// Gets the number of [`Member`]s that would be pruned with the given
    /// number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    pub fn get_prune_count(&self, days: u16) -> Result<GuildPrune> {
        let map = ObjectBuilder::new().insert("days", days).build();

        rest::get_guild_prune_count(self.0, &map)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn get_webhooks(&self) -> Result<Vec<Webhook>> {
        rest::get_guild_webhooks(self.0)
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    #[inline]
    pub fn kick<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        rest::kick_member(self.0, user_id.into().0)
    }

    /// Leaves the guild.
    #[inline]
    pub fn leave(&self) -> Result<PartialGuild> {
        rest::leave_guild(self.0)
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: permissions/constant.MOVE_MEMBERS.html
    pub fn move_member<C, U>(&self, user_id: U, channel_id: C)
        -> Result<()> where C: Into<ChannelId>, U: Into<UserId> {
        let map = ObjectBuilder::new().insert("channel_id", channel_id.into().0).build();

        rest::edit_member(self.0, user_id.into().0, &map)
    }

    /// Performs a search request to the API for the guild's [`Message`]s.
    ///
    /// This will search all of the guild's [`Channel`]s at once, that you have
    /// the [Read Message History] permission to. Use [`search_channels`] to
    /// specify a list of [channel][`GuildChannel`]s to search, where all other
    /// channels will be excluded.
    ///
    /// Refer to the documentation for the [`Search`] builder for examples and
    /// more information.
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`Message`]: struct.Message.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    /// [`search_channels`]: #method.search_channels
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn search<F: FnOnce(Search) -> Search>(&self, f: F) -> Result<SearchResult> {
        rest::search_guild_messages(self.0, &[], f(Search::default()).0)
    }

    /// Performs a search request to the API for the guild's [`Message`]s in
    /// given channels.
    ///
    /// Refer to [`Guild::search_channels`] for more information.
    ///
    /// Refer to the documentation for the [`Search`] builder for examples and
    /// more information.
    ///
    /// **Note**: Bot users can not search.
    ///
    /// [`Guild::search_channels`]: struct.Guild.html#method.search_channels
    /// [`Message`]: struct.Message.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    pub fn search_channels<F>(&self, channel_ids: &[ChannelId], f: F)
        -> Result<SearchResult> where F: FnOnce(Search) -> Search {
        let ids = channel_ids.iter().map(|x| x.0).collect::<Vec<u64>>();

        rest::search_guild_messages(self.0, &ids, f(Search::default()).0)
    }

    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn start_integration_sync<I: Into<IntegrationId>>(&self, integration_id: I) -> Result<()> {
        rest::start_integration_sync(self.0, integration_id.into().0)
    }

    /// Starts a prune of [`Member`]s.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// [`GuildPrune`]: struct.GuildPrune.html
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    #[inline]
    pub fn start_prune(&self, days: u16) -> Result<GuildPrune> {
        rest::start_guild_prune(self.0, &ObjectBuilder::new().insert("days", days).build())
    }

    /// Unbans a [`User`] from the guild.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[inline]
    pub fn unban<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        rest::remove_ban(self.0, user_id.into().0)
    }
}

impl Display for GuildId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.0, f)
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
