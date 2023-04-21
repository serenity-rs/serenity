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
use crate::model::permissions::Permissions;
use crate::model::prelude::*;
use crate::model::Timestamp;

/// Information about a member of a guild.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Member {
    /// Attached User struct.
    pub user: User,
    /// The member's nickname, if present.
    ///
    /// Can't be longer than 32 characters.
    pub nick: Option<String>,
    /// The guild avatar hash
    pub avatar: Option<String>,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: Vec<RoleId>,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<Timestamp>,
    /// Timestamp representing the date since the member is boosting the guild.
    pub premium_since: Option<Timestamp>,
    /// Indicator of whether the member can hear in voice channels.
    pub deaf: bool,
    /// Indicator of whether the member can speak in voice channels.
    pub mute: bool,
    /// Guild member flags.
    pub flags: GuildMemberFlags,
    /// Indicator that the member hasn't accepted the rules of the guild yet.
    #[serde(default)]
    pub pending: bool,
    /// The total permissions of the member in a channel, including overrides.
    ///
    /// This is only [`Some`] when returned in an [`Interaction`] object.
    ///
    /// [`Interaction`]: crate::model::application::Interaction
    pub permissions: Option<Permissions>,
    /// When the user's timeout will expire and the user will be able to communicate in the guild
    /// again.
    ///
    /// Will be None or a time in the past if the user is not timed out.
    pub communication_disabled_until: Option<Timestamp>,
    /// The unique Id of the guild that the member is a part of.
    #[serde(default)]
    pub guild_id: GuildId,
}

bitflags! {
    /// Flags for a guild member.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object-guild-member-flags).
    #[derive(Default)]
    pub struct GuildMemberFlags: u32 {
        /// Member has left and rejoined the guild. Not editable
        const DID_REJOIN = 1 << 0;
        /// Member has completed onboarding. Not editable
        const COMPLETED_ONBOARDING = 1 << 1;
        /// Member is exempt from guild verification requirements. Editable
        const BYPASSES_VERIFICATION = 1 << 2;
        /// Member has started onboarding. Not editable
        const STARTED_ONBOARDING = 1 << 3;
    }
}

#[cfg(feature = "model")]
impl Member {
    /// Adds a [`Role`] to the member, editing its roles in-place if the request was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a role with the given
    /// Id does not exist.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[inline]
    pub async fn add_role(
        &mut self,
        cache_http: impl CacheHttp,
        role_id: impl Into<RoleId>,
    ) -> Result<()> {
        self.add_roles(cache_http, &[role_id.into()]).await
    }

    /// Adds one or multiple [`Role`]s to the member, editing its roles in-place if the request was
    /// successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a role with a given Id
    /// does not exist.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn add_roles(
        &mut self,
        cache_http: impl CacheHttp,
        role_ids: &[RoleId],
    ) -> Result<()> {
        let mut target_roles = self.roles.clone();
        target_roles.extend_from_slice(role_ids);

        let builder = EditMember::new().roles(target_roles);
        self.edit(cache_http, builder).await
    }

    /// Ban a [`User`] from the guild, deleting a number of days' worth of messages (`dmd`) between
    /// the range 0 and 7.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::DeleteMessageDaysAmount`] if the `dmd` is greater than 7. Can also
    /// return [`Error::Http`] if the current user lacks permission to ban this member.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    #[inline]
    pub async fn ban(&self, http: impl AsRef<Http>, dmd: u8) -> Result<()> {
        self.ban_with_reason(http, dmd, "").await
    }

    /// Ban the member from the guild with a reason. Refer to [`Self::ban`] to further
    /// documentation.
    ///
    /// # Errors
    ///
    /// In addition to the errors [`Self::ban`] may return, can also return
    /// [`Error::ExceededLimit`] if the length of the reason is greater than 512.
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
        let guild = cache.as_ref().guild(self.guild_id)?;

        let mut roles = self
            .roles
            .iter()
            .filter_map(|role_id| guild.roles.get(role_id))
            .collect::<Vec<&Role>>();

        roles.sort_by_key(|&b| Reverse(b));

        let default = Colour::default();

        roles.iter().find(|r| r.colour.0 != default.0).map(|r| r.colour)
    }

    /// Returns the "default channel" of the guild for the member. (This returns the first channel
    /// that can be read by the member, if there isn't one returns [`None`])
    #[cfg(feature = "cache")]
    pub fn default_channel(&self, cache: impl AsRef<Cache>) -> Option<GuildChannel> {
        let guild = self.guild_id.to_guild_cached(&cache)?;

        let member = guild.members.get(&self.user.id)?;

        for channel in guild.channels.values() {
            if channel.kind != ChannelType::Category
                && guild.user_permissions_in(channel, member).ok()?.view_channel()
            {
                return Some(channel.clone());
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
        cache_http: impl CacheHttp,
        time: Timestamp,
    ) -> Result<()> {
        let builder = EditMember::new().disable_communication_until_datetime(time);
        match self.guild_id.edit_member(cache_http, self.user.id, builder).await {
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

    /// Edits the member in place with the given data.
    ///
    /// See [`EditMember`] for the permission(s) required for separate builder methods, as well as
    /// usage of this.
    ///
    /// # Examples
    ///
    /// See [`GuildId::edit_member`] for details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks necessary permissions.
    pub async fn edit(
        &mut self,
        cache_http: impl CacheHttp,
        builder: EditMember<'_>,
    ) -> Result<()> {
        *self = self.guild_id.edit_member(cache_http, self.user.id, builder).await?;
        Ok(())
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
    pub async fn enable_communication(&mut self, cache_http: impl CacheHttp) -> Result<()> {
        let builder = EditMember::new().enable_communication();
        *self = self.guild_id.edit_member(cache_http, self.user.id, builder).await?;
        Ok(())
    }

    /// Retrieves the ID and position of the member's highest role in the hierarchy, if they have
    /// one.
    ///
    /// This _may_ return [`None`] if:
    /// - the user has roles, but they are not present in the cache for cache inconsistency reasons
    /// - you already have a write lock to the member's guild
    ///
    /// The "highest role in hierarchy" is defined as the role with the highest position. If two or
    /// more roles have the same highest position, then the role with the lowest ID is the highest.
    #[cfg(feature = "cache")]
    pub fn highest_role_info(&self, cache: impl AsRef<Cache>) -> Option<(RoleId, u32)> {
        let guild = cache.as_ref().guild(self.guild_id)?;

        let mut highest = None;

        for role_id in &self.roles {
            if let Some(role) = guild.roles.get(role_id) {
                // Skip this role if this role in iteration has:
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
    /// Returns a [`ModelError::GuildNotFound`] if the Id of the member's guild could not be
    /// determined.
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to perform the kick.
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
    /// In addition to the reasons [`Self::kick`] may return an error, can also return an error if
    /// the given reason is too long.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    pub async fn kick_with_reason(&self, cache_http: impl CacheHttp, reason: &str) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                let lookup = cache.guild(self.guild_id).as_deref().cloned();
                if let Some(guild) = lookup {
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
    /// Returns [`Error::Http`] if the member is not currently in a voice channel, or if the
    /// current user lacks permission.
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    pub async fn move_to_voice_channel(
        &self,
        cache_http: impl CacheHttp,
        channel: impl Into<ChannelId>,
    ) -> Result<Member> {
        self.guild_id.move_member(cache_http, self.user.id, channel).await
    }

    /// Disconnects the member from their voice channel if any.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the member is not currently in a voice channel, or if the
    /// current user lacks permission.
    ///
    /// [Move Members]: Permissions::MOVE_MEMBERS
    pub async fn disconnect_from_voice(&self, cache_http: impl CacheHttp) -> Result<Member> {
        self.guild_id.disconnect_member(cache_http, self.user.id).await
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
        match cache.as_ref().guild(self.guild_id) {
            Some(guild) => Ok(guild._member_permission_from_member(self)),
            None => Err(From::from(ModelError::GuildNotFound)),
        }
    }

    /// Removes a [`Role`] from the member, editing its roles in-place if the request was
    /// successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a role with the given Id does not exist, or if the current user
    /// lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn remove_role(
        &mut self,
        cache_http: impl CacheHttp,
        role_id: impl Into<RoleId>,
    ) -> Result<()> {
        self.remove_roles(cache_http, &[role_id.into()]).await
    }

    /// Removes one or multiple [`Role`]s from the member, editing its roles in-place if the
    /// request was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a role with a given Id does not exist, or if the current user
    /// lacks permission.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn remove_roles(
        &mut self,
        cache_http: impl CacheHttp,
        role_ids: &[RoleId],
    ) -> Result<()> {
        let mut target_roles = self.roles.clone();
        target_roles.retain(|r| !role_ids.contains(r));

        let builder = EditMember::new().roles(target_roles);
        self.edit(cache_http, builder).await
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
                .guild(self.guild_id)?
                .roles
                .iter()
                .filter(|(id, _)| self.roles.contains(id))
                .map(|(_, role)| role.clone())
                .collect(),
        )
    }

    /// Unbans the [`User`] from the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// does not have permission to perform bans.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    #[inline]
    pub async fn unban(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().remove_ban(self.guild_id, self.user.id, None).await
    }

    /// Returns the formatted URL of the member's per guild avatar, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the member has a GIF avatar.
    #[inline]
    #[must_use]
    pub fn avatar_url(&self) -> Option<String> {
        avatar_url(self.guild_id, self.user.id, self.avatar.as_ref())
    }

    /// Retrieves the URL to the current member's avatar, falling back to the user's avatar, then
    /// default avatar if needed.
    ///
    /// This will call [`Self::avatar_url`] first, and if that returns [`None`], it then falls back
    /// to [`User::face()`].
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
    /// [`Interaction`]: crate::model::application::Interaction
    pub permissions: Option<Permissions>,
}

#[cfg(feature = "model")]
fn avatar_url(guild_id: GuildId, user_id: UserId, hash: Option<&String>) -> Option<String> {
    hash.map(|hash| {
        let ext = if hash.starts_with("a_") { "gif" } else { "webp" };

        cdn!("/guilds/{}/users/{}/avatars/{}.{}?size=1024", guild_id.0, user_id.0, hash, ext)
    })
}

/// [Discord docs](https://discord.com/developers/docs/resources/channel#thread-member-object),
/// [extra fields](https://discord.com/developers/docs/topics/gateway-events#thread-member-update-thread-member-update-event-extra-fields).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadMember {
    /// The id of the thread.
    ///
    /// This field is omitted on the member sent within each thread in the GUILD_CREATE event.
    pub id: Option<ChannelId>,
    /// The id of the user.
    ///
    /// This field is omitted on the member sent within each thread in the GUILD_CREATE event.
    pub user_id: Option<UserId>,
    /// The time the current user last joined the thread.
    pub join_timestamp: Timestamp,
    /// Any user-thread settings, currently only used for notifications
    pub flags: ThreadMemberFlags,
    /// Additional information about the user.
    ///
    /// This field is omitted on the member sent within each thread in the GUILD_CREATE event.
    ///
    /// This field is only present when `with_member` is set to `true` when calling
    /// List Thread Members or Get Thread Member.
    pub member: Option<Member>,
    /// ID of the guild.
    ///
    /// Always present in [`ThreadMemberUpdateEvent`], otherwise `None`.
    pub guild_id: Option<GuildId>,
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
