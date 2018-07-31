use model::prelude::*;

#[cfg(all(feature = "cache", feature = "model"))]
use CACHE;
#[cfg(feature = "model")]
use builder::{EditGuild, EditMember, EditRole};
#[cfg(feature = "model")]
use internal::prelude::*;
#[cfg(feature = "model")]
use model::guild::BanOptions;
#[cfg(feature = "model")]
use {http, utils};

#[cfg(feature = "model")]
impl GuildId {
    /// Converts the guild Id into the default channel's Id.
    #[inline]
    #[deprecated(note = "The concept of default channels is no more, use \
                         `Guild::default_channel{_guaranteed}` to simulate the
                         concept.")]
    pub fn as_channel_id(&self) -> ChannelId { ChannelId(self.0) }

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
    /// Returns a [`ModelError::DeleteMessageDaysAmount`] if the number of
    /// days' worth of messages to delete is over the maximum.
    ///
    /// [`ModelError::DeleteMessageDaysAmount`]:
    /// enum.ModelError.html#variant.DeleteMessageDaysAmount
    /// [`Guild::ban`]: struct.Guild.html#method.ban
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[inline]
    pub fn ban<U, BO>(&self, user: U, ban_options: &BO) -> Result<()>
        where U: Into<UserId>, BO: BanOptions {
        self._ban(user.into(), (ban_options.dmd(), ban_options.reason()))
    }

    fn _ban(self, user: UserId, ban_options: (u8, &str)) -> Result<()> {
        let (dmd, reason) = ban_options;

        if dmd > 7 {
            return Err(Error::Model(ModelError::DeleteMessageDaysAmount(dmd)));
        }

        if reason.len() > 512 {
            return Err(Error::ExceededLimit(reason.to_string(), 512));
        }

        http::ban_user(self.0, user.0, dmd, reason)
    }

    /// Gets a list of the guild's bans.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[inline]
    pub fn bans(&self) -> Result<Vec<Ban>> { http::get_bans(self.0) }

    /// Gets a list of the guild's audit log entries
    #[inline]
    pub fn audit_logs(&self, action_type: Option<u8>,
                             user_id: Option<UserId>,
                             before: Option<AuditLogEntryId>,
                             limit: Option<u8>) -> Result<AuditLogs> {
        http::get_audit_logs(self.0, action_type, user_id.map(|u| u.0), before.map(|a| a.0), limit)
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn channels(&self) -> Result<HashMap<ChannelId, GuildChannel>> {
        let mut channels = HashMap::new();

        for channel in http::get_channels(self.0)? {
            channels.insert(channel.id, channel);
        }

        Ok(channels)
    }

    /// Creates a [`GuildChannel`] in the the guild.
    ///
    /// Refer to [`http::create_channel`] for more information.
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
    /// let _channel = GuildId(7).create_channel("test", ChannelType::Voice, None);
    /// ```
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`http::create_channel`]: ../http/fn.create_channel.html
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn create_channel<C>(&self, name: &str, kind: ChannelType, category: C) -> Result<GuildChannel>
        where C: Into<Option<ChannelId>> {
        self._create_channel(name, kind, category.into())
    }

    fn _create_channel(
        self,
        name: &str,
        kind: ChannelType,
        category: Option<ChannelId>,
    ) -> Result<GuildChannel> {
        let map = json!({
            "name": name,
            "type": kind as u8,
            "parent_id": category.map(|c| c.0)
        });

        http::create_channel(self.0, &map)
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
    /// [`EditProfile::avatar`]: ../builder/struct.EditProfile.html#method.avatar
    /// [`Guild::create_emoji`]: struct.Guild.html#method.create_emoji
    /// [`utils::read_image`]: ../utils/fn.read_image.html
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn create_emoji(&self, name: &str, image: &str) -> Result<Emoji> {
        let map = json!({
            "name": name,
            "image": image,
        });

        http::create_emoji(self.0, &map)
    }

    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn create_integration<I>(&self, integration_id: I, kind: &str) -> Result<()>
        where I: Into<IntegrationId> {
        self._create_integration(integration_id.into(), kind)
    }

    fn _create_integration(
        self,
        integration_id: IntegrationId,
        kind: &str,
    ) -> Result<()> {
        let map = json!({
            "id": integration_id.0,
            "type": kind,
        });

        http::create_guild_integration(self.0, integration_id.0, &map)
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
        let map = utils::vecmap_to_json_map(f(EditRole::default()).0);

        let role = http::create_role(self.0, &map)?;

        if let Some(position) = map.get("position").and_then(Value::as_u64) {
            self.edit_role_position(role.id, position)?;
        }

        Ok(role)
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
    pub fn delete(&self) -> Result<PartialGuild> { http::delete_guild(self.0) }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn delete_emoji<E: Into<EmojiId>>(&self, emoji_id: E) -> Result<()> {
        self._delete_emoji(emoji_id.into())
    }

    fn _delete_emoji(self, emoji_id: EmojiId) -> Result<()> {
        http::delete_emoji(self.0, emoji_id.0)
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn delete_integration<I: Into<IntegrationId>>(&self, integration_id: I) -> Result<()> {
        self._delete_integration(integration_id.into())
    }

    fn _delete_integration(self, integration_id: IntegrationId) -> Result<()> {
        http::delete_guild_integration(self.0, integration_id.0)
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
        self._delete_role(role_id.into())
    }

    fn _delete_role(self, role_id: RoleId) -> Result<()> {
        http::delete_role(self.0, role_id.0)
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
        let map = utils::vecmap_to_json_map(f(EditGuild::default()).0);

        http::edit_guild(self.0, &map)
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
    #[inline]
    pub fn edit_emoji<E: Into<EmojiId>>(&self, emoji_id: E, name: &str) -> Result<Emoji> {
        self._edit_emoji(emoji_id.into(), name)
    }

    fn _edit_emoji(self, emoji_id: EmojiId, name: &str) -> Result<Emoji> {
        let map = json!({
            "name": name,
        });

        http::edit_emoji(self.0, emoji_id.0, &map)
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
        self._edit_member(user_id.into(), f)
    }

    fn _edit_member<F>(self, user_id: UserId, f: F) -> Result<()>
        where F: FnOnce(EditMember) -> EditMember {
        let map = utils::vecmap_to_json_map(f(EditMember::default()).0);

        http::edit_member(self.0, user_id.0, &map)
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
        http::edit_nickname(self.0, new_nickname)
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
        self._edit_role(role_id.into(), f)
    }

    fn _edit_role<F>(self, role_id: RoleId, f: F) -> Result<Role>
        where F: FnOnce(EditRole) -> EditRole {
        let map = utils::vecmap_to_json_map(f(EditRole::default()).0);

        http::edit_role(self.0, role_id.0, &map)
    }

    /// Edits the order of [`Role`]s
    /// Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Change the order of a role:
    ///
    /// ```rust,ignore
    /// use serenity::model::{GuildId, RoleId};
    /// GuildId(7).edit_role_position(RoleId(8), 2);
    /// ```
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[inline]
    pub fn edit_role_position<R>(&self, role_id: R, position: u64) -> Result<Vec<Role>>
        where R: Into<RoleId> {
        self._edit_role_position(role_id.into(), position)
    }

    fn _edit_role_position(
        &self,
        role_id: RoleId,
        position: u64,
    ) -> Result<Vec<Role>> {
        http::edit_role_position(self.0, role_id.0, position)
    }


    /// Search the cache for the guild.
    #[cfg(feature = "cache")]
    pub fn find(&self) -> Option<Arc<RwLock<Guild>>> { CACHE.read().guild(*self) }

    /// Requests the guild over REST.
    ///
    /// Note that this will not be a complete guild, as REST does not send
    /// all data with a guild retrieval.
    #[inline]
    pub fn get(&self) -> Result<PartialGuild> { http::get_guild(self.0) }

    /// Gets all integration of the guild.
    ///
    /// This performs a request over the REST API.
    #[inline]
    pub fn integrations(&self) -> Result<Vec<Integration>> { http::get_guild_integrations(self.0) }

    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/struct.MANAGE_GUILD.html
    #[inline]
    pub fn invites(&self) -> Result<Vec<RichInvite>> { http::get_guild_invites(self.0) }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    #[inline]
    pub fn kick<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        http::kick_member(self.0, user_id.into().0)
    }

    /// Leaves the guild.
    #[inline]
    pub fn leave(&self) -> Result<()> { http::leave_guild(self.0) }

    /// Gets a user's [`Member`] for the guild by Id. 
    /// 
    /// If the cache feature is enabled the cache will be checked
    /// first. If not found it will resort to an http request.
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`Member`]: struct.Member.html
    #[inline]
    pub fn member<U: Into<UserId>>(&self, user_id: U) -> Result<Member> {
        self._member(user_id.into())
    }

    fn _member(&self, user_id: UserId) -> Result<Member> {
        #[cfg(feature = "cache")]
        {
            if let Some(member) = CACHE.read().member(self.0, user_id) {
                return Ok(member);
            }
        }

        http::get_member(self.0, user_id.0)
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Maximum
    /// value is 1000. Optionally pass in `after` to offset the results by a
    /// [`User`]'s Id.
    ///
    /// [`User`]: struct.User.html
    #[inline]
    pub fn members<U>(&self, limit: Option<u64>, after: Option<U>) -> Result<Vec<Member>>
        where U: Into<UserId> {
        self._members(limit, after.map(Into::into))
    }

    fn _members(&self, limit: Option<u64>, after: Option<UserId>) -> Result<Vec<Member>> {
        http::get_guild_members(self.0, limit, after.map(|x| x.0))
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: permissions/constant.MOVE_MEMBERS.html
    #[inline]
    pub fn move_member<C, U>(&self, user_id: U, channel_id: C) -> Result<()>
        where C: Into<ChannelId>, U: Into<UserId> {
        self._move_member(user_id.into(), channel_id.into())
    }

    fn _move_member(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> Result<()> {
        let mut map = Map::new();
        map.insert(
            "channel_id".to_string(),
            Value::Number(Number::from(channel_id.0)),
        );

        http::edit_member(self.0, user_id.0, &map)
    }

    /// Gets the number of [`Member`]s that would be pruned with the given
    /// number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    pub fn prune_count(&self, days: u16) -> Result<GuildPrune> {
        let map = json!({
            "days": days,
        });

        http::get_guild_prune_count(self.0, &map)
    }

    /// Re-orders the channels of the guild.
    ///
    /// Accepts an iterator of a tuple of the channel ID to modify and its new
    /// position.
    ///
    /// Although not required, you should specify all channels' positions,
    /// regardless of whether they were updated. Otherwise, positioning can
    /// sometimes get weird.
    #[inline]
    pub fn reorder_channels<It>(&self, channels: It) -> Result<()>
        where It: IntoIterator<Item = (ChannelId, u64)> {
        self._reorder_channels(channels.into_iter().collect())
    }

    fn _reorder_channels(&self, channels: Vec<(ChannelId, u64)>) -> Result<()> {
        let items = channels.into_iter().map(|(id, pos)| json!({
            "id": id,
            "position": pos,
        })).collect();

        http::edit_guild_channel_positions(self.0, &Value::Array(items))
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total
    /// number of shards.
    ///
    /// **Note**: When the cache is enabled, this function unlocks the cache to
    /// retrieve the total number of shards in use. If you already have the
    /// total, consider using [`utils::shard_id`].
    ///
    /// [`utils::shard_id`]: ../utils/fn.shard_id.html
    #[cfg(all(feature = "cache", feature = "utils"))]
    #[inline]
    pub fn shard_id(&self) -> u64 { ::utils::shard_id(self.0, CACHE.read().shard_count) }

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
    #[cfg(all(feature = "utils", not(feature = "cache")))]
    #[inline]
    pub fn shard_id(&self, shard_count: u64) -> u64 { ::utils::shard_id(self.0, shard_count) }

    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn start_integration_sync<I: Into<IntegrationId>>(&self, integration_id: I) -> Result<()> {
        self._start_integration_sync(integration_id.into())
    }

    fn _start_integration_sync(
        &self,
        integration_id: IntegrationId,
    ) -> Result<()> {
        http::start_integration_sync(self.0, integration_id.0)
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
        let map = json!({
            "days": days,
        });

        http::start_guild_prune(self.0, &map)
    }

    /// Unbans a [`User`] from the guild.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[inline]
    pub fn unban<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        self._unban(user_id.into())
    }

    fn _unban(self, user_id: UserId) -> Result<()> {
        http::remove_ban(self.0, user_id.0)
    }

    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn vanity_url(&self) -> Result<String> {
        http::get_guild_vanity_url(self.0)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> { http::get_guild_webhooks(self.0) }
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
