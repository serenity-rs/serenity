mod emoji;
mod feature;
mod guild_id;
mod integration;
mod member;
mod partial_guild;
mod role;
mod audit_log;

pub use self::emoji::*;
pub use self::feature::*;
pub use self::guild_id::*;
pub use self::integration::*;
pub use self::member::*;
pub use self::partial_guild::*;
pub use self::role::*;
pub use self::audit_log::*;

use chrono::{DateTime, FixedOffset};
use serde::de::Error as DeError;
use serde_json;
use super::utils::*;
use model::*;

#[cfg(feature = "cache")]
use CACHE;
#[cfg(feature = "model")]
use http;
#[cfg(feature = "model")]
use builder::{EditGuild, EditMember, EditRole};
#[cfg(feature = "model")]
use constants::LARGE_THRESHOLD;

/// A representation of a banning of a user.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct Ban {
    /// The reason given for this ban.
    pub reason: Option<String>,
    /// The user that was banned.
    pub user: User,
}

/// Information about a Discord guild, such as channels, emojis, etc.
#[derive(Clone, Debug)]
pub struct Guild {
    /// Id of a voice channel that's considered the AFK channel.
    pub afk_channel_id: Option<ChannelId>,
    /// The amount of seconds a user can not show any activity in a voice
    /// channel before being moved to an AFK channel -- if one exists.
    pub afk_timeout: u64,
    /// All voice and text channels contained within a guild.
    ///
    /// This contains all channels regardless of permissions (i.e. the ability
    /// of the bot to read from or connect to them).
    pub channels: HashMap<ChannelId, Arc<RwLock<GuildChannel>>>,
    /// Indicator of whether notifications for all messages are enabled by
    /// default in the guild.
    pub default_message_notifications: u64,
    /// All of the guild's custom emojis.
    pub emojis: HashMap<EmojiId, Emoji>,
    /// VIP features enabled for the guild. Can be obtained through the
    /// [Discord Partnership] website.
    ///
    /// [Discord Partnership]: https://discordapp.com/partners
    pub features: Vec<Feature>,
    /// The hash of the icon used by the guild.
    ///
    /// In the client, this appears on the guild list on the left-hand side.
    pub icon: Option<String>,
    /// The unique Id identifying the guild.
    ///
    /// This is equivilant to the Id of the default role (`@everyone`) and also
    /// that of the default channel (typically `#general`).
    pub id: GuildId,
    /// The date that the current user joined the guild.
    pub joined_at: DateTime<FixedOffset>,
    /// Indicator of whether the guild is considered "large" by Discord.
    pub large: bool,
    /// The number of members in the guild.
    pub member_count: u64,
    /// Users who are members of the guild.
    ///
    /// Members might not all be available when the [`ReadyEvent`] is received
    /// if the [`member_count`] is greater than the `LARGE_THRESHOLD` set by
    /// the library.
    ///
    /// [`ReadyEvent`]: events/struct.ReadyEvent.html
    pub members: HashMap<UserId, Member>,
    /// Indicator of whether the guild requires multi-factor authentication for
    /// [`Role`]s or [`User`]s with moderation permissions.
    ///
    /// [`Role`]: struct.Role.html
    /// [`User`]: struct.User.html
    pub mfa_level: u64,
    /// The name of the guild.
    pub name: String,
    /// The Id of the [`User`] who owns the guild.
    ///
    /// [`User`]: struct.User.html
    pub owner_id: UserId,
    /// A mapping of [`User`]s' Ids to their current presences.
    ///
    /// [`User`]: struct.User.html
    pub presences: HashMap<UserId, Presence>,
    /// The region that the voice servers that the guild uses are located in.
    pub region: String,
    /// A mapping of the guild's roles.
    pub roles: HashMap<RoleId, Role>,
    /// An identifying hash of the guild's splash icon.
    ///
    /// If the [`InviteSplash`] feature is enabled, this can be used to generate
    /// a URL to a splash image.
    ///
    /// [`InviteSplash`]: enum.Feature.html#variant.InviteSplash
    pub splash: Option<String>,
    /// Indicator of the current verification level of the guild.
    pub verification_level: VerificationLevel,
    /// A mapping of of [`User`]s to their current voice state.
    ///
    /// [`User`]: struct.User.html
    pub voice_states: HashMap<UserId, VoiceState>,
}

#[cfg(feature = "model")]
impl Guild {
    #[cfg(feature = "cache")]
    /// Returns the "default" channel of the guild.
    /// (This returns the first channel that can be read by the bot, if there isn't one,
    /// returns `None`)
    pub fn default_channel(&self) -> Option<GuildChannel> {
        let uid = CACHE.read().unwrap().user.id;

        for (cid, channel) in &self.channels {
            if self.permissions_for(*cid, uid).read_messages() {
                return Some(channel.read().unwrap().clone());
            }
        }

        None
    }

    /// Returns the guaranteed "default" channel of the guild.
    /// (This returns the first channel that can be read by everyone, if there isn't one,
    /// returns `None`)
    /// Note however that this is very costy if used in a server with lots of channels,
    /// members, or both.
    pub fn default_channel_guaranteed(&self) -> Option<GuildChannel> {
        for (cid, channel) in &self.channels {
            for memid in self.members.keys() {
                if self.permissions_for(*cid, *memid).read_messages() {
                    return Some(channel.read().unwrap().clone());
                }
            }
        }

        None
    }

    #[cfg(feature = "cache")]
    fn has_perms(&self, mut permissions: Permissions) -> Result<bool> {
        let member = match self.members.get(&CACHE.read().unwrap().user.id) {
            Some(member) => member,
            None => return Err(Error::Model(ModelError::ItemMissing)),
        };

        let default_channel = match self.default_channel() {
            Some(dc) => dc,
            None => return Err(Error::Model(ModelError::ItemMissing)),
        };

        let perms = self.permissions_for(
            default_channel.id,
            member.user.read().unwrap().id,
        );
        permissions.remove(perms);

        Ok(permissions.is_empty())
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
    /// // assumes a `user` and `guild` have already been bound
    /// let _ = guild.ban(user, 4);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// Returns a [`ModelError::DeleteMessageDaysAmount`] if the number of
    /// days' worth of messages to delete is over the maximum.
    ///
    /// [`ModelError::DeleteMessageDaysAmount`]:
    /// enum.ModelError.html#variant.DeleteMessageDaysAmount
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`Guild::ban`]: struct.Guild.html#method.ban
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    pub fn ban<U: Into<UserId>>(&self, user: U, delete_message_days: u8) -> Result<()> {
        if delete_message_days > 7 {
            return Err(Error::Model(
                ModelError::DeleteMessageDaysAmount(delete_message_days),
            ));
        }

        #[cfg(feature = "cache")]
        {
            let req = permissions::BAN_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.ban(user, delete_message_days)
    }

    /// Retrieves a list of [`Ban`]s for the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`Ban`]: struct.Ban.html
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    pub fn bans(&self) -> Result<Vec<Ban>> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::BAN_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.bans()
    }

    /// Retrieves a list of [`AuditLogs`] for the guild.
    ///
    /// [`AuditLogs`]: audit_log/struct.AuditLogs.html
    #[inline]
    pub fn audit_logs(&self) -> Result<AuditLogs> { self.id.audit_logs() }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// [`Guild`]: struct.Guild.html
    #[inline]
    pub fn channels(&self) -> Result<HashMap<ChannelId, GuildChannel>> { self.id.channels() }

    /// Creates a guild with the data provided.
    ///
    /// Only a [`PartialGuild`] will be immediately returned, and a full
    /// [`Guild`] will be received over a [`Shard`].
    ///
    /// **Note**: This endpoint is usually only available for user accounts.
    /// Refer to Discord's information for the endpoint [here][whitelist] for
    /// more information. If you require this as a bot, re-think what you are
    /// doing and if it _really_ needs to be doing this.
    ///
    /// # Examples
    ///
    /// Create a guild called `"test"` in the [US West region] with no icon:
    ///
    /// ```rust,ignore
    /// use serenity::model::{Guild, Region};
    ///
    /// let _guild = Guild::create_guild("test", Region::UsWest, None);
    /// ```
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`PartialGuild`]: struct.PartialGuild.html
    /// [`Shard`]: ../gateway/struct.Shard.html
    /// [US West region]: enum.Region.html#variant.UsWest
    /// [whitelist]: https://discordapp.com/developers/docs/resources/guild#create-guild
    pub fn create(name: &str, region: Region, icon: Option<&str>) -> Result<PartialGuild> {
        let map = json!({
            "icon": icon,
            "name": name,
            "region": region.name(),
        });

        http::create_guild(&map)
    }

    /// Creates a new [`Channel`] in the guild.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use serenity::model::ChannelType;
    ///
    /// // assuming a `guild` has already been bound
    ///
    /// let _ = guild.create_channel("my-test-channel", ChannelType::Text);
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`Channel`]: struct.Channel.html
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    pub fn create_channel(&self, name: &str, kind: ChannelType) -> Result<GuildChannel> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::MANAGE_CHANNELS;

            if !self.has_perms(req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.create_channel(name, kind)
    }

    /// Creates an emoji in the guild with a name and base64-encoded image. The
    /// [`utils::read_image`] function is provided for you as a simple method to
    /// read an image and encode it into base64, if you are reading from the
    /// filesystem.
    ///
    /// The name of the emoji must be at least 2 characters long and can only
    /// contain alphanumeric characters and underscores.
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
    /// [`utils::read_image`]: ../fn.read_image.html
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn create_emoji(&self, name: &str, image: &str) -> Result<Emoji> {
        self.id.create_emoji(name, image)
    }

    /// Creates an integration for the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn create_integration<I>(&self, integration_id: I, kind: &str) -> Result<()>
        where I: Into<IntegrationId> {
        self.id.create_integration(integration_id, kind)
    }

    /// Creates a new role in the guild with the data set, if any.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Create a role which can be mentioned, with the name 'test':
    ///
    /// ```rust,ignore
    /// // assuming a `guild` has been bound
    ///
    /// let role = guild.create_role(|r| r.hoist(true).name("role"));
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    pub fn create_role<F>(&self, f: F) -> Result<Role>
        where F: FnOnce(EditRole) -> EditRole {
        #[cfg(feature = "cache")]
        {
            let req = permissions::MANAGE_ROLES;

            if !self.has_perms(req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.create_role(f)
    }

    /// Deletes the current guild if the current user is the owner of the
    /// guild.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a [`ModelError::InvalidUser`]
    /// if the current user is not the guild owner.
    ///
    /// [`ModelError::InvalidUser`]: enum.ModelError.html#variant.InvalidUser
    pub fn delete(&self) -> Result<PartialGuild> {
        #[cfg(feature = "cache")]
        {
            if self.owner_id != CACHE.read().unwrap().user.id {
                let req = permissions::MANAGE_GUILD;

                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.delete()
    }

    /// Deletes an [`Emoji`] from the guild.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn delete_emoji<E: Into<EmojiId>>(&self, emoji_id: E) -> Result<()> {
        self.id.delete_emoji(emoji_id)
    }

    /// Deletes an integration by Id from the guild.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn delete_integration<I: Into<IntegrationId>>(&self, integration_id: I) -> Result<()> {
        self.id.delete_integration(integration_id)
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
        self.id.delete_role(role_id)
    }

    /// Edits the current guild with new data where specified.
    ///
    /// Refer to `EditGuild`'s documentation for a full list of methods.
    ///
    /// **Note**: Requires the current user to have the [Manage Guild]
    /// permission.
    ///
    /// # Examples
    ///
    /// Change a guild's icon using a file name "icon.png":
    ///
    /// ```rust,ignore
    /// use serenity::utils;
    ///
    /// // We are using read_image helper function from utils.
    /// let base64_icon = utils::read_image("./icon.png")
    ///     .expect("Failed to read image");
    ///
    /// guild.edit(|g| g.icon(base64_icon));
    /// ```
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditGuild) -> EditGuild {
        #[cfg(feature = "cache")]
        {
            let req = permissions::MANAGE_GUILD;

            if !self.has_perms(req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        match self.id.edit(f) {
            Ok(guild) => {
                self.afk_channel_id = guild.afk_channel_id;
                self.afk_timeout = guild.afk_timeout;
                self.default_message_notifications = guild.default_message_notifications;
                self.emojis = guild.emojis;
                self.features = guild.features;
                self.icon = guild.icon;
                self.mfa_level = guild.mfa_level;
                self.name = guild.name;
                self.owner_id = guild.owner_id;
                self.region = guild.region;
                self.roles = guild.roles;
                self.splash = guild.splash;
                self.verification_level = guild.verification_level;

                Ok(())
            },
            Err(why) => Err(why),
        }
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
        self.id.edit_emoji(emoji_id, name)
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
        self.id.edit_member(user_id, f)
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
    pub fn edit_nickname(&self, new_nickname: Option<&str>) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::CHANGE_NICKNAME;

            if !self.has_perms(req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.edit_nickname(new_nickname)
    }

    /// Edits a role, optionally setting its fields.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// Make a role hoisted:
    ///
    /// ```rust,ignore
    /// guild.edit_role(RoleId(7), |r| r.hoist(true));
    /// ```
    ///
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[inline]
    pub fn edit_role<F, R>(&self, role_id: R, f: F) -> Result<Role>
        where F: FnOnce(EditRole) -> EditRole, R: Into<RoleId> {
        self.id.edit_role(role_id, f)
    }

    /// Gets an emoji in the guild by Id.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn emoji<E: Into<EmojiId>>(&self, emoji_id: E) -> Result<Emoji> { self.id.emoji(emoji_id) }

    /// Gets a list of all of the guild's emojis.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn emojis(&self) -> Result<Vec<Emoji>> { self.id.emojis() }

    /// Gets a partial amount of guild data by its Id.
    ///
    /// Requires that the current user be in the guild.
    #[inline]
    pub fn get<G: Into<GuildId>>(guild_id: G) -> Result<PartialGuild> { guild_id.into().get() }

    /// Returns the formatted URL of the guild's icon, if one exists.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(
            |icon| format!(cdn!("/icons/{}/{}.webp"), self.id, icon),
        )
    }

    /// Gets all integration of the guild.
    ///
    /// This performs a request over the REST API.
    #[inline]
    pub fn integrations(&self) -> Result<Vec<Integration>> { self.id.integrations() }

    /// Retrieves the active invites for the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn invites(&self) -> Result<Vec<RichInvite>> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::MANAGE_GUILD;

            if !self.has_perms(req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.invites()
    }

    /// Checks if the guild is 'large'. A guild is considered large if it has
    /// more than 250 members.
    #[inline]
    pub fn is_large(&self) -> bool { self.members.len() > LARGE_THRESHOLD as usize }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    #[inline]
    pub fn kick<U: Into<UserId>>(&self, user_id: U) -> Result<()> { self.id.kick(user_id) }

    /// Leaves the guild.
    #[inline]
    pub fn leave(&self) -> Result<()> { self.id.leave() }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`Member`]: struct.Member.html
    #[inline]
    pub fn member<U: Into<UserId>>(&self, user_id: U) -> Result<Member> { self.id.member(user_id) }

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
        self.id.members(limit, after)
    }

    /// Gets a list of all the members (satisfying the status provided to the function) in this
    /// guild.
    pub fn members_with_status(&self, status: OnlineStatus) -> Vec<&Member> {
        let mut members = vec![];

        for (&id, member) in &self.members {
            match self.presences.get(&id) {
                Some(presence) => {
                    if status == presence.status {
                        members.push(member);
                    }
                },
                None => continue,
            }
        }

        members
    }

    /// Retrieves the first [`Member`] found that matches the name - with an
    /// optional discriminator - provided.
    ///
    /// Searching with a discriminator given is the most precise form of lookup,
    /// as no two people can share the same username *and* discriminator.
    ///
    /// If a member can not be found by username or username#discriminator,
    /// then a search will be done for the nickname. When searching by nickname,
    /// the hash (`#`) and everything after it is included in the search.
    ///
    /// The following are valid types of searches:
    ///
    /// - **username**: "zey"
    /// - **username and discriminator**: "zey#5479"
    /// - **nickname**: "zeyla" or "zeylas#nick"
    ///
    /// [`Member`]: struct.Member.html
    pub fn member_named(&self, name: &str) -> Option<&Member> {
        let (name, discrim) = if let Some(pos) = name.find('#') {
            let split = name.split_at(pos);

            match split.1.parse::<u16>() {
                Ok(discrim_int) => (split.0, Some(discrim_int)),
                Err(_) => (name, None),
            }
        } else {
            (&name[..], None)
        };

        self.members
            .values()
            .find(|member| {
                let name_matches = member.user.read().unwrap().name == name;
                let discrim_matches = match discrim {
                    Some(discrim) => member.user.read().unwrap().discriminator == discrim,
                    None => true,
                };

                name_matches && discrim_matches
            })
            .or_else(|| {
                self.members.values().find(|member| {
                    member.nick.as_ref().map_or(false, |nick| nick == name)
                })
            })
    }

    /// Moves a member to a specific voice channel.
    ///
    /// Requires the [Move Members] permission.
    ///
    /// [Move Members]: permissions/constant.MOVE_MEMBERS.html
    #[inline]
    pub fn move_member<C, U>(&self, user_id: U, channel_id: C) -> Result<()>
        where C: Into<ChannelId>, U: Into<UserId> {
        self.id.move_member(user_id, channel_id)
    }

    /// Calculate a [`User`]'s permissions in a given channel in the guild.
    ///
    /// [`User`]: struct.User.html
    pub fn permissions_for<C, U>(&self, channel_id: C, user_id: U) -> Permissions
        where C: Into<ChannelId>, U: Into<UserId> {
        use super::permissions::*;

        let user_id = user_id.into();

        // The owner has all permissions in all cases.
        if user_id == self.owner_id {
            return Permissions::all();
        }

        let channel_id = channel_id.into();

        // Start by retrieving the @everyone role's permissions.
        let everyone = match self.roles.get(&RoleId(self.id.0)) {
            Some(everyone) => everyone,
            None => {
                error!("(╯°□°）╯︵ ┻━┻ @everyone role ({}) missing in '{}'",
                self.id,
                self.name);

                return Permissions::empty();
            },
        };

        // Create a base set of permissions, starting with `@everyone`s.
        let mut permissions = everyone.permissions;

        let member = match self.members.get(&user_id) {
            Some(member) => member,
            None => return everyone.permissions,
        };

        for &role in &member.roles {
            if let Some(role) = self.roles.get(&role) {
                permissions |= role.permissions;
            } else {
                warn!("(╯°□°）╯︵ ┻━┻ {} on {} has non-existent role {:?}",
                member.user.read().unwrap().id,
                self.id,
                role);
            }
        }

        // Administrators have all permissions in any channel.
        if permissions.contains(ADMINISTRATOR) {
            return Permissions::all();
        }

        if let Some(channel) = self.channels.get(&channel_id) {
            let channel = channel.read().unwrap();

            // If this is a text channel, then throw out voice permissions.
            if channel.kind == ChannelType::Text {
                permissions &= !(CONNECT | SPEAK | MUTE_MEMBERS | DEAFEN_MEMBERS | MOVE_MEMBERS |
                                 USE_VAD);
            }

            // Apply the permission overwrites for the channel for each of the
            // overwrites that - first - applies to the member's roles, and then
            // the member itself.
            //
            // First apply the denied permission overwrites for each, then apply
            // the allowed.

            // Roles
            for overwrite in &channel.permission_overwrites {
                if let PermissionOverwriteType::Role(role) = overwrite.kind {
                    if role.0 != self.id.0 && !member.roles.contains(&role) {
                        continue;
                    }

                    permissions = (permissions & !overwrite.deny) | overwrite.allow;
                }
            }

            // Member
            for overwrite in &channel.permission_overwrites {
                if PermissionOverwriteType::Member(user_id) != overwrite.kind {
                    continue;
                }

                permissions = (permissions & !overwrite.deny) | overwrite.allow;
            }
        } else {
            warn!("(╯°□°）╯︵ ┻━┻ Guild {} does not contain channel {}",
            self.id,
            channel_id);
        }

        // The default channel is always readable.
        if channel_id.0 == self.id.0 {
            permissions |= READ_MESSAGES;
        }

        // No SEND_MESSAGES => no message-sending-related actions
        // If the member does not have the `SEND_MESSAGES` permission, then
        // throw out message-able permissions.
        if !permissions.contains(SEND_MESSAGES) {
            permissions &= !(SEND_TTS_MESSAGES | MENTION_EVERYONE | EMBED_LINKS | ATTACH_FILES);
        }

        // If the member does not have the `READ_MESSAGES` permission, then
        // throw out actionable permissions.
        if !permissions.contains(READ_MESSAGES) {
            permissions &= KICK_MEMBERS | BAN_MEMBERS | ADMINISTRATOR | MANAGE_GUILD |
                           CHANGE_NICKNAME | MANAGE_NICKNAMES;
        }

        permissions
    }

    /// Retrieves the count of the number of [`Member`]s that would be pruned
    /// with the number of given days.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`GuildPrune`]: struct.GuildPrune.html
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    pub fn prune_count(&self, days: u16) -> Result<GuildPrune> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::KICK_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.prune_count(days)
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
    pub fn shard_id(&self) -> u64 { self.id.shard_id() }

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
    #[cfg(all(feature = "utils", not(feature = "cache")))]
    #[inline]
    pub fn shard_id(&self, shard_count: u64) -> u64 { self.id.shard_id(shard_count) }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon.as_ref().map(
            |icon| format!(cdn!("/splashes/{}/{}.webp"), self.id, icon),
        )
    }

    /// Starts an integration sync for the given integration Id.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn start_integration_sync<I: Into<IntegrationId>>(&self, integration_id: I) -> Result<()> {
        self.id.start_integration_sync(integration_id)
    }

    /// Starts a prune of [`Member`]s.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`GuildPrune`]: struct.GuildPrune.html
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    pub fn start_prune(&self, days: u16) -> Result<GuildPrune> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::KICK_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.start_prune(days)
    }

    /// Unbans the given [`User`] from the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    pub fn unban<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            let req = permissions::BAN_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Model(ModelError::InvalidPermissions(req)));
            }
        }

        self.id.unban(user_id)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> { self.id.webhooks() }
}

impl<'de> Deserialize<'de> for Guild {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("id").and_then(|x| x.as_str()).and_then(|x| {
            x.parse::<u64>().ok()
        });

        if let Some(guild_id) = id {
            if let Some(array) = map.get_mut("channels").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(channel) = value.as_object_mut() {
                        channel.insert(
                            "guild_id".to_owned(),
                            Value::Number(Number::from(guild_id)),
                        );
                    }
                }
            }

            if let Some(array) = map.get_mut("members").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(member) = value.as_object_mut() {
                        member.insert(
                            "guild_id".to_owned(),
                            Value::Number(Number::from(guild_id)),
                        );
                    }
                }
            }
        }

        let afk_channel_id = match map.remove("afk_channel_id") {
            Some(v) => {
                serde_json::from_value::<Option<ChannelId>>(v).map_err(
                    DeError::custom,
                )?
            },
            None => None,
        };
        let afk_timeout = map.remove("afk_timeout")
            .ok_or_else(|| DeError::custom("expected guild afk_timeout"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let channels = map.remove("channels")
            .ok_or_else(|| DeError::custom("expected guild channels"))
            .and_then(deserialize_guild_channels)
            .map_err(DeError::custom)?;
        let default_message_notifications = map.remove("default_message_notifications")
            .ok_or_else(|| {
                DeError::custom("expected guild default_message_notifications")
            })
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let emojis = map.remove("emojis")
            .ok_or_else(|| DeError::custom("expected guild emojis"))
            .and_then(deserialize_emojis)
            .map_err(DeError::custom)?;
        let features = map.remove("features")
            .ok_or_else(|| DeError::custom("expected guild features"))
            .and_then(serde_json::from_value::<Vec<Feature>>)
            .map_err(DeError::custom)?;
        let icon = match map.remove("icon") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let id = map.remove("id")
            .ok_or_else(|| DeError::custom("expected guild id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;
        let joined_at = map.remove("joined_at")
            .ok_or_else(|| DeError::custom("expected guild joined_at"))
            .and_then(DateTime::deserialize)
            .map_err(DeError::custom)?;
        let large = map.remove("large")
            .ok_or_else(|| DeError::custom("expected guild large"))
            .and_then(bool::deserialize)
            .map_err(DeError::custom)?;
        let member_count = map.remove("member_count")
            .ok_or_else(|| DeError::custom("expected guild member_count"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let members = map.remove("members")
            .ok_or_else(|| DeError::custom("expected guild members"))
            .and_then(deserialize_members)
            .map_err(DeError::custom)?;
        let mfa_level = map.remove("mfa_level")
            .ok_or_else(|| DeError::custom("expected guild mfa_level"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let name = map.remove("name")
            .ok_or_else(|| DeError::custom("expected guild name"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let owner_id = map.remove("owner_id")
            .ok_or_else(|| DeError::custom("expected guild owner_id"))
            .and_then(UserId::deserialize)
            .map_err(DeError::custom)?;
        let presences = map.remove("presences")
            .ok_or_else(|| DeError::custom("expected guild presences"))
            .and_then(deserialize_presences)
            .map_err(DeError::custom)?;
        let region = map.remove("region")
            .ok_or_else(|| DeError::custom("expected guild region"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let roles = map.remove("roles")
            .ok_or_else(|| DeError::custom("expected guild roles"))
            .and_then(deserialize_roles)
            .map_err(DeError::custom)?;
        let splash = match map.remove("splash") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let verification_level = map.remove("verification_level")
            .ok_or_else(|| DeError::custom("expected guild verification_level"))
            .and_then(VerificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let voice_states = map.remove("voice_states")
            .ok_or_else(|| DeError::custom("expected guild voice_states"))
            .and_then(deserialize_voice_states)
            .map_err(DeError::custom)?;

        Ok(Self {
            afk_channel_id: afk_channel_id,
            afk_timeout: afk_timeout,
            channels: channels,
            default_message_notifications: default_message_notifications,
            emojis: emojis,
            features: features,
            icon: icon,
            id: id,
            joined_at: joined_at,
            large: large,
            member_count: member_count,
            members: members,
            mfa_level: mfa_level,
            name: name,
            owner_id: owner_id,
            presences: presences,
            region: region,
            roles: roles,
            splash: splash,
            verification_level: verification_level,
            voice_states: voice_states,
        })
    }
}

/// Information relating to a guild's widget embed.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct GuildEmbed {
    /// The Id of the channel to show the embed for.
    pub channel_id: ChannelId,
    /// Whether the widget embed is enabled.
    pub enabled: bool,
}

/// Representation of the number of members that would be pruned by a guild
/// prune operation.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct GuildPrune {
    /// The number of members that would be pruned by the operation.
    pub pruned: u64,
}

/// Basic information about a guild.
#[derive(Clone, Debug, Deserialize)]
pub struct GuildInfo {
    /// The unique Id of the guild.
    ///
    /// Can be used to calculate creation date.
    pub id: GuildId,
    /// The hash of the icon of the guild.
    ///
    /// This can be used to generate a URL to the guild's icon image.
    pub icon: Option<String>,
    /// The name of the guild.
    pub name: String,
    /// Indicator of whether the current user is the owner.
    pub owner: bool,
    /// The permissions that the current user has.
    pub permissions: Permissions,
}

#[cfg(any(feature = "model", feature = "utils"))]
impl GuildInfo {
    /// Returns the formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(
            |icon| format!(cdn!("/icons/{}/{}.webp"), self.id, icon),
        )
    }
}

impl From<PartialGuild> for GuildContainer {
    fn from(guild: PartialGuild) -> GuildContainer { GuildContainer::Guild(guild) }
}

impl From<GuildId> for GuildContainer {
    fn from(guild_id: GuildId) -> GuildContainer { GuildContainer::Id(guild_id) }
}

impl From<u64> for GuildContainer {
    fn from(id: u64) -> GuildContainer { GuildContainer::Id(GuildId(id)) }
}

#[cfg(feature = "model")]
impl InviteGuild {
    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon.as_ref().map(
            |icon| format!(cdn!("/splashes/{}/{}.webp"), self.id, icon),
        )
    }
}

/// Data for an unavailable guild.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct GuildUnavailable {
    /// The Id of the [`Guild`] that is unavailable.
    ///
    /// [`Guild`]: struct.Guild.html
    pub id: GuildId,
    /// Indicator of whether the guild is unavailable.
    ///
    /// This should always be `true`.
    pub unavailable: bool,
}

#[allow(large_enum_variant)]
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum GuildStatus {
    OnlinePartialGuild(PartialGuild),
    OnlineGuild(Guild),
    Offline(GuildUnavailable),
}

#[cfg(feature = "model")]
impl GuildStatus {
    /// Retrieves the Id of the inner [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn id(&self) -> GuildId {
        match *self {
            GuildStatus::Offline(offline) => offline.id,
            GuildStatus::OnlineGuild(ref guild) => guild.id,
            GuildStatus::OnlinePartialGuild(ref partial_guild) => partial_guild.id,
        }
    }
}

enum_number!(
    #[doc="The level to set as criteria prior to a user being able to send
    messages in a [`Guild`].

    [`Guild`]: struct.Guild.html"]
    VerificationLevel {
        /// Does not require any verification.
        None = 0,
        /// Must have a verified email on the user's Discord account.
        Low = 1,
        /// Must also be a registered user on Discord for longer than 5 minutes.
        Medium = 2,
        /// Must also be a member of the guild for longer than 10 minutes.
        High = 3,
        /// Must have a verified phone on the user's Discord account.
        Higher = 4,
    }
);

impl VerificationLevel {
    pub fn num(&self) -> u64 {
        match *self {
            VerificationLevel::None => 0,
            VerificationLevel::Low => 1,
            VerificationLevel::Medium => 2,
            VerificationLevel::High => 3,
            VerificationLevel::Higher => 4,
        }
    }
}
