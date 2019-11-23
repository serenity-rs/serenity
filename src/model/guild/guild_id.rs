#[cfg(feature = "http")]
use crate::http::CacheHttp;
use crate::{model::prelude::*};

#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::CacheRwLock;
#[cfg(feature = "model")]
use crate::builder::{EditGuild, EditMember, EditRole};
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::model::guild::BanOptions;
#[cfg(feature = "model")]
use crate::utils;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "model")]
use crate::builder::CreateChannel;
#[cfg(feature = "model")]
use serde_json::json;

#[cfg(feature = "model")]
impl GuildId {
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
    /// use serenity::model::id::GuildId;
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
    /// [`ModelError::DeleteMessageDaysAmount`]: ../error/enum.Error.html#variant.DeleteMessageDaysAmount
    /// [`Guild::ban`]: ../guild/struct.Guild.html#method.ban
    /// [`User`]: ../user/struct.User.html
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub fn ban<U, BO>(self, http: impl AsRef<Http>, user: U, ban_options: &BO) -> Result<()>
        where U: Into<UserId>, BO: BanOptions {
        self._ban(&http, user.into(), (ban_options.dmd(), ban_options.reason()))
    }

    #[cfg(feature = "http")]
    fn _ban(self, http: impl AsRef<Http>, user: UserId, ban_options: (u8, &str)) -> Result<()> {
        let (dmd, reason) = ban_options;

        if dmd > 7 {
            return Err(Error::Model(ModelError::DeleteMessageDaysAmount(dmd)));
        }

        if reason.len() > 512 {
            return Err(Error::ExceededLimit(reason.to_string(), 512));
        }

        http.as_ref().ban_user(self.0, user.0, dmd, reason)
    }

    /// Gets a list of the guild's bans.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub fn bans(self, http: impl AsRef<Http>) -> Result<Vec<Ban>> {http.as_ref().get_bans(self.0) }

    /// Gets a list of the guild's audit log entries
    #[cfg(feature = "http")]
    #[inline]
    pub fn audit_logs(self, http: impl AsRef<Http>,
                             action_type: Option<u8>,
                             user_id: Option<UserId>,
                             before: Option<AuditLogEntryId>,
                             limit: Option<u8>) -> Result<AuditLogs> {
        http.as_ref().get_audit_logs(self.0, action_type, user_id.map(|u| u.0), before.map(|a| a.0), limit)
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    #[cfg(feature = "http")]
    pub fn channels(self, http: impl AsRef<Http>) -> Result<HashMap<ChannelId, GuildChannel>> {
        let mut channels = HashMap::new();

        // Clippy is suggesting:
        // consider removing
        // `http.as_ref().get_channels(self.0)?()`:
        // `http.as_ref().get_channels(self.0)?`.
        #[allow(clippy::identity_conversion)]
        for channel in http.as_ref().get_channels(self.0)? {
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
    /// use serenity::model::id::GuildId;
    /// use serenity::model::channel::ChannelType;
    ///
    /// let _channel = GuildId(7).create_channel(|c| c.name("test").kind(ChannelType::Voice));
    /// ```
    ///
    /// [`GuildChannel`]: ../channel/struct.GuildChannel.html
    /// [`http::create_channel`]: ../../http/fn.create_channel.html
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[cfg(feature = "http")]
    #[inline]
    pub fn create_channel(self, http: impl AsRef<Http>, f: impl FnOnce(&mut CreateChannel) -> &mut CreateChannel) -> Result<GuildChannel> {
        let mut builder = CreateChannel::default();
        f(&mut builder);

        let map = utils::hashmap_to_json_map(builder.0);

        http.as_ref().create_channel(self.0, &map)
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
    /// [`EditProfile::avatar`]: ../../builder/struct.EditProfile.html#method.avatar
    /// [`Guild::create_emoji`]: ../guild/struct.Guild.html#method.create_emoji
    /// [`utils::read_image`]: ../../utils/fn.read_image.html
    /// [Manage Emojis]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[cfg(feature = "http")]
    #[inline]
    pub fn create_emoji(self, http: impl AsRef<Http>, name: &str, image: &str) -> Result<Emoji> {
        let map = json!({
            "name": name,
            "image": image,
        });

        http.as_ref().create_emoji(self.0, &map)
    }

    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub fn create_integration<I>(self, http: impl AsRef<Http>, integration_id: I, kind: &str) -> Result<()>
        where I: Into<IntegrationId> {
        self._create_integration(&http, integration_id.into(), kind)
    }

    #[cfg(feature = "http")]
    fn _create_integration(
        self,
        http: impl AsRef<Http>,
        integration_id: IntegrationId,
        kind: &str,
    ) -> Result<()> {
        let map = json!({
            "id": integration_id.0,
            "type": kind,
        });

        http.as_ref().create_guild_integration(self.0, integration_id.0, &map)
    }

    /// Creates a new role in the guild with the data set, if any.
    ///
    /// See the documentation for [`Guild::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Guild::create_role`]: ../guild/struct.Guild.html#method.create_role
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(feature = "http")]
    #[inline]
    pub fn create_role<F>(self, http: impl AsRef<Http>, f: F) -> Result<Role>
    where F: FnOnce(&mut EditRole) -> &mut EditRole {
        let mut edit_role = EditRole::default();
        f(&mut edit_role);
        let map = utils::hashmap_to_json_map(edit_role.0);

        let role = http.as_ref().create_role(self.0, &map)?;

        if let Some(position) = map.get("position").and_then(Value::as_u64) {
            self.edit_role_position(&http, role.id, position)?;
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
    /// [`Guild::delete`]: ../guild/struct.Guild.html#method.delete
    #[cfg(feature = "http")]
    #[inline]
    pub fn delete(self, http: impl AsRef<Http>) -> Result<PartialGuild> { http.as_ref().delete_guild(self.0) }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [Manage Emojis]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[cfg(feature = "http")]
    #[inline]
    pub fn delete_emoji<E: Into<EmojiId>>(self, http: impl AsRef<Http>, emoji_id: E) -> Result<()> {
        self._delete_emoji(&http, emoji_id.into())
    }

    #[cfg(feature = "http")]
    fn _delete_emoji(self, http: impl AsRef<Http>, emoji_id: EmojiId) -> Result<()> {
        http.as_ref().delete_emoji(self.0, emoji_id.0)
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub fn delete_integration<I: Into<IntegrationId>>(self, http: impl AsRef<Http>, integration_id: I) -> Result<()> {
        self._delete_integration(&http, integration_id.into())
    }

    fn _delete_integration(self, http: impl AsRef<Http>, integration_id: IntegrationId) -> Result<()> {
        http.as_ref().delete_guild_integration(self.0, integration_id.0)
    }

    /// Deletes a [`Role`] by Id from the guild.
    ///
    /// Also see [`Role::delete`] if you have the `cache` and `methods` features
    /// enabled.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: ../guild/struct.Role.html
    /// [`Role::delete`]: ../guild/struct.Role.html#method.delete
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(feature = "http")]
    #[inline]
    pub fn delete_role<R: Into<RoleId>>(self, http: impl AsRef<Http>, role_id: R) -> Result<()> {
        self._delete_role(&http, role_id.into())
    }

    #[cfg(feature = "http")]
    fn _delete_role(self, http: impl AsRef<Http>, role_id: RoleId) -> Result<()> {
        http.as_ref().delete_role(self.0, role_id.0)
    }

    /// Edits the current guild with new data where specified.
    ///
    /// Refer to [`Guild::edit`] for more information.
    ///
    /// **Note**: Requires the current user to have the [Manage Guild]
    /// permission.
    ///
    /// [`Guild::edit`]: ../guild/struct.Guild.html#method.edit
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit<F>(&mut self, http: impl AsRef<Http>, f: F) -> Result<PartialGuild>
    where F: FnOnce(&mut EditGuild) -> &mut EditGuild{
        let mut edit_guild = EditGuild::default();
        f(&mut edit_guild);
        let map = utils::hashmap_to_json_map(edit_guild.0);

        http.as_ref().edit_guild(self.0, &map)
    }

    /// Edits an [`Emoji`]'s name in the guild.
    ///
    /// Also see [`Emoji::edit`] if you have the `cache` and `methods` features
    /// enabled.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [`Emoji::edit`]: ../guild/struct.Emoji.html#method.edit
    /// [Manage Emojis]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit_emoji<E: Into<EmojiId>>(self, http: impl AsRef<Http>, emoji_id: E, name: &str) -> Result<Emoji> {
        self._edit_emoji(&http, emoji_id.into(), name)
    }

    fn _edit_emoji(self, http: impl AsRef<Http>, emoji_id: EmojiId, name: &str) -> Result<Emoji> {
        let map = json!({
            "name": name,
        });

        http.as_ref().edit_emoji(self.0, emoji_id.0, &map)
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
    /// guild.edit_member(&context, user_id, |m| m.mute(true).roles(&vec![role_id]));
    /// ```
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit_member<F, U>(self, http: impl AsRef<Http>, user_id: U, f: F) -> Result<()>
        where F: FnOnce(&mut EditMember) -> &mut EditMember, U: Into<UserId> {
        self._edit_member(&http, user_id.into(), f)
    }

    #[cfg(feature = "http")]
    fn _edit_member<F>(self, http: impl AsRef<Http>, user_id: UserId, f: F) -> Result<()>
        where F: FnOnce(&mut EditMember) -> &mut EditMember {
        let mut edit_member = EditMember::default();
        f(&mut edit_member);
        let map = utils::hashmap_to_json_map(edit_member.0);

        http.as_ref().edit_member(self.0, user_id.0, &map)
    }

    /// Edits the current user's nickname for the guild.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// Requires the [Change Nickname] permission.
    ///
    /// [Change Nickname]: ../permissions/struct.Permissions.html#associatedconstant.CHANGE_NICKNAME
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit_nickname(self, http: impl AsRef<Http>, new_nickname: Option<&str>) -> Result<()> {
        http.as_ref().edit_nickname(self.0, new_nickname)
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
    /// GuildId(7).edit_role(&context, RoleId(8), |r| r.hoist(true));
    /// ```
    ///
    /// [`Role`]: ../guild/struct.Role.html
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit_role<F, R>(self, http: impl AsRef<Http>, role_id: R, f: F) -> Result<Role>
        where F: FnOnce(&mut EditRole) -> &mut EditRole, R: Into<RoleId> {
        self._edit_role(&http, role_id.into(), f)
    }

    #[cfg(feature = "http")]
    fn _edit_role<F>(self, http: impl AsRef<Http>, role_id: RoleId, f: F) -> Result<Role>
        where F: FnOnce(&mut EditRole) -> &mut EditRole {
        let mut edit_role = EditRole::default();
        f(&mut edit_role);
        let map = utils::hashmap_to_json_map(edit_role.0);

        http.as_ref().edit_role(self.0, role_id.0, &map)
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
    /// GuildId(7).edit_role_position(&context, RoleId(8), 2);
    /// ```
    ///
    /// [`Role`]: ../guild/struct.Role.html
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[cfg(feature = "http")]
    #[inline]
    pub fn edit_role_position<R>(self, http: impl AsRef<Http>, role_id: R, position: u64) -> Result<Vec<Role>>
        where R: Into<RoleId> {
        self._edit_role_position(&http, role_id.into(), position)
    }

    #[cfg(feature = "http")]
    fn _edit_role_position(
        self,
        http: impl AsRef<Http>,
        role_id: RoleId,
        position: u64,
    ) -> Result<Vec<Role>> {
        http.as_ref().edit_role_position(self.0, role_id.0, position)
    }

    /// Tries to find the [`Guild`] by its Id in the cache.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    #[cfg(feature = "cache")]
    #[inline]
    pub fn to_guild_cached(self, cache: impl AsRef<CacheRwLock>) -> Option<Arc<RwLock<Guild>>> {cache.as_ref().read().guild(self) }

    /// Requests [`PartialGuild`] over REST API.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send
    /// all data with a guild retrieval.
    ///
    /// [`PartialGuild`]: ../guild/struct.PartialGuild.html
    /// [`Guild`]: ../guild/struct.Guild.html
    #[cfg(feature = "http")]
    #[inline]
    pub fn to_partial_guild(self, http: impl AsRef<Http>) -> Result<PartialGuild> {http.as_ref().get_guild(self.0) }

    /// Gets all integration of the guild.
    ///
    /// This performs a request over the REST API.
    #[cfg(feature = "http")]
    #[inline]
    pub fn integrations(self, http: impl AsRef<Http>) -> Result<Vec<Integration>> {http.as_ref().get_guild_integrations(self.0) }

    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub fn invites(self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {http.as_ref().get_guild_invites(self.0) }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: ../guild/struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub fn kick<U: Into<UserId>>(self, http: impl AsRef<Http>, user_id: U) -> Result<()> {
        http.as_ref().kick_member(self.0, user_id.into().0)
    }

    /// Leaves the guild.
    #[cfg(feature = "http")]
    #[inline]
    pub fn leave(self, http: impl AsRef<Http>) -> Result<()> { http.as_ref().leave_guild(self.0) }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// If the cache feature is enabled the cache will be checked
    /// first. If not found it will resort to an http request.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [`Member`]: ../guild/struct.Member.html
    #[cfg(feature = "http")]
    #[inline]
    pub fn member<U: Into<UserId>>(self, cache_http: impl CacheHttp, user_id: U) -> Result<Member> {
        self._member(cache_http, user_id.into())
    }

    #[cfg(feature = "http")]
    fn _member(self, cache_http: impl CacheHttp, user_id: UserId) -> Result<Member> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {

                if let Some(member) = cache.read().member(self.0, user_id) {
                    return Ok(member);
                }
            }
        }

        cache_http.http().get_member(self.0, user_id.0)
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Maximum
    /// value is 1000. Optionally pass in `after` to offset the results by a
    /// [`User`]'s Id.
    ///
    /// [`User`]: ../user/struct.User.html
    #[cfg(feature = "http")]
    #[inline]
    pub fn members<U>(self, http: impl AsRef<Http>, limit: Option<u64>, after: U) -> Result<Vec<Member>>
        where U: Into<Option<UserId>> {
        self._members(&http, limit, after.into())
    }

    #[cfg(feature = "http")]
    fn _members(self, http: impl AsRef<Http>, limit: Option<u64>, after: Option<UserId>) -> Result<Vec<Member>> {
        http.as_ref().get_guild_members(self.0, limit, after.map(|x| x.0))
    }

    /// Iterates over all the members in a guild.
    ///
    /// This is accomplished and equivilent to repeated calls to [`members`].
    /// A buffer of at most 1,000 members is used to reduce the number of calls
    /// necessary.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use serenity::model::id::GuildId;
    /// # use serenity::http::Http;
    /// # let guild_id = GuildId::default();
    /// # let ctx = Http::default();
    /// for member_result in guild_id.members_iter(&ctx) {
    ///     match member_result {
    ///         Ok(member) => println!(
    ///             "{} is {}",
    ///             member,
    ///             member.display_name()
    ///         ),
    ///         Err(error) => eprintln!("Uh oh!  Error: {}", error),
    ///     }
    /// }
    #[cfg(all(feature = "http", feature = "cache"))]
    pub fn members_iter<H: AsRef<Http>>(self, http: H) -> MembersIter<H> {
        MembersIter::new(self, http)
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: ../permissions/struct.Permissions.html#associatedconstant.MOVE_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub fn move_member<C, U>(self, http: impl AsRef<Http>, user_id: U, channel_id: C) -> Result<()>
        where C: Into<ChannelId>, U: Into<UserId> {
        self._move_member(&http, user_id.into(), channel_id.into())
    }

    #[cfg(feature = "http")]
    fn _move_member(
        self,
        http: impl AsRef<Http>,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> Result<()> {
        let mut map = Map::new();
        map.insert(
            "channel_id".to_string(),
            Value::Number(Number::from(channel_id.0)),
        );

        http.as_ref().edit_member(self.0, user_id.0, &map)
    }

    /// Gets the number of [`Member`]s that would be pruned with the given
    /// number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: ../guild/struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[cfg(feature = "http")]
    pub fn prune_count(self, http: impl AsRef<Http>, days: u16) -> Result<GuildPrune> {
        let map = json!({
            "days": days,
        });

        http.as_ref().get_guild_prune_count(self.0, &map)
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
    pub fn reorder_channels<It>(self, http: impl AsRef<Http>, channels: It) -> Result<()>
        where It: IntoIterator<Item = (ChannelId, u64)> {
        self._reorder_channels(&http, channels.into_iter().collect())
    }

    fn _reorder_channels(self, http: impl AsRef<Http>, channels: Vec<(ChannelId, u64)>) -> Result<()> {
        let items = channels.into_iter().map(|(id, pos)| json!({
            "id": id,
            "position": pos,
        })).collect();

        http.as_ref().edit_guild_channel_positions(self.0, &Value::Array(items))
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
    /// [`utils::shard_id`]: ../../utils/fn.shard_id.html
    #[cfg(all(feature = "cache", feature = "utils"))]
    #[inline]
    pub fn shard_id(self, cache: impl AsRef<CacheRwLock>) -> u64 {
        crate::utils::shard_id(self.0, cache.as_ref().read().shard_count)
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
    /// use serenity::model::id::GuildId;
    /// use serenity::utils;
    ///
    /// let guild_id = GuildId(81384788765712384);
    ///
    /// assert_eq!(guild_id.shard_id(17), 7);
    /// ```
    #[cfg(all(feature = "utils", not(feature = "cache")))]
    #[inline]
    pub fn shard_id(self, shard_count: u64) -> u64 { crate::utils::shard_id(self.0, shard_count) }

    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub fn start_integration_sync<I: Into<IntegrationId>>(self, http: impl AsRef<Http>, integration_id: I) -> Result<()> {
        self._start_integration_sync(&http, integration_id.into())
    }

    #[cfg(feature = "http")]
    fn _start_integration_sync(
        self,
        http: impl AsRef<Http>,
        integration_id: IntegrationId,
    ) -> Result<()> {
        http.as_ref().start_integration_sync(self.0, integration_id.0)
    }

    /// Starts a prune of [`Member`]s.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// [`GuildPrune`]: ../guild/struct.GuildPrune.html
    /// [`Member`]: ../guild/struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub fn start_prune(self, http: impl AsRef<Http>, days: u16) -> Result<GuildPrune> {
        let map = json!({
            "days": days,
        });

        http.as_ref().start_guild_prune(self.0, &map)
    }

    /// Unbans a [`User`] from the guild.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [`User`]: ../user/struct.User.html
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[cfg(feature = "http")]
    #[inline]
    pub fn unban<U: Into<UserId>>(self, http: impl AsRef<Http>, user_id: U) -> Result<()> {
        self._unban(&http, user_id.into())
    }

    #[cfg(feature = "http")]
    fn _unban(self, http: impl AsRef<Http>, user_id: UserId) -> Result<()> {
        http.as_ref().remove_ban(self.0, user_id.0)
    }

    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[cfg(feature = "http")]
    #[inline]
    pub fn vanity_url(self, http: impl AsRef<Http>) -> Result<String> {
        http.as_ref().get_guild_vanity_url(self.0)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    #[inline]
    pub fn webhooks(self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> {http.as_ref().get_guild_webhooks(self.0) }
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

/// A helper class returned by [`GuildId.members_iter()`]
///
/// [`GuildId.members_iter()`]: #method.members_iter
#[derive(Clone, Debug)]
#[cfg(all(feature = "http", feature = "cache"))]
pub struct MembersIter<H: AsRef<Http>> {
    guild_id: GuildId,
    http: H,
    buffer: Vec<Member>,
    after: Option<UserId>,
    tried_fetch: bool,
}

#[cfg(all(feature = "http", feature = "cache"))]
impl<H: AsRef<Http>> MembersIter<H> {
    fn new(guild_id: GuildId, http: H) -> MembersIter<H> {
        MembersIter {
            guild_id,
            http,
            buffer: Vec::new(),
            after: None,
            tried_fetch: false,
        }
    }

    /// Fills the `self.buffer` cache of Members.
    ///
    /// This drops any members that
    /// were currently in the buffer, so it should only be called when
    /// `self.buffer` is empty.  Additionally, this updates `self.after` so that
    /// the next call does not return duplicate items.  If there are no more
    /// members to be fetched, then this marks `self.after` as None, indicating
    /// that no more calls ought to be made.
    fn refresh(&mut self) -> Result<()> {
        // Number of profiles to fetch
        let grab_size: u64 = 1000;

        self.buffer = self.guild_id
            ._members(self.http.as_ref(), Some(grab_size), self.after)?;

         //Get the last member.  If shorter than 1000, there are no more results anyway
        self.after = self.buffer.get(grab_size as usize - 1)
            .map(|member| member.user_id());

        // Reverse to optimize pop()
        self.buffer.reverse();

        self.tried_fetch = true;

        Ok(())
    }
}

#[cfg(all(feature = "http", feature = "cache"))]
impl<H: AsRef<Http>> Iterator for MembersIter<H> {
    type Item = Result<Member>;

    fn next(&mut self) -> Option<Result<Member>> {
        if self.buffer.is_empty() && self.after.is_some() || !self.tried_fetch {
            if let Err(e) = self.refresh() {
                return Some(Err(e))
            }
        }

        self.buffer.pop().map(Ok)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let buffer_size = self.buffer.len();
        if self.after.is_none() && self.tried_fetch {
            (buffer_size, Some(buffer_size))
        } else {
            (buffer_size, None)
        }
    }
}

#[cfg(all(feature = "http", feature = "cache"))]
impl<H: AsRef<Http>> std::iter::FusedIterator for MembersIter<H> {}
