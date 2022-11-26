#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(feature = "cache")]
use std::cmp::Reverse;
use std::fmt;

#[cfg(feature = "model")]
use crate::builder::EditMember;
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
#[cfg(all(feature = "cache", feature = "model"))]
use crate::internal::prelude::*;
#[cfg(feature = "model")]
use crate::json;
use crate::model::permissions::Permissions;
use crate::model::prelude::*;
use crate::model::Timestamp;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use crate::utils::Colour;

/// Information about a member of a guild.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Member {
    /// Indicator of whether the member can hear in voice channels.
    pub deaf: bool,
    /// The unique Id of the guild that the member is a part of.
    pub guild_id: GuildId,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<Timestamp>,
    /// Indicator of whether the member can speak in voice channels.
    pub mute: bool,
    /// The member's nickname, if present.
    ///
    /// Can't be longer than 32 characters.
    pub nick: Option<String>,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: Vec<RoleId>,
    /// Attached User struct.
    pub user: User,
    /// Indicator that the member hasn't accepted the rules of the guild yet.
    #[serde(default)]
    pub pending: bool,
    /// Timestamp representing the date since the member is boosting the guild.
    pub premium_since: Option<Timestamp>,
    /// The total permissions of the member in a channel, including overrides.
    ///
    /// This is only [`Some`] when returned in an [`Interaction`] object.
    ///
    /// [`Interaction`]: crate::model::application::interaction::Interaction
    pub permissions: Option<Permissions>,
    /// The guild avatar hash
    pub avatar: Option<String>,
    /// When the user's timeout will expire and the user will be able to communicate in the guild again.
    ///
    /// Will be None or a time in the past if the user is not timed out.
    pub communication_disabled_until: Option<Timestamp>,
}

/// Helper for deserialization without a `GuildId` but then later updated to the correct `GuildId`.
///
/// The only difference to `Member` is `#[serde(default)]` on `guild_id`.
#[derive(Deserialize)]
pub(crate) struct InterimMember {
    pub deaf: bool,
    #[serde(default)]
    pub guild_id: GuildId,
    pub joined_at: Option<Timestamp>,
    pub mute: bool,
    pub nick: Option<String>,
    pub roles: Vec<RoleId>,
    pub user: User,
    #[serde(default)]
    pub pending: bool,
    pub premium_since: Option<Timestamp>,
    pub permissions: Option<Permissions>,
    pub avatar: Option<String>,
    pub communication_disabled_until: Option<Timestamp>,
}

impl From<InterimMember> for Member {
    fn from(m: InterimMember) -> Self {
        Self {
            deaf: m.deaf,
            guild_id: m.guild_id,
            joined_at: m.joined_at,
            mute: m.mute,
            nick: m.nick,
            roles: m.roles,
            user: m.user,
            pending: m.pending,
            premium_since: m.premium_since,
            permissions: m.permissions,
            avatar: m.avatar,
            communication_disabled_until: m.communication_disabled_until,
        }
    }
}

#[cfg(feature = "model")]
impl Member {
    /// Adds a [`Role`] to the member, editing its roles in-place if the request
    /// was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if a role with the given Id does not exist.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn add_role(
        &mut self,
        http: impl AsRef<Http>,
        role_id: impl Into<RoleId>,
    ) -> Result<()> {
        self._add_role(&http, role_id.into()).await
    }

    async fn _add_role(&mut self, http: impl AsRef<Http>, role_id: RoleId) -> Result<()> {
        if self.roles.contains(&role_id) {
            return Ok(());
        }

        match http.as_ref().add_member_role(self.guild_id.0, self.user.id.0, role_id.0, None).await
        {
            Ok(()) => {
                self.roles.push(role_id);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Adds one or multiple [`Role`]s to the member, editing
    /// its roles in-place if the request was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if a role with a given Id does not exist.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn add_roles(
        &mut self,
        http: impl AsRef<Http>,
        role_ids: &[RoleId],
    ) -> Result<Vec<RoleId>> {
        self.roles.extend_from_slice(role_ids);

        let mut builder = EditMember::default();
        builder.roles(&self.roles);
        let map = json::hashmap_to_json_map(builder.0);

        match http.as_ref().edit_member(self.guild_id.0, self.user.id.0, &map, None).await {
            Ok(member) => Ok(member.roles),
            Err(why) => {
                self.roles.retain(|r| !role_ids.contains(r));

                Err(why)
            },
        }
    }

    /// Ban a [`User`] from the guild, deleting a number of
    /// days' worth of messages (`dmd`) between the range 0 and 7.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::DeleteMessageDaysAmount`] if the `dmd` is greater than 7.
    /// Can also return [`Error::Http`] if the current user lacks permission to ban
    /// this member.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    #[inline]
    pub async fn ban(&self, http: impl AsRef<Http>, dmd: u8) -> Result<()> {
        self.ban_with_reason(&http, dmd, "").await
    }

    /// Ban the member from the guild with a reason. Refer to [`Self::ban`] to further documentation.
    ///
    /// # Errors
    ///
    /// In addition to the errors [`Self::ban`] may return, can also return [`Error::ExceededLimit`]
    /// if the length of the reason is greater than 512.
    #[inline]
    pub async fn ban_with_reason(
        &self,
        http: impl AsRef<Http>,
        dmd: u8,
        reason: impl AsRef<str>,
    ) -> Result<()> {
        self.guild_id.ban_with_reason(http, self.user.id, dmd, reason).await
    }

    /// Determines the member's colour.
    #[cfg(feature = "cache")]
    pub fn colour(&self, cache: impl AsRef<Cache>) -> Option<Colour> {
        let guild_roles = cache.as_ref().guild_field(self.guild_id, |g| g.roles.clone())?;

        let mut roles = self
            .roles
            .iter()
            .filter_map(|role_id| guild_roles.get(role_id))
            .collect::<Vec<&Role>>();

        roles.sort_by_key(|&b| Reverse(b));

        let default = Colour::default();

        roles.iter().find(|r| r.colour.0 != default.0).map(|r| r.colour)
    }

    /// Returns the "default channel" of the guild for the member.
    /// (This returns the first channel that can be read by the member, if there isn't
    /// one returns [`None`])
    #[cfg(feature = "cache")]
    pub fn default_channel(&self, cache: impl AsRef<Cache>) -> Option<GuildChannel> {
        let guild = self.guild_id.to_guild_cached(cache)?;

        let member = guild.members.get(&self.user.id)?;

        for channel in guild.channels.values() {
            if let Channel::Guild(channel) = channel {
                if guild.user_permissions_in(channel, member).ok()?.view_channel() {
                    return Some(channel.clone());
                }
            }
        }

        None
    }

    /// Times the user out until `time`.
    ///
    /// Requires the [Moderate Members] permission.
    ///
    /// **Note**: [Moderate Members]: crate::model::permission::Permissions::MODERATE_MEMBERS
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission or if `time` is greater than
    /// 28 days from the current time.
    ///
    /// [Moderate Members]: Permissions::MODERATE_MEMBERS
    #[doc(alias = "timeout")]
    pub async fn disable_communication_until_datetime(
        &mut self,
        http: impl AsRef<Http>,
        time: Timestamp,
    ) -> Result<()> {
        match self
            .guild_id
            .edit_member(http, self.user.id, |member| {
                member.disable_communication_until_datetime(time)
            })
            .await
        {
            Ok(_) => {
                self.communication_disabled_until = Some(time);
                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Calculates the member's display name.
    ///
    /// The nickname takes priority over the member's username if it exists.
    #[inline]
    pub fn display_name(&self) -> Cow<'_, String> {
        self.nick.as_ref().map_or_else(|| Cow::Owned(self.user.name.clone()), Cow::Borrowed)
    }

    /// Returns the DiscordTag of a Member, taking possible nickname into account.
    #[inline]
    #[must_use]
    pub fn distinct(&self) -> String {
        format!("{}#{:04}", self.display_name(), self.user.discriminator)
    }

    /// Edits the member with the given data. See [`Guild::edit_member`] for
    /// more information.
    ///
    /// See [`EditMember`] for the permission(s) required for separate builder
    /// methods, as well as usage of this.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks necessary permissions.
    pub async fn edit<F>(&self, http: impl AsRef<Http>, f: F) -> Result<Member>
    where
        F: FnOnce(&mut EditMember) -> &mut EditMember,
    {
        let mut edit_member = EditMember::default();
        f(&mut edit_member);
        let map = json::hashmap_to_json_map(edit_member.0);

        http.as_ref().edit_member(self.guild_id.0, self.user.id.0, &map, None).await
    }

    /// Allow a user to communicate, removing their timeout, if there is one.
    ///
    /// **Note**: Requires the [Moderate Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Moderate Members]: Permissions::MODERATE_MEMBERS
    #[doc(alias = "timeout")]
    pub async fn enable_communication(&mut self, http: impl AsRef<Http>) -> Result<()> {
        match self.guild_id.edit_member(&http, self.user.id, EditMember::enable_communication).await
        {
            Ok(_) => {
                self.communication_disabled_until = None;
                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Retrieves the ID and position of the member's highest role in the
    /// hierarchy, if they have one.
    ///
    /// This _may_ return [`None`] if:
    ///
    /// - the user has roles, but they are not present in the cache for cache
    /// inconsistency reasons
    /// - you already have a write lock to the member's guild
    ///
    /// The "highest role in hierarchy" is defined as the role with the highest
    /// position. If two or more roles have the same highest position, then the
    /// role with the lowest ID is the highest.
    #[cfg(feature = "cache")]
    pub fn highest_role_info(&self, cache: impl AsRef<Cache>) -> Option<(RoleId, i64)> {
        let guild_roles = cache.as_ref().guild_field(self.guild_id, |g| g.roles.clone())?;

        let mut highest = None;

        for role_id in &self.roles {
            if let Some(role) = guild_roles.get(role_id) {
                // Skip this role if this role in iteration has:
                //
                // - a position less than the recorded highest
                // - a position equal to the recorded, but a higher ID
                if let Some((id, pos)) = highest {
                    if role.position < pos || (role.position == pos && role.id > id) {
                        continue;
                    }
                }

                highest = Some((role.id, role.position));
            }
        }

        highest
    }

    /// Kick the member from the guild.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Examples
    ///
    /// Kick a member from its guild:
    ///
    /// ```rust,ignore
    /// // assuming a `member` has already been bound
    /// match member.kick().await {
    ///     Ok(()) => println!("Successfully kicked member"),
    ///     Err(Error::Model(ModelError::GuildNotFound)) => {
    ///         println!("Couldn't determine guild of member");
    ///     },
    ///     Err(Error::Model(ModelError::InvalidPermissions(missing_perms))) => {
    ///         println!("Didn't have permissions; missing: {:?}", missing_perms);
    ///     },
    ///     _ => {},
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the Id of the member's guild
    /// could not be determined.
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform the kick.
    ///
    /// Otherwise will return [`Error::Http`] if the current user lacks permission.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    #[inline]
    pub async fn kick(&self, cache_http: impl CacheHttp) -> Result<()> {
        self.kick_with_reason(cache_http, "").await
    }

    /// Kicks the member from the guild, with a reason.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Examples
    ///
    /// Kicks a member from it's guild, with an optional reason:
    ///
    /// ```rust,ignore
    /// match member.kick(&ctx.http, "A Reason").await {
    ///     Ok(()) => println!("Successfully kicked member"),
    ///     Err(Error::Model(ModelError::GuildNotFound)) => {
    ///         println!("Couldn't determine guild of member");
    ///     },
    ///     Err(Error::Model(ModelError::InvalidPermissions(missing_perms))) => {
    ///         println!("Didn't have permissions; missing: {:?}", missing_perms);
    ///     },
    ///     _ => {},
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// In addition to the reasons [`Self::kick`] may return an error,
    /// can also return an error if the given reason is too long.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    pub async fn kick_with_reason(&self, cache_http: impl CacheHttp, reason: &str) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(guild) = cache.guilds.get(&self.guild_id) {
                    let req = Permissions::KICK_MEMBERS;

                    if !guild.has_perms(&cache_http, req).await {
                        return Err(Error::Model(ModelError::InvalidPermissions(req)));
                    }

                    guild.check_hierarchy(cache, self.user.id)?;
                }
            }
        }

        self.guild_id.kick_with_reason(cache_http.http(), self.user.id, reason).await
    }

    /// Moves the member to a voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member is not currently in a
    /// voice channel, or if the current user lacks permission.
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    pub async fn move_to_voice_channel(
        &self,
        http: impl AsRef<Http>,
        channel: impl Into<ChannelId>,
    ) -> Result<Member> {
        self.guild_id.move_member(http, self.user.id, channel).await
    }

    /// Disconnects the member from their voice channel if any.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member is not currently in a
    /// voice channel, or if the current user lacks permission.
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    pub async fn disconnect_from_voice(&self, http: impl AsRef<Http>) -> Result<Member> {
        self.guild_id.disconnect_member(http, self.user.id).await
    }

    /// Returns the guild-level permissions for the member.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // assuming there's a `member` variable gotten from anything.
    /// println!("The permission bits for the member are: {}",
    /// member.permissions(&cache).expect("permissions").bits());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the guild the member's in could not be
    /// found in the cache.
    ///
    /// And/or returns [`ModelError::ItemMissing`] if the "default channel" of the guild is not
    /// found.
    #[cfg(feature = "cache")]
    pub fn permissions(&self, cache: impl AsRef<Cache>) -> Result<Permissions> {
        let perms_opt = cache
            .as_ref()
            .guild_field(self.guild_id, |guild| guild._member_permission_from_member(self));

        match perms_opt {
            Some(perms) => Ok(perms),
            None => Err(From::from(ModelError::GuildNotFound)),
        }
    }

    /// Removes a [`Role`] from the member, editing its roles in-place if the
    /// request was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a role with the given Id does not exist,
    /// or if the current user lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn remove_role(
        &mut self,
        http: impl AsRef<Http>,
        role_id: impl Into<RoleId>,
    ) -> Result<()> {
        let role_id = role_id.into();

        if !self.roles.contains(&role_id) {
            return Ok(());
        }

        match http
            .as_ref()
            .remove_member_role(self.guild_id.0, self.user.id.0, role_id.0, None)
            .await
        {
            Ok(()) => {
                self.roles.retain(|r| r.0 != role_id.0);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Removes one or multiple [`Role`]s from the member. Returns the member's
    /// new roles.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a role with a given Id does not exist,
    /// or if the current user lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn remove_roles(
        &mut self,
        http: impl AsRef<Http>,
        role_ids: &[RoleId],
    ) -> Result<Vec<RoleId>> {
        self.roles.retain(|r| !role_ids.contains(r));

        let mut builder = EditMember::default();
        builder.roles(&self.roles);
        let map = json::hashmap_to_json_map(builder.0);

        match http.as_ref().edit_member(self.guild_id.0, self.user.id.0, &map, None).await {
            Ok(member) => Ok(member.roles),
            Err(why) => {
                self.roles.extend_from_slice(role_ids);

                Err(why)
            },
        }
    }

    /// Retrieves the full role data for the user's roles.
    ///
    /// This is shorthand for manually searching through the Cache.
    ///
    /// If role data can not be found for the member, then [`None`] is returned.
    #[cfg(feature = "cache")]
    pub fn roles(&self, cache: impl AsRef<Cache>) -> Option<Vec<Role>> {
        Some(
            cache
                .as_ref()
                .guild_field(self.guild_id, |g| g.roles.clone())?
                .into_iter()
                .map(|(_, v)| v)
                .filter(|role| self.roles.contains(&role.id))
                .collect(),
        )
    }

    /// Unbans the [`User`] from the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    #[inline]
    pub async fn unban(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().remove_ban(self.guild_id.0, self.user.id.0, None).await
    }

    /// Returns the formatted URL of the member's per guild avatar, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the member has a GIF avatar.
    #[inline]
    #[must_use]
    pub fn avatar_url(&self) -> Option<String> {
        avatar_url(self.guild_id, self.user.id, self.avatar.as_ref())
    }

    /// Retrieves the URL to the current member's avatar, falling back to the
    /// user's avatar, then default avatar if needed.
    ///
    /// This will call [`Self::avatar_url`] first, and if that returns [`None`],
    /// it then falls back to [`User::face()`].
    #[inline]
    #[must_use]
    pub fn face(&self) -> String {
        self.avatar_url().unwrap_or_else(|| self.user.face())
    }
}

impl fmt::Display for Member {
    /// Mentions the user so that they receive a notification.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // assumes a `member` has already been bound
    /// println!("{} is a member!", member);
    /// ```
    ///
    /// This is in the format of `<@USER_ID>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.user.mention(), f)
    }
}

/// A partial amount of data for a member.
///
/// This is used in [`Message`]s from [`Guild`]s.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object), subset specification unknown
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialMember {
    /// Indicator of whether the member can hear in voice channels.
    #[serde(default)]
    pub deaf: bool,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<Timestamp>,
    /// Indicator of whether the member can speak in voice channels
    #[serde(default)]
    pub mute: bool,
    /// The member's nickname, if present.
    ///
    /// Can't be longer than 32 characters.
    pub nick: Option<String>,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: Vec<RoleId>,
    /// Indicator that the member hasn't accepted the rules of the guild yet.
    #[serde(default)]
    pub pending: bool,
    /// Timestamp representing the date since the member is boosting the guild.
    pub premium_since: Option<Timestamp>,
    /// The unique Id of the guild that the member is a part of.
    pub guild_id: Option<GuildId>,
    /// Attached User struct.
    pub user: Option<User>,
    /// The total permissions of the member in a channel, including overrides.
    ///
    /// This is only [`Some`] when returned in an [`Interaction`] object.
    ///
    /// [`Interaction`]: crate::model::application::interaction::Interaction
    pub permissions: Option<Permissions>,
}

#[cfg(feature = "model")]
fn avatar_url(guild_id: GuildId, user_id: UserId, hash: Option<&String>) -> Option<String> {
    hash.map(|hash| {
        let ext = if hash.starts_with("a_") { "gif" } else { "webp" };

        cdn!("/guilds/{}/users/{}/avatars/{}.{}?size=1024", guild_id.0, user_id.0, hash, ext)
    })
}

/// [Discord docs](https://discord.com/developers/docs/resources/channel#thread-member-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadMember {
    /// The id of the thread.
    pub id: Option<ChannelId>,
    /// The id of the user.
    pub user_id: Option<UserId>,
    /// The time the current user last joined the thread.
    pub join_timestamp: Timestamp,
    /// Any user-thread settings, currently only used for notifications
    pub flags: ThreadMemberFlags,
}

bitflags! {
    /// Describes extra features of the message.
    ///
    /// Discord docs: flags field on [Thread Member](https://discord.com/developers/docs/resources/channel#thread-member-object).
    #[derive(Default)]
    pub struct ThreadMemberFlags: u64 {
        // Not documented.
        const NOTIFICATIONS = 1 << 0;
    }
}
