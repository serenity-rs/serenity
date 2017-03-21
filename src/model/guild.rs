use serde_json::builder::ObjectBuilder;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fmt;
use super::utils::{
    decode_emojis,
    decode_members,
    decode_presences,
    decode_roles,
    decode_voice_states,
    into_map,
    into_string,
    opt,
    remove,
};
use super::*;
use ::client::rest;
use ::constants::LARGE_THRESHOLD;
use ::internal::prelude::*;
use ::utils::builder::{EditGuild, EditMember, EditRole, Search};
use ::utils::decode_array;

#[cfg(feature="cache")]
use std::mem;

#[cfg(feature="cache")]
use ::client::CACHE;
#[cfg(feature="cache")]
use ::utils::Colour;

impl From<PartialGuild> for GuildContainer {
    fn from(guild: PartialGuild) -> GuildContainer {
        GuildContainer::Guild(guild)
    }
}

impl From<GuildId> for GuildContainer {
    fn from(guild_id: GuildId) -> GuildContainer {
        GuildContainer::Id(guild_id)
    }
}

impl From<u64> for GuildContainer {
    fn from(id: u64) -> GuildContainer {
        GuildContainer::Id(GuildId(id))
    }
}

impl Emoji {
    /// Deletes the emoji.
    ///
    /// **Note**: The [Manage Emojis] permission is required.
    ///
    /// **Note**: Only user accounts may use this method.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[cfg(feature="cache")]
    pub fn delete(&self) -> Result<()> {
        match self.find_guild_id() {
            Some(guild_id) => rest::delete_emoji(guild_id.0, self.id.0),
            None => Err(Error::Client(ClientError::ItemMissing)),
        }
    }

    /// Edits the emoji by updating it with a new name.
    ///
    /// **Note**: The [Manage Emojis] permission is required.
    ///
    /// **Note**: Only user accounts may use this method.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[cfg(feature="cache")]
    pub fn edit(&mut self, name: &str) -> Result<()> {
        match self.find_guild_id() {
            Some(guild_id) => {
                let map = ObjectBuilder::new()
                    .insert("name", name)
                    .build();

                match rest::edit_emoji(guild_id.0, self.id.0, &map) {
                    Ok(emoji) => {
                        mem::replace(self, emoji);

                        Ok(())
                    },
                    Err(why) => Err(why),
                }
            },
            None => Err(Error::Client(ClientError::ItemMissing)),
        }
    }

    /// Finds the [`Guild`] that owns the emoji by looking through the Cache.
    ///
    /// [`Guild`]: struct.Guild.html
    #[cfg(feature="cache")]
    pub fn find_guild_id(&self) -> Option<GuildId> {
        for guild in CACHE.read().unwrap().guilds.values() {
            let guild = guild.read().unwrap();

            if guild.emojis.contains_key(&self.id) {
                return Some(guild.id);
            }
        }

        None
    }

    /// Generates a URL to the emoji's image.
    #[inline]
    pub fn url(&self) -> String {
        format!(cdn!("/emojis/{}.png"), self.id)
    }
}

impl fmt::Display for Emoji {
    /// Formats the emoji into a string that will cause Discord clients to
    /// render the emoji.
    ///
    /// This is in the format of: `<:NAME:EMOJI_ID>`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("<:")?;
        f.write_str(&self.name)?;
        fmt::Write::write_char(f, ':')?;
        fmt::Display::fmt(&self.id, f)?;
        fmt::Write::write_char(f, '>')
    }
}

impl fmt::Display for EmojiId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Emoji> for EmojiId {
    /// Gets the Id of an `Emoji`.
    fn from(emoji: Emoji) -> EmojiId {
        emoji.id
    }
}

impl GuildInfo {
    /// Returns the formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }
}

impl InviteGuild {
    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
    }
}

impl Guild {
    #[cfg(feature="cache")]
    fn has_perms(&self, mut permissions: Permissions) -> Result<bool> {
        let member = match self.members.get(&CACHE.read().unwrap().user.id) {
            Some(member) => member,
            None => return Err(Error::Client(ClientError::ItemMissing)),
        };

        let perms = self.permissions_for(ChannelId(self.id.0), member.user.read().unwrap().id);
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
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// Returns a [`ClientError::DeleteMessageDaysAmount`] if the number of
    /// days' worth of messages to delete is over the maximum.
    ///
    /// [`ClientError::DeleteMessageDaysAmount`]: ../client/enum.ClientError.html#variant.DeleteMessageDaysAmount
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Guild::ban`]: struct.Guild.html#method.ban
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    pub fn ban<U: Into<UserId>>(&self, user: U, delete_message_days: u8)
        -> Result<()> {
        if delete_message_days > 7 {
            return Err(Error::Client(ClientError::DeleteMessageDaysAmount(delete_message_days)));
        }

        #[cfg(feature="cache")]
        {
            let req = permissions::BAN_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
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
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`Ban`]: struct.Ban.html
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    pub fn bans(&self) -> Result<Vec<Ban>> {
        #[cfg(feature="cache")]
        {
            let req = permissions::BAN_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        self.id.get_bans()
    }

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
    /// [`Shard`]: ../client/gateway/struct.Shard.html
    /// [US West region]: enum.Region.html#variant.UsWest
    /// [whitelist]: https://discordapp.com/developers/docs/resources/guild#create-guild
    pub fn create(name: &str, region: Region, icon: Option<&str>) -> Result<PartialGuild> {
        let map = ObjectBuilder::new()
            .insert("icon", icon)
            .insert("name", name)
            .insert("region", region.name())
            .build();

        rest::create_guild(&map)
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
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`Channel`]: struct.Channel.html
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    pub fn create_channel(&mut self, name: &str, kind: ChannelType) -> Result<GuildChannel> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_CHANNELS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
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
    /// [`EditProfile::avatar`]: ../utils/builder/struct.EditProfile.html#method.avatar
    /// [`utils::read_image`]: ../utils/fn.read_image.html
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
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Context::create_role`]: ../client/struct.Context.html#method.create_role
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    pub fn create_role<F>(&self, f: F) -> Result<Role>
        where F: FnOnce(EditRole) -> EditRole {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_ROLES;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        self.id.create_role(f)
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Guild> {
        let mut map = into_map(value)?;

        let id = remove(&mut map, "id").and_then(GuildId::decode)?;

        let channels = {
            let mut channels = HashMap::new();

            let vals = decode_array(remove(&mut map, "channels")?,
                |v| GuildChannel::decode_guild(v, id))?;

            for channel in vals {
                channels.insert(channel.id, Arc::new(RwLock::new(channel)));
            }

            channels
        };

        Ok(Guild {
            afk_channel_id: opt(&mut map, "afk_channel_id", ChannelId::decode)?,
            afk_timeout: req!(remove(&mut map, "afk_timeout")?.as_u64()),
            channels: channels,
            default_message_notifications: req!(remove(&mut map, "default_message_notifications")?.as_u64()),
            emojis: remove(&mut map, "emojis").and_then(decode_emojis)?,
            features: remove(&mut map, "features").and_then(|v| decode_array(v, Feature::decode_str))?,
            icon: opt(&mut map, "icon", into_string)?,
            id: id,
            joined_at: remove(&mut map, "joined_at").and_then(into_string)?,
            large: req!(remove(&mut map, "large")?.as_bool()),
            member_count: req!(remove(&mut map, "member_count")?.as_u64()),
            members: remove(&mut map, "members").and_then(decode_members)?,
            mfa_level: req!(remove(&mut map, "mfa_level")?.as_u64()),
            name: remove(&mut map, "name").and_then(into_string)?,
            owner_id: remove(&mut map, "owner_id").and_then(UserId::decode)?,
            presences: remove(&mut map, "presences").and_then(decode_presences)?,
            region: remove(&mut map, "region").and_then(into_string)?,
            roles: remove(&mut map, "roles").and_then(decode_roles)?,
            splash: opt(&mut map, "splash", into_string)?,
            verification_level: remove(&mut map, "verification_level").and_then(VerificationLevel::decode)?,
            voice_states: remove(&mut map, "voice_states").and_then(decode_voice_states)?,
        })
    }

    /// Deletes the current guild if the current user is the owner of the
    /// guild.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a [`ClientError::InvalidUser`]
    /// if the current user is not the guild owner.
    ///
    /// [`ClientError::InvalidUser`]: ../client/enum.ClientError.html#variant.InvalidUser
    pub fn delete(&self) -> Result<PartialGuild> {
        #[cfg(feature="cache")]
        {
            if self.owner_id != CACHE.read().unwrap().user.id {
                let req = permissions::MANAGE_GUILD;

                return Err(Error::Client(ClientError::InvalidPermissions(req)));
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
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Context::edit_guild`]: ../client/struct.Context.html#method.edit_guild
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditGuild) -> EditGuild {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_GUILD;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
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
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to change their own
    /// nickname.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Change Nickname]: permissions/constant.CHANGE_NICKNAME.html
    pub fn edit_nickname(&self, new_nickname: Option<&str>) -> Result<()> {
        #[cfg(feature="cache")]
        {
            let req = permissions::CHANGE_NICKNAME;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
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

    /// Gets a partial amount of guild data by its Id.
    ///
    /// Requires that the current user be in the guild.
    #[inline]
    pub fn get<G: Into<GuildId>>(guild_id: G) -> Result<PartialGuild> {
        guild_id.into().get()
    }

    /// Gets a list of the guild's bans.
    ///
    /// Requires the [Ban Members] permission.
    #[inline]
    pub fn get_bans(&self) -> Result<Vec<Ban>> {
        self.id.get_bans()
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// [`Guild`]: struct.Guild.html
    #[inline]
    pub fn get_channels(&self) -> Result<HashMap<ChannelId, GuildChannel>> {
        self.id.get_channels()
    }

    /// Gets an emoji in the guild by Id.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn get_emoji<E: Into<EmojiId>>(&self, emoji_id: E) -> Result<Emoji> {
        self.id.get_emoji(emoji_id)
    }

    /// Gets a list of all of the guild's emojis.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn get_emojis(&self) -> Result<Vec<Emoji>> {
        self.id.get_emojis()
    }

    /// Gets all integration of the guild.
    ///
    /// This performs a request over the REST API.
    #[inline]
    pub fn get_integrations(&self) -> Result<Vec<Integration>> {
        self.id.get_integrations()
    }

    /// Retrieves the active invites for the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn get_invites(&self) -> Result<Vec<RichInvite>> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_GUILD;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        self.id.get_invites()
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`Member`]: struct.Member.html
    #[inline]
    pub fn get_member<U: Into<UserId>>(&self, user_id: U) -> Result<Member> {
        self.id.get_member(user_id)
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
        self.id.get_members(limit, after)
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
    pub fn get_member_named(&self, name: &str) -> Option<&Member> {
        let hash_pos = name.find('#');

        let (name, discrim) = if let Some(pos) = hash_pos {
            let split = name.split_at(pos);

            (split.0, Some(split.1))
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
            }).or_else(|| self.members.values().find(|member| {
                member.nick.as_ref().map_or(false, |nick| nick == name)
            }))
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
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`GuildPrune`]: struct.GuildPrune.html
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    pub fn get_prune_count(&self, days: u16) -> Result<GuildPrune> {
        #[cfg(feature="cache")]
        {
            let req = permissions::KICK_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        self.id.get_prune_count(days)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn get_webhooks(&self) -> Result<Vec<Webhook>> {
        self.id.get_webhooks()
    }

    /// Returns the formatted URL of the guild's icon, if one exists.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }

    /// Checks if the guild is 'large'. A guild is considered large if it has
    /// more than 250 members.
    #[inline]
    pub fn is_large(&self) -> bool {
        self.members.len() > LARGE_THRESHOLD as usize
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    #[inline]
    pub fn kick<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        self.id.kick(user_id)
    }

    /// Leaves the guild.
    #[inline]
    pub fn leave(&self) -> Result<PartialGuild> {
        self.id.leave()
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
    pub fn permissions_for<C, U>(&self, channel_id: C, user_id: U)
        -> Permissions where C: Into<ChannelId>, U: Into<UserId> {
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
                permissions &= !(CONNECT | SPEAK | MUTE_MEMBERS |
                    DEAFEN_MEMBERS | MOVE_MEMBERS | USE_VAD);
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
                    if !member.roles.contains(&role) || role.0 == self.id.0 {
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
            permissions &= !(SEND_TTS_MESSAGES |
                             MENTION_EVERYONE |
                             EMBED_LINKS |
                             ATTACH_FILES);
        }

        // If the member does not have the `READ_MESSAGES` permission, then
        // throw out actionable permissions.
        if !permissions.contains(READ_MESSAGES) {
            permissions &= KICK_MEMBERS | BAN_MEMBERS | ADMINISTRATOR |
                MANAGE_GUILD | CHANGE_NICKNAME | MANAGE_NICKNAMES;
        }

        permissions
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
    /// **Note**: Bot users can not search.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Channel`]: enum.Channel.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`Message`]: struct.Message.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    /// [`search_channels`]: #method.search_channels
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    pub fn search<F: FnOnce(Search) -> Search>(&self, f: F) -> Result<SearchResult> {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.id.search(f)
    }

    /// Performs a search request to the API for the guild's [`Message`]s in
    /// given channels.
    ///
    /// This will search all of the messages in the guild's provided
    /// [`Channel`]s by Id that you have the [Read Message History] permission
    /// to. Use [`search`] to search all of a guild's [channel][`GuildChannel`]s
    /// at once.
    ///
    /// Refer to the documentation for the [`Search`] builder for examples and
    /// more information.
    ///
    /// **Note**: Bot users can not search.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Channel`]: enum.Channel.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`Message`]: struct.Message.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    /// [`search`]: #method.search
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    pub fn search_channels<F>(&self, channel_ids: &[ChannelId], f: F)
        -> Result<SearchResult> where F: FnOnce(Search) -> Search {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.id.search_channels(channel_ids, f)
    }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
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
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`GuildPrune`]: struct.GuildPrune.html
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    pub fn start_prune(&self, days: u16) -> Result<GuildPrune> {
        #[cfg(feature="cache")]
        {
            let req = permissions::KICK_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
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
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    pub fn unban<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        #[cfg(feature="cache")]
        {
            let req = permissions::BAN_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        self.id.unban(user_id)
    }
}

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

impl fmt::Display for GuildId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
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

impl From<Integration> for IntegrationId {
    /// Gets the Id of integration.
    fn from(integration: Integration) -> IntegrationId {
        integration.id
    }
}

impl Member {
    /// Adds a [`Role`] to the member, editing its roles in-place if the request
    /// was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature="cache")]
    pub fn add_role<R: Into<RoleId>>(&mut self, role_id: R) -> Result<()> {
        let role_id = role_id.into();

        if self.roles.contains(&role_id) {
            return Ok(());
        }

        let guild_id = self.find_guild()?;

        match rest::add_member_role(guild_id.0, self.user.read().unwrap().id.0, role_id.0) {
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
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature="cache")]
    pub fn add_roles(&mut self, role_ids: &[RoleId]) -> Result<()> {
        let guild_id = self.find_guild()?;
        self.roles.extend_from_slice(role_ids);

        let map = EditMember::default().roles(&self.roles).0.build();

        match rest::edit_member(guild_id.0, self.user.read().unwrap().id.0, &map) {
            Ok(()) => Ok(()),
            Err(why) => {
                self.roles.retain(|r| !role_ids.contains(r));

                Err(why)
            }
        }
    }

    /// Ban the member from its guild, deleting the last X number of
    /// days' worth of messages.
    ///
    /// **Note**: Requires the [Ban Members] role.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::GuildNotFound`] if the guild could not be
    /// found.
    ///
    /// [`ClientError::GuildNotFound`]: ../client/enum.ClientError.html#variant.GuildNotFound
    ///
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[cfg(feature="cache")]
    pub fn ban(&self, delete_message_days: u8) -> Result<()> {
        rest::ban_user(self.find_guild()?.0, self.user.read().unwrap().id.0, delete_message_days)
    }

    /// Determines the member's colour.
    #[cfg(feature="cache")]
    pub fn colour(&self) -> Option<Colour> {
        let guild_id = match self.find_guild() {
            Ok(guild_id) => guild_id,
            Err(_) => return None,
        };

        let cache = CACHE.read().unwrap();
        let guild = match cache.guilds.get(&guild_id) {
            Some(guild) => guild.read().unwrap(),
            None => return None,
        };

        let mut roles = self.roles
            .iter()
            .filter_map(|role_id| guild.roles.get(role_id))
            .collect::<Vec<&Role>>();
        roles.sort_by(|a, b| b.cmp(a));

        let default = Colour::default();

        roles.iter().find(|r| r.colour.0 != default.0).map(|r| r.colour)
    }

    #[doc(hidden)]
    pub fn decode_guild(guild_id: GuildId, mut value: Value) -> Result<Member> {
        if let Some(v) = value.as_object_mut() {
            v.insert("guild_id".to_owned(), Value::U64(guild_id.0));
        }

        Self::decode(value)
    }

    /// Calculates the member's display name.
    ///
    /// The nickname takes priority over the member's username if it exists.
    #[inline]
    pub fn display_name(&self) -> Cow<String> {
        self.nick.as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(self.user.read().unwrap().name.clone()))
    }

    /// Returns the DiscordTag of a Member, taking possible nickname into account.
    #[inline]
    pub fn distinct(&self) -> String {
        format!("{}#{}", self.display_name(), self.user.read().unwrap().discriminator)
    }

    /// Edits the member with the given data. See [`Context::edit_member`] for
    /// more information.
    ///
    /// See [`EditMember`] for the permission(s) required for separate builder
    /// methods, as well as usage of this.
    ///
    /// [`Context::edit_member`]: ../client/struct.Context.html#method.edit_member
    /// [`EditMember`]: ../builder/struct.EditMember.html
    #[cfg(feature="cache")]
    pub fn edit<F: FnOnce(EditMember) -> EditMember>(&self, f: F) -> Result<()> {
        let guild_id = self.find_guild()?;
        let map = f(EditMember::default()).0.build();

        rest::edit_member(guild_id.0, self.user.read().unwrap().id.0, &map)
    }

    /// Finds the Id of the [`Guild`] that the member is in.
    ///
    /// If some value is present in [`guild_id`], then that value is returned.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::GuildNotFound`] if the guild could not be
    /// found.
    ///
    /// [`ClientError::GuildNotFound`]: ../client/enum.ClientError.html#variant.GuildNotFound
    /// [`Guild`]: struct.Guild.html
    /// [`guild_id`]: #structfield.guild_id
    #[cfg(feature="cache")]
    pub fn find_guild(&self) -> Result<GuildId> {
        if let Some(guild_id) = self.guild_id {
            return Ok(guild_id);
        }

        for guild in CACHE.read().unwrap().guilds.values() {
            let guild = guild.read().unwrap();

            let predicate = guild.members
                .values()
                .any(|m| m.joined_at == self.joined_at && m.user.read().unwrap().id == self.user.read().unwrap().id);

            if predicate {
                return Ok(guild.id);
            }
        }

        Err(Error::Client(ClientError::GuildNotFound))
    }

    /// Removes a [`Role`] from the member, editing its roles in-place if the
    /// request was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature="cache")]
    pub fn remove_role<R: Into<RoleId>>(&mut self, role_id: R) -> Result<()> {
        let role_id = role_id.into();

        if !self.roles.contains(&role_id) {
            return Ok(());
        }

        let guild_id = self.find_guild()?;

        match rest::remove_member_role(guild_id.0, self.user.read().unwrap().id.0, role_id.0) {
            Ok(()) => {
                self.roles.retain(|r| r.0 != role_id.0);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Removes one or multiple [`Role`]s from the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature="cache")]
    pub fn remove_roles(&mut self, role_ids: &[RoleId]) -> Result<()> {
        let guild_id = self.find_guild()?;
        self.roles.retain(|r| !role_ids.contains(r));

        let map = EditMember::default().roles(&self.roles).0.build();

        match rest::edit_member(guild_id.0, self.user.read().unwrap().id.0, &map) {
            Ok(()) => Ok(()),
            Err(why) => {
                self.roles.extend_from_slice(role_ids);

                Err(why)
            },
        }
    }

    /// Retrieves the full role data for the user's roles.
    ///
    /// This is shorthand for manually searching through the CACHE.
    ///
    /// If role data can not be found for the member, then `None` is returned.
    #[cfg(feature="cache")]
    pub fn roles(&self) -> Option<Vec<Role>> {
        CACHE.read().unwrap()
            .guilds
            .values()
            .find(|guild| guild
                .read()
                .unwrap()
                .members
                .values()
                .any(|m| m.user.read().unwrap().id == self.user.read().unwrap().id && m.joined_at == *self.joined_at))
            .map(|guild| guild
                .read()
                .unwrap()
                .roles
                .values()
                .filter(|role| self.roles.contains(&role.id))
                .cloned()
                .collect())
    }

    /// Unbans the [`User`] from the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[cfg(feature="cache")]
    pub fn unban(&self) -> Result<()> {
        rest::remove_ban(self.find_guild()?.0, self.user.read().unwrap().id.0)
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
    // This is in the format of `<@USER_ID>`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.user.read().unwrap().mention(), f)
    }
}

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
    /// Returns a [`ClientError::DeleteMessageDaysAmount`] if the number of
    /// days' worth of messages to delete is over the maximum.
    ///
    /// [`ClientError::DeleteMessageDaysAmount`]: ../client/enum.ClientError.html#variant.DeleteMessageDaysAmount
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    pub fn ban<U: Into<UserId>>(&self, user: U, delete_message_days: u8) -> Result<()> {
        if delete_message_days > 7 {
            return Err(Error::Client(ClientError::DeleteMessageDaysAmount(delete_message_days)));
        }

        self.id.ban(user, delete_message_days)
    }

    /// Creates a [`GuildChannel`] in the guild.
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
    /// use serenity::model::ChannelType;
    ///
    /// guild.create_channel("test", ChannelType::Voice);
    /// ```
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`rest::create_channel`]: ../client/rest/fn.create_channel.html
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn create_channel(&self, name: &str, kind: ChannelType) -> Result<GuildChannel> {
        self.id.create_channel(name, kind)
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
    /// See the documentation for [`Guild::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Guild::create_role`]: struct.Guild.html#method.create_role
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[inline]
    pub fn create_role<F: FnOnce(EditRole) -> EditRole>(&self, f: F) -> Result<Role> {
        self.id.create_role(f)
    }

    /// Deletes the current guild if the current user is the owner of the
    /// guild.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    #[inline]
    pub fn delete(&self) -> Result<PartialGuild> {
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
    /// **Note**: Requires the current user to have the [Manage Guild]
    /// permission.
    ///
    /// [`Context::edit_guild`]: ../client/struct.Context.html#method.edit_guild
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditGuild) -> EditGuild {
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
    /// use serenity::model::GuildId;
    ///
    /// GuildId(7).edit_member(user_id, |m| m.mute(true).roles(&vec![role_id]));
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
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to change their own
    /// nickname.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Change Nickname]: permissions/constant.CHANGE_NICKNAME.html
    #[inline]
    pub fn edit_nickname(&self, new_nickname: Option<&str>) -> Result<()> {
        self.id.edit_nickname(new_nickname)
    }

    /// Gets a partial amount of guild data by its Id.
    ///
    /// Requires that the current user be in the guild.
    #[inline]
    pub fn get<G: Into<GuildId>>(guild_id: G) -> Result<PartialGuild> {
        guild_id.into().get()
    }

    /// Gets a list of the guild's bans.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[inline]
    pub fn get_bans(&self) -> Result<Vec<Ban>> {
        self.id.get_bans()
    }

    /// Gets all of the guild's channels over the REST API.
    ///
    /// [`Guild`]: struct.Guild.html
    #[inline]
    pub fn get_channels(&self) -> Result<HashMap<ChannelId, GuildChannel>> {
        self.id.get_channels()
    }

    /// Gets an emoji in the guild by Id.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn get_emoji<E: Into<EmojiId>>(&self, emoji_id: E) -> Result<Emoji> {
        self.id.get_emoji(emoji_id)
    }

    /// Gets a list of all of the guild's emojis.
    ///
    /// Requires the [Manage Emojis] permission.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    #[inline]
    pub fn get_emojis(&self) -> Result<Vec<Emoji>> {
        self.id.get_emojis()
    }

    /// Gets all integration of the guild.
    ///
    /// This performs a request over the REST API.
    #[inline]
    pub fn get_integrations(&self) -> Result<Vec<Integration>> {
        self.id.get_integrations()
    }

    /// Gets all of the guild's invites.
    ///
    /// Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    #[inline]
    pub fn get_invites(&self) -> Result<Vec<RichInvite>> {
        self.id.get_invites()
    }

    /// Gets a user's [`Member`] for the guild by Id.
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`Member`]: struct.Member.html
    pub fn get_member<U: Into<UserId>>(&self, user_id: U) -> Result<Member> {
        self.id.get_member(user_id)
    }

    /// Gets a list of the guild's members.
    ///
    /// Optionally pass in the `limit` to limit the number of results. Maximum
    /// value is 1000. Optionally pass in `after` to offset the results by a
    /// [`User`]'s Id.
    ///
    /// [`User`]: struct.User.html
    pub fn get_members<U>(&self, limit: Option<u64>, after: Option<U>)
        -> Result<Vec<Member>> where U: Into<UserId> {
        self.id.get_members(limit, after)
    }

    /// Gets the number of [`Member`]s that would be pruned with the given
    /// number of days.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    #[inline]
    pub fn get_prune_count(&self, days: u16) -> Result<GuildPrune> {
        self.id.get_prune_count(days)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn get_webhooks(&self) -> Result<Vec<Webhook>> {
        self.id.get_webhooks()
    }

    /// Kicks a [`Member`] from the guild.
    ///
    /// Requires the [Kick Members] permission.
    ///
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    #[inline]
    pub fn kick<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        self.id.kick(user_id)
    }

    /// Returns a formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }

    /// Leaves the guild.
    #[inline]
    pub fn leave(&self) -> Result<PartialGuild> {
        self.id.leave()
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
    /// **Note**: Bot users can not search.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Channel`]: enum.Channel.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`Message`]: struct.Message.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    /// [`search_channels`]: #method.search_channels
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    pub fn search<F: FnOnce(Search) -> Search>(&self, f: F) -> Result<SearchResult> {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.id.search(f)
    }

    /// Performs a search request to the API for the guild's [`Message`]s in
    /// given channels.
    ///
    /// This will search all of the messages in the guild's provided
    /// [`Channel`]s by Id that you have the [Read Message History] permission
    /// to. Use [`search`] to search all of a guild's [channel][`GuildChannel`]s
    /// at once.
    ///
    /// Refer to the documentation for the [`Search`] builder for examples and
    /// more information.
    ///
    /// **Note**: Bot users can not search.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user is a bot.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: ../client/enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Channel`]: enum.Channel.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`Message`]: struct.Message.html
    /// [`Search`]: ../utils/builder/struct.Search.html
    /// [`search`]: #method.search
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    pub fn search_channels<F>(&self, channel_ids: &[ChannelId], f: F)
        -> Result<SearchResult> where F: FnOnce(Search) -> Search {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        self.id.search_channels(channel_ids, f)
    }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
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

    /// Unbans a [`User`] from the guild.
    ///
    /// Requires the [Ban Members] permission.
    ///
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[inline]
    pub fn unban<U: Into<UserId>>(&self, user_id: U) -> Result<()> {
        self.id.unban(user_id)
    }
}

impl PossibleGuild<Guild> {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Self> {
        let mut value = into_map(value)?;
        if remove(&mut value, "unavailable").ok().and_then(|v| v.as_bool()).unwrap_or(false) {
            remove(&mut value, "id").and_then(GuildId::decode).map(PossibleGuild::Offline)
        } else {
            Guild::decode(Value::Object(value)).map(PossibleGuild::Online)
        }
    }

    /// Retrieves the Id of the inner [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn id(&self) -> GuildId {
        match *self {
            PossibleGuild::Offline(guild_id) => guild_id,
            PossibleGuild::Online(ref live_guild) => live_guild.id,
        }
    }
}

impl PossibleGuild<PartialGuild> {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Self> {
        let mut value = into_map(value)?;
        if remove(&mut value, "unavailable").ok().and_then(|v| v.as_bool()).unwrap_or(false) {
            remove(&mut value, "id").and_then(GuildId::decode).map(PossibleGuild::Offline)
        } else {
            PartialGuild::decode(Value::Object(value)).map(PossibleGuild::Online)
        }
    }

    /// Retrieves the Id of the inner [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn id(&self) -> GuildId {
        match *self {
            PossibleGuild::Offline(id) => id,
            PossibleGuild::Online(ref live_guild) => live_guild.id,
        }
    }
}

impl Role {
    /// Deletes the role.
    ///
    /// **Note** Requires the [Manage Roles] permission.
    ///
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature="cache")]
    #[inline]
    pub fn delete(&self) -> Result<()> {
        rest::delete_role(self.find_guild()?.0, self.id.0)
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
    /// // assuming a `guild` and `role_id` have been bound
    //
    /// guild.edit_role(role_id, |r| r.hoist(true));
    /// ```
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature="cache")]
    pub fn edit_role<F: FnOnce(EditRole) -> EditRole>(&self, f: F) -> Result<Role> {
        match self.find_guild() {
            Ok(guild_id) => guild_id.edit_role(self.id, f),
            Err(why) => Err(why),
        }
    }

    /// Searches the cache for the guild that owns the role.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::GuildNotFound`] if a guild is not in the cache
    /// that contains the role.
    ///
    /// [`ClientError::GuildNotFound`]: ../client/enum.ClientError.html#variant.GuildNotFound
    #[cfg(feature="cache")]
    pub fn find_guild(&self) -> Result<GuildId> {
        for guild in CACHE.read().unwrap().guilds.values() {
            let guild = guild.read().unwrap();

            if guild.roles.contains_key(&RoleId(self.id.0)) {
                return Ok(guild.id);
            }
        }

        Err(Error::Client(ClientError::GuildNotFound))
    }

    /// Check that the role has the given permission.
    #[inline]
    pub fn has_permission(&self, permission: Permissions) -> bool {
        self.permissions.contains(permission)
    }

    /// Checks whether the role has all of the given permissions.
    ///
    /// The 'precise' argument is used to check if the role's permissions are
    /// precisely equivalent to the given permissions. If you need only check
    /// that the role has at least the given permissions, pass `false`.
    pub fn has_permissions(&self, permissions: Permissions, precise: bool)
        -> bool {
        if precise {
            self.permissions == permissions
        } else {
            self.permissions.contains(permissions)
        }
    }
}

impl fmt::Display for Role {
    /// Format a mention for the role, pinging its members.
    // This is in the format of: `<@&ROLE_ID>`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}

impl Eq for Role {}

impl Ord for Role {
    fn cmp(&self, other: &Role) -> Ordering {
        if self.position == other.position {
            self.id.cmp(&other.id)
        } else {
            self.position.cmp(&other.position)
        }
    }
}

impl PartialEq for Role {
    fn eq(&self, other: &Role) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Role {
    fn partial_cmp(&self, other: &Role) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RoleId {
    /// Search the cache for the role.
    #[cfg(feature="cache")]
    pub fn find(&self) -> Option<Role> {
        let cache = CACHE.read().unwrap();

        for guild in cache.guilds.values() {
            let guild = guild.read().unwrap();

            if !guild.roles.contains_key(self) {
                continue;
            }

            if let Some(role) = guild.roles.get(self) {
                return Some(role.clone());
            }
        }

        None
    }
}

impl fmt::Display for RoleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Role> for RoleId {
    /// Gets the Id of a role.
    fn from(role: Role) -> RoleId {
        role.id
    }
}
