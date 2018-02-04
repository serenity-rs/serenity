use futures::{Future, future};
use model::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use super::super::utils::{deserialize_emojis, deserialize_roles};
use super::super::WrappedClient;
use ::FutureResult;

#[cfg(feature = "model")]
use builder::{EditGuild, EditMember, EditRole};

/// Partial information about a [`Guild`]. This does not include information
/// like member data.
///
/// [`Guild`]: struct.Guild.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialGuild {
    pub id: GuildId,
    pub afk_channel_id: Option<ChannelId>,
    pub afk_timeout: u64,
    pub default_message_notifications: DefaultMessageNotificationLevel,
    pub embed_channel_id: Option<ChannelId>,
    pub embed_enabled: bool,
    #[serde(deserialize_with = "deserialize_emojis")] pub emojis: HashMap<EmojiId, Emoji>,
    /// Features enabled for the guild.
    ///
    /// Refer to [`Guild::features`] for more information.
    ///
    /// [`Guild::features`]: struct.Guild.html#structfield.features
    pub features: Vec<String>,
    pub icon: Option<String>,
    pub mfa_level: MfaLevel,
    pub name: String,
    pub owner_id: UserId,
    pub region: String,
    #[serde(deserialize_with = "deserialize_roles",
            serialize_with = "serialize_gen_rc_map")]
    pub roles: HashMap<RoleId, Rc<RefCell<Role>>>,
    pub splash: Option<String>,
    pub verification_level: VerificationLevel,
    #[serde(skip)]
    pub(crate) client: WrappedClient,
}

#[cfg(feature = "model")]
impl PartialGuild {
    /// Ban a [`User`] from the guild. All messages by the
    /// user within the last given number of days given will be deleted. This
    /// may be a range between `0` and `7`.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Examples
    ///
    /// Ban a member and remove all messages they've sent in the last 4 days:
    ///
    /// ```rust,ignore
    /// // assumes a `user` and `guild` have already been bound
    /// let _ = guild.ban(user, 4);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::DeleteMessageDaysAmount`] if the number of
    /// days' worth of messages to delete is over the maximum.
    ///
    /// [`ModelError::DeleteMessageDaysAmount`]:
    /// enum.ModelError.html#variant.DeleteMessageDaysAmount
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    // todo: add ban reason
    pub fn ban<U: Into<UserId>>(&self, user_id: U, delete_message_days: u8)
        -> FutureResult<()> {
        if delete_message_days > 7 {
            return Box::new(future::err(Error::Model(
                ModelError::DeleteMessageDaysAmount(delete_message_days),
            )));
        }

        Box::new(ftryopt!(self.client).http.ban_user(
            self.id.0,
            user_id.into().0,
            delete_message_days,
            "",
        ))
    }

    /// Gets a list of the guild's bans.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[inline]
    pub fn bans(&self) -> FutureResult<Vec<Ban>> {
        ftryopt!(self.client).http.get_bans(self.id.0)
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// [`Guild`]: struct.Guild.html
    #[inline]
    pub fn channels(&self) -> FutureResult<HashMap<ChannelId, GuildChannel>> {
        let done = ftryopt!(self.client)
            .http
            .get_channels(self.id.0)
            .map(|channels| {
                let mut map = HashMap::with_capacity(channels.len());

                for channel in channels {
                    map.insert(channel.id, channel);
                }

                map
            });

        Box::new(done)
    }

    /// Creates a [`GuildChannel`] in the guild.
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
    /// use serenity::model::ChannelType;
    ///
    /// guild.create_channel("test", ChannelType::Voice, None);
    /// ```
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`http::create_channel`]: ../http/fn.create_channel.html
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn create_channel<C>(&self, name: &str, kind: ChannelType, category: C)
        -> FutureResult<GuildChannel> where C: Into<Option<ChannelId>> {
        ftryopt!(self.client).http.create_channel(
            self.id.0,
            name,
            kind,
            category.into().map(|x| x.0),
        )
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
    pub fn create_emoji(&self, name: &str, image: &str) -> FutureResult<Emoji> {
        ftryopt!(self.client).http.create_emoji(self.id.0, name, image)
    }

    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn create_integration<I>(&self, integration_id: I, kind: &str)
        -> FutureResult<()> where I: Into<IntegrationId> {
        ftryopt!(self.client).http.create_guild_integration(
            self.id.0,
            integration_id.into().0,
            kind,
        )
    }

    /// Creates a new role in the guild with the data set, if any.
    ///
    /// See the documentation for [`Guild::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`Guild::create_role`]: struct.Guild.html#method.create_role
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[inline]
    pub fn create_role<F: FnOnce(EditRole) -> EditRole>(&self, f: F)
        -> FutureResult<Role> {
        ftryopt!(self.client).http.create_role(self.id.0, f)
    }

    /// Deletes the current guild if the current user is the owner of the
    /// guild.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    #[inline]
    pub fn delete(&self) -> FutureResult<PartialGuild> {
        ftryopt!(self.client).http.delete_guild(self.id.0)
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn delete_emoji<E: Into<EmojiId>>(&self, emoji_id: E)
        -> FutureResult<()> {
        ftryopt!(self.client).http.delete_emoji(self.id.0, emoji_id.into().0)
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn delete_integration<I: Into<IntegrationId>>(&self, integration_id: I)
        -> FutureResult<()> {
        ftryopt!(self.client).http.delete_guild_integration(
            self.id.0,
            integration_id.into().0,
        )
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
    pub fn delete_role<R: Into<RoleId>>(&self, role_id: R) -> FutureResult<()> {
        let role_id = role_id.into().0;

        ftryopt!(self.client).http.delete_role(self.id.0, role_id)
    }

    /// Edits the current guild with new data where specified.
    ///
    /// **Note**: Requires the current user to have the [Manage Guild]
    /// permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn edit<'a, F: FnOnce(EditGuild) -> EditGuild>(&self, f: F)
        -> FutureResult<PartialGuild> {
        ftryopt!(self.client).http.edit_guild(self.id.0, f)
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
    pub fn edit_emoji<E: Into<EmojiId>>(&self, emoji_id: E, name: &str)
        -> FutureResult<Emoji> {
        ftryopt!(self.client)
            .http
            .edit_emoji(self.id.0, emoji_id.into().0, name)
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
    /// use serenity::model::GuildId;
    ///
    /// GuildId(7).edit_member(user_id, |m| m.mute(true).roles(&vec![role_id]));
    /// ```
    #[inline]
    pub fn edit_member<F, U>(&self, user_id: U, f: F) -> FutureResult<()>
        where F: FnOnce(EditMember) -> EditMember, U: Into<UserId> {
        ftryopt!(self.client).http.edit_member(self.id.0, user_id.into().0, f)
    }

    /// Edits the current user's nickname for the guild.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// **Note**: Requires the [Change Nickname] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to change their own
    /// nickname.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [Change Nickname]: permissions/constant.CHANGE_NICKNAME.html
    #[inline]
    pub fn edit_nickname(&self, new_nickname: Option<&str>)
        -> FutureResult<()> {
        ftryopt!(self.client).http.edit_nickname(self.id.0, new_nickname)
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    #[inline]
    pub fn kick<U: Into<UserId>>(&self, user_id: U) -> FutureResult<()> {
        ftryopt!(self.client).http.kick_member(self.id.0, user_id.into().0)
    }

    /// Returns a formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }

    /// Gets all integration of the guild.
    ///
    /// This performs a request over the REST API.
    #[inline]
    pub fn integrations(&self) -> FutureResult<Vec<Integration>> {
        ftryopt!(self.client).http.get_guild_integrations(self.id.0)
    }

    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn invites(&self) -> FutureResult<Vec<RichInvite>> {
        ftryopt!(self.client).http.get_guild_invites(self.id.0)
    }

    /// Leaves the guild.
    #[inline]
    pub fn leave(&self) -> FutureResult<()> {
        ftryopt!(self.client).http.leave_guild(self.id.0)
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`Member`]: struct.Member.html
    pub fn member<U: Into<UserId>>(&self, user_id: U) -> FutureResult<Member> {
        ftryopt!(self.client).http.get_member(self.id.0, user_id.into().0)
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Maximum
    /// value is 1000. Optionally pass in `after` to offset the results by a
    /// [`User`]'s Id.
    ///
    /// [`User`]: struct.User.html
    pub fn members<U: Into<UserId>>(&self, limit: Option<u64>, after: Option<U>)
        -> FutureResult<Vec<Member>> {
        let after = after.map(Into::into).map(|x| x.0);

        ftryopt!(self.client).http.get_guild_members(self.id.0, limit, after)
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: permissions/constant.MOVE_MEMBERS.html
    #[inline]
    pub fn move_member<C, U>(&self, user_id: U, channel_id: C)
        -> FutureResult<()> where C: Into<ChannelId>, U: Into<UserId> {
        ftryopt!(self.client)
            .http
            .edit_member(self.id.0, user_id.into().0, |f| f
                .voice_channel(channel_id.into().0))
    }

    /// Gets the number of [`Member`]s that would be pruned with the given
    /// number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    #[inline]
    pub fn prune_count(&self, days: u16) -> FutureResult<GuildPrune> {
        ftryopt!(self.client).http.get_guild_prune_count(self.id.0, days)
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
    /// ```rust,ignore
    /// use serenity::utils;
    ///
    /// // assumes a `guild` has already been bound
    ///
    /// assert_eq!(guild.shard_id(17), 7);
    /// ```
    #[cfg(feature = "utils")]
    #[inline]
    pub fn shard_id(&self, shard_count: u64) -> u64 { self.id.shard_id(shard_count) }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
    }

    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn start_integration_sync<I>(&self, integration_id: I)
        -> FutureResult<()> where I: Into<IntegrationId> {
        ftryopt!(self.client)
            .http
            .start_integration_sync(self.id.0, integration_id.into().0)
    }

    /// Unbans a [`User`] from the guild.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[inline]
    pub fn unban<U: Into<UserId>>(&self, user_id: U) -> FutureResult<()> {
        ftryopt!(self.client).http.remove_ban(self.id.0, user_id.into().0)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn webhooks(&self) -> FutureResult<Vec<Webhook>> {
        ftryopt!(self.client).http.get_guild_webhooks(self.id.0)
    }

    /// Obtain a reference to a role by its name.
    ///
    /// **Note**: If two or more roles have the same name, obtained reference will be one of
    /// them.
    ///
    /// # Examples
    ///
    /// Obtain a reference to a [`Role`] by its name.
    ///
    /// ```rust,no_run
    /// use serenity::model::prelude::*;
    /// use serenity::prelude::*;
    ///
    /// struct Handler;
    ///
    /// use serenity::CACHE;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, _: Context, msg: Message) {
    ///         let guild = msg.guild_id().unwrap().get().unwrap();
    ///         let possible_role = guild.role_by_name("role_name");
    ///
    ///         if let Some(role) = possible_role {
    ///             println!("Obtained role's reference: {:?}", role);
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    pub fn role_by_name(&self, role_name: &str) -> Option<&Rc<RefCell<Role>>> {
        self.roles.values().find(|role| role_name == role.borrow().name)
    }
}
