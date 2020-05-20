use crate::model::prelude::*;

#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "model")]
use crate::builder::{EditGuild, EditMember, EditRole};
#[cfg(feature = "model")]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::utils;
#[cfg(feature = "model")]
use crate::builder::CreateChannel;
#[cfg(any(feature = "model", feature = "http"))]
use serde_json::json;
#[cfg(feature = "cache")]
use futures::stream::Stream;
#[cfg(feature = "collector")]
use crate::client::bridge::gateway::ShardMessenger;
#[cfg(feature = "collector")]
use crate::collector::{
    CollectReply, MessageCollectorBuilder,
    CollectReaction, ReactionCollectorBuilder,
};
#[cfg(feature = "model")]
use crate::http::{Http, CacheHttp};

#[cfg(feature = "model")]
impl GuildId {
    /// Ban a [`User`] from the guild, deleting a number of
    /// days' worth of messages (`dmd`) between the range 0 and 7.
    ///
    /// Refer to the documentation for [`Guild::ban`] for more information.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Examples
    ///
    /// Ban a member and remove all messages they've sent in the last 4 days:
    ///
    /// ```rust,no_run
    /// use serenity::model::id::UserId;
    /// use serenity::model::id::GuildId;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use serenity::http::Http;
    /// # let http = Http::default();
    /// # let user = UserId(1);
    /// // assuming a `user` has already been bound
    /// let _ = GuildId(81384788765712384).ban(&http, user, 4).await;
    /// #    Ok(())
    /// # }
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
    #[inline]
    pub async fn ban(self, http: impl AsRef<Http>, user: impl Into<UserId>, dmd: u8) -> Result<()> {
        self._ban_with_reason(http, user.into(), dmd, "").await
    }

    /// Ban a [`User`] from the guild with a reason. Refer to [`ban`] to further documentation.
    ///
    /// [`User`]: ../user/struct.User.html
    /// [`ban`]: #method.ban
    #[inline]
    pub async fn ban_with_reason(self, http: impl AsRef<Http>,
                                 user: impl Into<UserId>,
                                 dmd: u8,
                                 reason: impl AsRef<str>) -> Result<()> {
        self._ban_with_reason(http, user.into(), dmd, reason.as_ref()).await
    }

    async fn _ban_with_reason(self, http: impl AsRef<Http>, user: UserId, dmd: u8, reason: &str) -> Result<()> {
        if dmd > 7 {
            return Err(Error::Model(ModelError::DeleteMessageDaysAmount(dmd)));
        }

        if reason.len() > 512 {
            return Err(Error::ExceededLimit(reason.to_string(), 512));
        }

        http
            .as_ref()
            .ban_user(self.0,
                user.0,
                dmd,
                reason
            ).await
    }

    /// Gets a list of the guild's bans.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[inline]
    pub async fn bans(self, http: impl AsRef<Http>) -> Result<Vec<Ban>> {
        http.as_ref().get_bans(self.0).await
    }

    /// Gets a list of the guild's audit log entries
    #[inline]
    pub async fn audit_logs(self, http: impl AsRef<Http>,
                            action_type: Option<u8>,
                            user_id: Option<UserId>,
                            before: Option<AuditLogEntryId>,
                            limit: Option<u8>) -> Result<AuditLogs> {
        http.as_ref()
            .get_audit_logs(
                self.0, action_type, user_id.map(|u| u.0),
                before.map(|a| a.0), limit
            ).await
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    pub async fn channels(self, http: impl AsRef<Http>) -> Result<HashMap<ChannelId, GuildChannel>> {
        let mut channels = HashMap::new();

        // Clippy is suggesting:
        // consider removing
        // `http.as_ref().get_channels(self.0)?()`:
        // `http.as_ref().get_channels(self.0)?`.
        #[allow(clippy::identity_conversion)]
        for channel in http.as_ref().get_channels(self.0).await? {
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
    /// ```rust,no_run
    /// use serenity::model::id::GuildId;
    /// use serenity::model::channel::ChannelType;
    ///
    /// # async fn run() {
    /// # use serenity::http::Http;
    /// # let http = Http::default();
    /// let _channel = GuildId(7).create_channel(&http, |c| c.name("test").kind(ChannelType::Voice)).await;
    /// # }
    /// ```
    ///
    /// [`GuildChannel`]: ../channel/struct.GuildChannel.html
    /// [`http::create_channel`]: ../../http/fn.create_channel.html
    /// [Manage Channels]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    #[inline]
    pub async fn create_channel(self, http: impl AsRef<Http>, f: impl FnOnce(&mut CreateChannel) -> &mut CreateChannel) -> Result<GuildChannel> {
        let mut builder = CreateChannel::default();
        f(&mut builder);

        let map = utils::hashmap_to_json_map(builder.0);

        http.as_ref().create_channel(self.0, &map).await
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
    #[inline]
    pub async fn create_emoji(self, http: impl AsRef<Http>, name: &str, image: &str) -> Result<Emoji> {
        let map = json!({
            "name": name,
            "image": image,
        });

        http.as_ref().create_emoji(self.0, &map).await
    }

    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[inline]
    pub async fn create_integration<I>(self, http: impl AsRef<Http>, integration_id: I, kind: &str) -> Result<()>
        where I: Into<IntegrationId> {
        self._create_integration(&http, integration_id.into(), kind).await
    }

    async fn _create_integration(
        self,
        http: impl AsRef<Http>,
        integration_id: IntegrationId,
        kind: &str,
    ) -> Result<()> {
        let map = json!({
            "id": integration_id.0,
            "type": kind,
        });

        http.as_ref().create_guild_integration(self.0, integration_id.0, &map).await
    }

    /// Creates a new role in the guild with the data set, if any.
    ///
    /// See the documentation for [`Guild::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Guild::create_role`]: ../guild/struct.Guild.html#method.create_role
    /// [Manage Roles]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    #[inline]
    pub async fn create_role<F>(self, http: impl AsRef<Http>, f: F) -> Result<Role>
    where F: FnOnce(&mut EditRole) -> &mut EditRole {
        let mut edit_role = EditRole::default();
        f(&mut edit_role);
        let map = utils::hashmap_to_json_map(edit_role.0);

        let role = http.as_ref().create_role(self.0, &map).await?;

        if let Some(position) = map.get("position").and_then(Value::as_u64) {
            self.edit_role_position(&http, role.id, position).await?;
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
    #[inline]
    pub async fn delete(self, http: impl AsRef<Http>) -> Result<PartialGuild> {
        http.as_ref().delete_guild(self.0).await
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: ../guild/struct.Emoji.html
    /// [Manage Emojis]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    #[inline]
    pub async fn delete_emoji<E: Into<EmojiId>>(self, http: impl AsRef<Http>, emoji_id: E) -> Result<()> {
        self._delete_emoji(&http, emoji_id.into()).await
    }

    #[inline]
    async fn _delete_emoji(self, http: impl AsRef<Http>, emoji_id: EmojiId) -> Result<()> {
        http.as_ref().delete_emoji(self.0, emoji_id.0).await
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[inline]
    pub async fn delete_integration<I: Into<IntegrationId>>(self, http: impl AsRef<Http>, integration_id: I) -> Result<()> {
        self._delete_integration(&http, integration_id.into()).await
    }

    async fn _delete_integration(self, http: impl AsRef<Http>, integration_id: IntegrationId) -> Result<()> {
        http.as_ref().delete_guild_integration(self.0, integration_id.0).await
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
    #[inline]
    pub async fn delete_role<R: Into<RoleId>>(self, http: impl AsRef<Http>, role_id: R) -> Result<()> {
        self._delete_role(&http, role_id.into()).await
    }

    #[inline]
    async fn _delete_role(self, http: impl AsRef<Http>, role_id: RoleId) -> Result<()> {
        http.as_ref().delete_role(self.0, role_id.0).await
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
    #[inline]
    pub async fn edit<F>(&mut self, http: impl AsRef<Http>, f: F) -> Result<PartialGuild>
    where F: FnOnce(&mut EditGuild) -> &mut EditGuild{
        let mut edit_guild = EditGuild::default();
        f(&mut edit_guild);
        let map = utils::hashmap_to_json_map(edit_guild.0);

        http.as_ref().edit_guild(self.0, &map).await
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
    #[inline]
    pub async fn edit_emoji<E: Into<EmojiId>>(self, http: impl AsRef<Http>, emoji_id: E, name: &str) -> Result<Emoji> {
        self._edit_emoji(&http, emoji_id.into(), name).await
    }

    async fn _edit_emoji(self, http: impl AsRef<Http>, emoji_id: EmojiId, name: &str) -> Result<Emoji> {
        let map = json!({
            "name": name,
        });

        http.as_ref().edit_emoji(self.0, emoji_id.0, &map).await
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
    #[inline]
    pub async fn edit_member<F, U>(self, http: impl AsRef<Http>, user_id: U, f: F) -> Result<()>
        where F: FnOnce(&mut EditMember) -> &mut EditMember, U: Into<UserId> {
        self._edit_member(&http, user_id.into(), f).await
    }

    async fn _edit_member<F>(self, http: impl AsRef<Http>, user_id: UserId, f: F) -> Result<()>
        where F: FnOnce(&mut EditMember) -> &mut EditMember {
        let mut edit_member = EditMember::default();
        f(&mut edit_member);
        let map = utils::hashmap_to_json_map(edit_member.0);

        http.as_ref().edit_member(self.0, user_id.0, &map).await
    }

    /// Edits the current user's nickname for the guild.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// Requires the [Change Nickname] permission.
    ///
    /// [Change Nickname]: ../permissions/struct.Permissions.html#associatedconstant.CHANGE_NICKNAME
    #[inline]
    pub async fn edit_nickname(self, http: impl AsRef<Http>, new_nickname: Option<&str>) -> Result<()> {
        http.as_ref().edit_nickname(self.0, new_nickname).await
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
    #[inline]
    pub async fn edit_role<F, R>(self, http: impl AsRef<Http>, role_id: R, f: F) -> Result<Role>
        where F: FnOnce(&mut EditRole) -> &mut EditRole, R: Into<RoleId> {
        self._edit_role(&http, role_id.into(), f).await
    }

    async fn _edit_role<F>(self, http: impl AsRef<Http>, role_id: RoleId, f: F) -> Result<Role>
        where F: FnOnce(&mut EditRole) -> &mut EditRole {
        let mut edit_role = EditRole::default();
        f(&mut edit_role);
        let map = utils::hashmap_to_json_map(edit_role.0);

        http.as_ref().edit_role(self.0, role_id.0, &map).await
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
    #[inline]
    pub async fn edit_role_position<R>(self, http: impl AsRef<Http>, role_id: R, position: u64) -> Result<Vec<Role>>
        where R: Into<RoleId> {
        self._edit_role_position(&http, role_id.into(), position).await
    }

    #[inline]
    async fn _edit_role_position(
        self,
        http: impl AsRef<Http>,
        role_id: RoleId,
        position: u64,
    ) -> Result<Vec<Role>> {
        http.as_ref().edit_role_position(self.0, role_id.0, position).await
    }

    /// Tries to find the [`Guild`] by its Id in the cache.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    #[cfg(feature = "cache")]
    #[inline]
    pub async fn to_guild_cached(self, cache: impl AsRef<Cache>) -> Option<Guild> {
        cache.as_ref().guild(self).await
    }

    /// Requests [`PartialGuild`] over REST API.
    ///
    /// **Note**: This will not be a [`Guild`], as the REST API does not send
    /// all data with a guild retrieval.
    ///
    /// [`PartialGuild`]: ../guild/struct.PartialGuild.html
    /// [`Guild`]: ../guild/struct.Guild.html
    #[inline]
    pub async fn to_partial_guild(self, http: impl AsRef<Http>) -> Result<PartialGuild> {
        http.as_ref().get_guild(self.0).await
    }

    /// Gets all integration of the guild.
    ///
    /// This performs a request over the REST API.
    #[inline]
    pub async fn integrations(self, http: impl AsRef<Http>) -> Result<Vec<Integration>> {
        http.as_ref().get_guild_integrations(self.0).await
    }

    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[inline]
    pub async fn invites(self, http: impl AsRef<Http>) -> Result<Vec<RichInvite>> {
        http.as_ref().get_guild_invites(self.0).await
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: ../guild/struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[inline]
    pub async fn kick<U: Into<UserId>>(self, http: impl AsRef<Http>, user_id: U) -> Result<()> {
        self.kick_with_reason(http, user_id, "").await
    }

    #[inline]
    pub async fn kick_with_reason<U: Into<UserId>>(self, http: impl AsRef<Http>, user_id: U, reason: &str) -> Result<()> {
        http.as_ref().kick_member(self.0, user_id.into().0, reason).await
    }

    /// Leaves the guild.
    #[inline]
    pub async fn leave(self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().leave_guild(self.0).await
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// If the cache feature is enabled the cache will be checked
    /// first, if found, the [`Member`] will be cloned out.
    ///
    /// If not found it will resort to an http request.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
    /// [`Member`]: ../guild/struct.Member.html
    #[inline]
    pub async fn member<U: Into<UserId>>(self, cache_http: impl CacheHttp, user_id: U) -> Result<Member> {
        self._member(cache_http, user_id.into()).await
    }

    async fn _member(self, cache_http: impl CacheHttp, user_id: UserId) -> Result<Member> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {

                if let Some(member) = cache.member(self.0, user_id).await {
                    return Ok(member);
                }
            }
        }

        cache_http.http().get_member(self.0, user_id.0).await
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Maximum
    /// value is 1000. Optionally pass in `after` to offset the results by a
    /// [`User`]'s Id.
    ///
    /// [`User`]: ../user/struct.User.html
    #[inline]
    pub async fn members<U>(self, http: impl AsRef<Http>, limit: Option<u64>, after: U) -> Result<Vec<Member>>
        where U: Into<Option<UserId>> {
        self._members(&http, limit, after.into()).await
    }

    #[inline]
    async fn _members(self, http: impl AsRef<Http>, limit: Option<u64>, after: Option<UserId>) -> Result<Vec<Member>> {
        http.as_ref().get_guild_members(self.0, limit, after.map(|x| x.0)).await
    }

    /// Streams over all the members in a guild.
    ///
    /// This is accomplished and equivilent to repeated calls to [`members`].
    /// A buffer of at most 1,000 members is used to reduce the number of calls
    /// necessary.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use serenity::model::id::GuildId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() {
    /// # let guild_id = GuildId::default();
    /// # let ctx = Http::default();
    /// use serenity::model::guild::MembersIter;
    /// use serenity::futures::StreamExt;
    ///
    /// let mut members = guild_id.members_iter(&ctx).boxed();
    /// while let Some(member_result) = members.next().await {
    ///     match member_result {
    ///         Ok(member) => println!(
    ///             "{} is {}",
    ///             member,
    ///             member.display_name(),
    ///         ),
    ///         Err(error) => eprintln!("Uh oh!  Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    #[cfg(feature = "cache")]
    pub fn members_iter<H: AsRef<Http>>(self, http: H) -> impl Stream<Item=Result<Member>> {
        MembersIter::<H>::stream(http, self)
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: ../permissions/struct.Permissions.html#associatedconstant.MOVE_MEMBERS
    #[inline]
    pub async fn move_member<C, U>(self, http: impl AsRef<Http>, user_id: U, channel_id: C) -> Result<()>
        where C: Into<ChannelId>, U: Into<UserId> {
        self._move_member(&http, user_id.into(), channel_id.into()).await
    }

    async fn _move_member(
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

        http.as_ref().edit_member(self.0, user_id.0, &map).await
    }

    /// Returns the name of whatever guild this id holds.
    #[cfg(feature = "cache")]
    pub async fn name(self, cache: impl AsRef<Cache>) -> Option<String> {
        cache.as_ref().guild_field(self, |guild| guild.name.clone()).await
    }

    /// Gets the number of [`Member`]s that would be pruned with the given
    /// number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: ../guild/struct.Member.html
    /// [Kick Members]: ../permissions/struct.Permissions.html#associatedconstant.KICK_MEMBERS
    #[inline]
    pub async fn prune_count(self, http: impl AsRef<Http>, days: u16) -> Result<GuildPrune> {
        let map = json!({
            "days": days,
        });

        http.as_ref().get_guild_prune_count(self.0, &map).await
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
    pub async fn reorder_channels<It>(self, http: impl AsRef<Http>, channels: It) -> Result<()>
        where It: IntoIterator<Item = (ChannelId, u64)> {
        self._reorder_channels(&http, channels.into_iter().collect()).await
    }

    async fn _reorder_channels(self, http: impl AsRef<Http>, channels: Vec<(ChannelId, u64)>) -> Result<()> {
        let items = channels.into_iter().map(|(id, pos)| json!({
            "id": id,
            "position": pos,
        })).collect();

        http.as_ref().edit_guild_channel_positions(self.0, &Value::Array(items)).await
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
    pub async fn shard_id(self, cache: impl AsRef<Cache>) -> u64 {
        crate::utils::shard_id(self.0, cache.as_ref().shard_count().await)
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
    /// # async fn test() {
    /// let guild_id = GuildId(81384788765712384);
    ///
    /// assert_eq!(guild_id.shard_id(17).await, 7);
    /// # }
    /// ```
    #[cfg(all(feature = "utils", not(feature = "cache")))]
    #[inline]
    pub async fn shard_id(self, shard_count: u64) -> u64 {
        crate::utils::shard_id(self.0, shard_count)
    }

    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[inline]
    pub async fn start_integration_sync<I: Into<IntegrationId>>(self, http: impl AsRef<Http>, integration_id: I) -> Result<()> {
        self._start_integration_sync(&http, integration_id.into()).await
    }

    #[inline]
    async fn _start_integration_sync(
        self,
        http: impl AsRef<Http>,
        integration_id: IntegrationId,
    ) -> Result<()> {
        http.as_ref().start_integration_sync(self.0, integration_id.0).await
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
    #[inline]
    pub async fn start_prune(self, http: impl AsRef<Http>, days: u16) -> Result<GuildPrune> {
        let map = json!({
            "days": days,
        });

        http.as_ref().start_guild_prune(self.0, &map).await
    }

    /// Unbans a [`User`] from the guild.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [`User`]: ../user/struct.User.html
    /// [Ban Members]: ../permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    #[inline]
    pub async fn unban<U: Into<UserId>>(self, http: impl AsRef<Http>, user_id: U) -> Result<()> {
        self._unban(&http, user_id.into()).await
    }

    #[inline]
    async fn _unban(self, http: impl AsRef<Http>, user_id: UserId) -> Result<()> {
        http.as_ref().remove_ban(self.0, user_id.0).await
    }

    /// Retrieve's the guild's vanity URL.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    #[inline]
    pub async fn vanity_url(self, http: impl AsRef<Http>) -> Result<String> {
        http.as_ref().get_guild_vanity_url(self.0).await
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: ../permissions/struct.Permissions.html#associatedconstant.MANAGE_WEBHOOKS
    #[inline]
    pub async fn webhooks(self, http: impl AsRef<Http>) -> Result<Vec<Webhook>> {
        http.as_ref().get_guild_webhooks(self.0).await
    }

    /// Returns a future that will await one message sent in this guild.
    #[cfg(feature = "collector")]
    pub fn await_reply<'a>(&self, shard_messenger: &'a impl AsRef<ShardMessenger>) -> CollectReply<'a> {
        CollectReply::new(shard_messenger).guild_id(self.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of messages in this guild.
    #[cfg(feature = "collector")]
    pub fn await_replies<'a>(&self, shard_messenger: &'a impl AsRef<ShardMessenger>) -> MessageCollectorBuilder<'a> {
        MessageCollectorBuilder::new(shard_messenger).guild_id(self.0)
    }

    /// Await a single reaction in this guild.
    #[cfg(feature = "collector")]
    pub fn await_reaction<'a>(&self, shard_messenger: &'a impl AsRef<ShardMessenger>) -> CollectReaction<'a> {
        CollectReaction::new(shard_messenger).guild_id(self.0)
    }

    /// Returns a stream builder which can be awaited to obtain a stream of reactions sent in this guild.
    #[cfg(feature = "collector")]
    pub fn await_reactions<'a>(&self, shard_messenger: &'a impl AsRef<ShardMessenger>) -> ReactionCollectorBuilder<'a> {
        ReactionCollectorBuilder::new(shard_messenger).guild_id(self.0)
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

/// A helper class returned by [`GuildId.members_iter()`]
///
/// [`GuildId.members_iter()`]: #method.members_iter
#[derive(Clone, Debug)]
#[cfg(all(feature = "model", feature = "cache"))]
pub struct MembersIter<H: AsRef<Http>> {
    guild_id: GuildId,
    http: H,
    buffer: Vec<Member>,
    after: Option<UserId>,
    tried_fetch: bool,
}

#[cfg(all(feature = "model", feature = "cache"))]
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
    async fn refresh(&mut self) -> Result<()> {
        // Number of profiles to fetch
        let grab_size: u64 = 1000;

        self.buffer = self.guild_id
            ._members(self.http.as_ref(), Some(grab_size), self.after).await?;

        // Get the last member. If shorter than 1000, there are no more results anyway
        self.after = match self.buffer.get(grab_size as usize - 1) {
            Some(member) => Some(member.user.id),
            None => None,
        };

        // Reverse to optimize pop()
        self.buffer.reverse();

        self.tried_fetch = true;

        Ok(())
    }

    /// Streams over all the members in a guild.
    ///
    /// This is accomplished and equivilent to repeated calls to [`members`].
    /// A buffer of at most 1,000 members is used to reduce the number of calls
    /// necessary.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::GuildId;
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() {
    /// # let guild_id = GuildId::default();
    /// # let ctx = Http::default();
    /// use serenity::model::guild::MembersIter;
    /// use serenity::futures::StreamExt;
    ///
    /// let mut members = MembersIter::<Http>::stream(&ctx, guild_id).boxed();
    /// while let Some(member_result) = members.next().await {
    ///     match member_result {
    ///         Ok(member) => println!(
    ///             "{} is {}",
    ///             member,
    ///             member.display_name(),
    ///         ),
    ///         Err(error) => eprintln!("Uh oh!  Error: {}", error),
    ///     }
    /// }
    /// # }
    /// ```
    pub fn stream(http: impl AsRef<Http>, guild_id: GuildId) -> impl Stream<Item=Result<Member>> {
        let init_state = MembersIter::new(guild_id, http);

        futures::stream::unfold(init_state, |mut state| async {
            if state.buffer.is_empty() && state.after.is_some() || !state.tried_fetch {

                if let Err(error) = state.refresh().await {
                    return Some((Err(error), state));
                }
            }

            state.buffer.pop().map(|entry| (Ok(entry), state))
        })
    }
}
