#[cfg(feature = "cache")]
use std::cmp::Reverse;
use std::fmt;

#[cfg(feature = "model")]
use crate::builder::EditMember;
#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "model")]
use crate::http::Http;
use crate::model::prelude::*;
#[cfg(feature = "model")]
use crate::model::utils::avatar_url;

/// Information about a member of a guild.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object),
/// [extra fields](https://discord.com/developers/docs/topics/gateway-events#guild-member-add-guild-member-add-extra-fields).
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct Member {
    /// Attached User struct.
    pub user: User,
    /// The member's nickname, if present.
    ///
    /// Can't be longer than 32 characters.
    pub nick: Option<FixedString<u8>>,
    /// The guild avatar hash
    pub avatar: Option<ImageHash>,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: FixedArray<RoleId>,
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
    /// If the member is currently flagged for sending excessive DMs to non-friend server members
    /// in the last 24 hours.
    ///
    /// Will be None or a time in the past if the user is not flagged.
    pub unusual_dm_activity_until: Option<Timestamp>,
}

bitflags! {
    /// Flags for a guild member.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object-guild-member-flags).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct GuildMemberFlags: u32 {
        /// Member has left and rejoined the guild. Not editable
        const DID_REJOIN = 1 << 0;
        /// Member has completed onboarding. Not editable
        const COMPLETED_ONBOARDING = 1 << 1;
        /// Member is exempt from guild verification requirements. Editable
        const BYPASSES_VERIFICATION = 1 << 2;
        /// Member has started onboarding. Not editable
        const STARTED_ONBOARDING = 1 << 3;
        /// Member is a guest and can only access the voice channel they were invited to. Not
        /// editable
        const IS_GUEST = 1 << 4;
        /// Member has started Server Guide new member actions. Not editable
        const STARTED_HOME_ACTIONS = 1 << 5;
        /// Member has completed Server Guide new member actions. Not editable
        const COMPLETED_HOME_ACTIONS = 1 << 6;
        /// Member's username, display name, or nickname is blocked by AutoMod. Not editable
        const AUTOMOD_QUARANTINED_USERNAME = 1 << 7;
        /// Member has dismissed the DM settings upsell. Not editable
        const DM_SETTINGS_UPSELL_ACKNOWLEDGED = 1 << 9;
    }
}

#[cfg(feature = "model")]
impl Member {
    /// Adds a [`Role`] to the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if a role with the given
    /// Id does not exist.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn add_role(&self, http: &Http, role_id: RoleId, reason: Option<&str>) -> Result<()> {
        http.add_member_role(self.guild_id, self.user.id, role_id, reason).await
    }

    /// Adds one or multiple [`Role`]s to the member.
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
        &self,
        http: &Http,
        role_ids: &[RoleId],
        reason: Option<&str>,
    ) -> Result<()> {
        for &role_id in role_ids {
            self.add_role(http, role_id, reason).await?;
        }

        Ok(())
    }

    /// Ban a [`User`] from the guild, deleting a number of days' worth of messages (`dmd`) between
    /// the range 0 and 7.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::TooLarge`] if the `dmd` is greater than 7. Can also
    /// return [`Error::Http`] if the current user lacks permission to ban this member.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    pub async fn ban(&self, http: &Http, dmd: u8, audit_log_reason: Option<&str>) -> Result<()> {
        self.guild_id.ban(http, self.user.id, dmd, audit_log_reason).await
    }

    /// Determines the member's colour.
    #[cfg(feature = "cache")]
    pub fn colour(&self, cache: &Cache) -> Option<Colour> {
        let guild = cache.guild(self.guild_id)?;

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
    pub fn default_channel(&self, cache: &Cache) -> Option<GuildChannel> {
        let guild = self.guild_id.to_guild_cached(cache)?;

        let member = guild.members.get(&self.user.id)?;

        for channel in &guild.channels {
            if channel.kind != ChannelType::Category
                && guild.user_permissions_in(channel, member).view_channel()
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
    pub async fn disable_communication_until(
        &mut self,
        http: &Http,
        time: Timestamp,
    ) -> Result<()> {
        let builder = EditMember::new().disable_communication_until(time);
        match self.guild_id.edit_member(http, self.user.id, builder).await {
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
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.nick.as_ref().or(self.user.global_name.as_ref()).unwrap_or(&self.user.name)
    }

    /// Returns the DiscordTag of a Member, taking possible nickname into account.
    #[must_use]
    #[deprecated = "Use User::tag to get the correct Discord username format or Self::display_name for the name that users will see."]
    pub fn distinct(&self) -> String {
        if let Some(discriminator) = self.user.discriminator {
            format!("{}#{:04}", self.display_name(), discriminator.get())
        } else {
            self.display_name().to_string()
        }
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
    pub async fn edit(&mut self, http: &Http, builder: EditMember<'_>) -> Result<()> {
        *self = self.guild_id.edit_member(http, self.user.id, builder).await?;
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
    pub async fn enable_communication(&mut self, http: &Http) -> Result<()> {
        let builder = EditMember::new().enable_communication();
        *self = self.guild_id.edit_member(http, self.user.id, builder).await?;
        Ok(())
    }

    /// Kick the member from the guild.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Examples
    ///
    /// Kick a member from the guild:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::guild::Member;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let member: Member = unimplemented!();
    /// // assuming a `member` has already been bound
    /// member.kick(&http, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the Id of the member's guild could not be
    /// determined.
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Kick Members]: Permissions::KICK_MEMBERS
    pub async fn kick(&self, http: &Http, reason: Option<&str>) -> Result<()> {
        self.guild_id.kick(http, self.user.id, reason).await
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
    pub async fn move_to_voice_channel(&self, http: &Http, channel: ChannelId) -> Result<Member> {
        self.guild_id.move_member(http, self.user.id, channel).await
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
    pub async fn disconnect_from_voice(&self, http: &Http) -> Result<Member> {
        self.guild_id.disconnect_member(http, self.user.id).await
    }

    /// Returns the guild-level permissions for the member.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the guild the member's in could not be
    /// found in the cache.
    ///
    /// And/or returns [`ModelError::ItemMissing`] if the "default channel" of the guild is not
    /// found.
    #[cfg(feature = "cache")]
    #[deprecated = "Use Guild::user_permissions_in, as this doesn't consider permission overwrites"]
    pub fn permissions(&self, cache: &Cache) -> Result<Permissions> {
        let guild = cache.guild(self.guild_id).ok_or(ModelError::GuildNotFound)?;

        #[expect(deprecated)]
        Ok(guild.member_permissions(self))
    }

    /// Removes a [`Role`] from the member.
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
        &self,
        http: &Http,
        role_id: RoleId,
        reason: Option<&str>,
    ) -> Result<()> {
        http.remove_member_role(self.guild_id, self.user.id, role_id, reason).await
    }

    /// Removes one or multiple [`Role`]s from the member.
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
        &self,
        http: &Http,
        role_ids: &[RoleId],
        reason: Option<&str>,
    ) -> Result<()> {
        for &role_id in role_ids {
            self.remove_role(http, role_id, reason).await?;
        }

        Ok(())
    }

    /// Retrieves the full role data for the user's roles.
    ///
    /// This is shorthand for manually searching through the Cache.
    ///
    /// If role data can not be found for the member, then [`None`] is returned.
    #[cfg(feature = "cache")]
    pub fn roles(&self, cache: &Cache) -> Option<Vec<Role>> {
        Some(
            cache
                .guild(self.guild_id)?
                .roles
                .iter()
                .filter(|r| self.roles.contains(&r.id))
                .cloned()
                .collect(),
        )
    }

    /// Unbans the [`User`] from the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have permission to perform bans.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    pub async fn unban(&self, http: &Http, reason: Option<&str>) -> Result<()> {
        http.remove_ban(self.guild_id, self.user.id, reason).await
    }

    /// Returns the formatted URL of the member's per guild avatar, if one exists.
    ///
    /// This will produce a WEBP image URL, or GIF if the member has a GIF avatar.
    #[must_use]
    pub fn avatar_url(&self) -> Option<String> {
        avatar_url(Some(self.guild_id), self.user.id, self.avatar.as_ref())
    }

    /// Retrieves the URL to the current member's avatar, falling back to the user's avatar, then
    /// default avatar if needed.
    ///
    /// This will call [`Self::avatar_url`] first, and if that returns [`None`], it then falls back
    /// to [`User::face()`].
    #[must_use]
    pub fn face(&self) -> String {
        self.avatar_url().unwrap_or_else(|| self.user.face())
    }
}

impl fmt::Display for Member {
    /// Mentions the user so that they receive a notification.
    ///
    /// This is in the format of `<@USER_ID>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.user.mention(), f)
    }
}

impl ExtractKey<UserId> for Member {
    fn extract_key(&self) -> &UserId {
        &self.user.id
    }
}

/// A partial amount of data for a member.
///
/// This is used in [`Message`]s from [`Guild`]s.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object),
/// subset specification unknown (field type "partial member" is used in
/// [link](https://discord.com/developers/docs/topics/gateway-events#message-create),
/// [link](https://discord.com/developers/docs/resources/invite#invite-stage-instance-object),
/// [link](https://discord.com/developers/docs/topics/gateway-events#message-create),
/// [link](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-resolved-data-structure),
/// [link](https://discord.com/developers/docs/interactions/receiving-and-responding#message-interaction-object))
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
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
    pub nick: Option<FixedString<u8>>,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: FixedArray<RoleId>,
    /// Indicator that the member hasn't accepted the rules of the guild yet.
    #[serde(default)]
    pub pending: bool,
    /// Timestamp representing the date since the member is boosting the guild.
    pub premium_since: Option<Timestamp>,
    /// The unique Id of the guild that the member is a part of.
    ///
    /// Manually inserted in [`Reaction::deserialize`].
    pub guild_id: Option<GuildId>,
    /// Attached User struct.
    pub user: Option<User>,
    /// The total permissions of the member in a channel, including overrides.
    ///
    /// This is only [`Some`] when returned in an [`Interaction`] object.
    ///
    /// [`Interaction`]: crate::model::application::Interaction
    pub permissions: Option<Permissions>,
    /// If the member is currently flagged for sending excessive DMs to non-friend server members
    /// in the last 24 hours.
    ///
    /// Will be None or a time in the past if the user is not flagged.
    pub unusual_dm_activity_until: Option<Timestamp>,
}

impl From<PartialMember> for Member {
    fn from(partial: PartialMember) -> Self {
        let (pending, deaf, mute) = (partial.pending(), partial.deaf(), partial.mute());
        let mut member = Member {
            __generated_flags: MemberGeneratedFlags::empty(),
            user: partial.user.unwrap_or_default(),
            nick: partial.nick,
            avatar: None,
            roles: partial.roles,
            joined_at: partial.joined_at,
            premium_since: partial.premium_since,
            flags: GuildMemberFlags::default(),
            permissions: partial.permissions,
            communication_disabled_until: None,
            guild_id: partial.guild_id.unwrap_or_default(),
            unusual_dm_activity_until: partial.unusual_dm_activity_until,
        };

        member.set_pending(pending);
        member.set_deaf(deaf);
        member.set_mute(mute);
        member
    }
}

impl From<Member> for PartialMember {
    fn from(member: Member) -> Self {
        let (pending, deaf, mute) = (member.pending(), member.deaf(), member.mute());
        let mut partial = PartialMember {
            __generated_flags: PartialMemberGeneratedFlags::empty(),
            joined_at: member.joined_at,
            nick: member.nick,
            roles: member.roles,
            premium_since: member.premium_since,
            guild_id: Some(member.guild_id),
            user: Some(member.user),
            permissions: member.permissions,
            unusual_dm_activity_until: member.unusual_dm_activity_until,
        };

        partial.set_deaf(deaf);
        partial.set_mute(mute);
        partial.set_pending(pending);
        partial
    }
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialThreadMember {
    /// The time the current user last joined the thread.
    pub join_timestamp: Timestamp,
    /// Any user-thread settings, currently only used for notifications
    pub flags: ThreadMemberFlags,
}

/// A model representing a user in a Guild Thread.
///
/// [Discord docs](https://discord.com/developers/docs/resources/channel#thread-member-object),
/// [extra fields](https://discord.com/developers/docs/topics/gateway-events#thread-member-update-thread-member-update-event-extra-fields).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ThreadMember {
    #[serde(flatten)]
    pub inner: PartialThreadMember,
    /// The id of the thread.
    pub id: ChannelId,
    /// The id of the user.
    pub user_id: UserId,
    /// Additional information about the user.
    ///
    /// This field is only present when `with_member` is set to `true` when calling
    /// List Thread Members or Get Thread Member, or inside [`ThreadMembersUpdateEvent`].
    pub member: Option<Member>,
    /// ID of the guild.
    ///
    /// Always present in [`ThreadMemberUpdateEvent`], otherwise `None`.
    pub guild_id: Option<GuildId>,
    // According to https://discord.com/developers/docs/topics/gateway-events#thread-members-update,
    // > the thread member objects will also include the guild member and nullable presence objects
    // > for each added thread member
    // Which implies that ThreadMember has a presence field. But https://discord.com/developers/docs/resources/channel#thread-member-object
    // says that's not true. I'm not adding the presence field here for now
}

bitflags! {
    /// Describes extra features of the message.
    ///
    /// Discord docs: flags field on [Thread Member](https://discord.com/developers/docs/resources/channel#thread-member-object).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct ThreadMemberFlags: u64 {
        // Not documented.
        const NOTIFICATIONS = 1 << 0;
    }
}
