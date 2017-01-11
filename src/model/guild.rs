use std::cmp::Ordering;
use std::collections::HashMap;
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
use ::internal::prelude::*;
use ::utils::decode_array;

#[cfg(feature="methods")]
use serde_json::builder::ObjectBuilder;
#[cfg(all(feature="cache", feature = "methods"))]
use std::mem;
#[cfg(all(feature="cache", feature="methods"))]
use ::utils::builder::EditMember;
#[cfg(feature="methods")]
use ::utils::builder::{EditGuild, EditRole, Search};
#[cfg(feature = "methods")]
use ::client::rest;

#[cfg(all(feature="cache", feature="methods"))]
use ::client::CACHE;
#[cfg(all(feature="cache", feature="methods"))]
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
    #[cfg(all(feature="cache", feature="methods"))]
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
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn edit(&mut self, name: &str) -> Result<()> {
        match self.find_guild_id() {
            Some(guild_id) => {
                let map = ObjectBuilder::new()
                    .insert("name", name)
                    .build();

                match rest::edit_emoji(guild_id.0, self.id.0, map) {
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
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn find_guild_id(&self) -> Option<GuildId> {
        CACHE.read()
            .unwrap()
            .guilds
            .values()
            .find(|guild| guild.emojis.contains_key(&self.id))
            .map(|guild| guild.id)
    }

    /// Generates a URL to the emoji's image.
    #[cfg(feature="methods")]
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

impl GuildInfo {
    /// Returns the formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }
}

impl InviteGuild {
    /// Returns the formatted URL of the guild's splash image, if one exists.
    #[cfg(feature="methods")]
    pub fn splash_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
    }
}

impl PartialGuild {
    /// Edits the current user's nickname for the guild.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// **Note**: Requires the [Change Nickname] permission.
    ///
    /// [Change Nickname]: permissions/constant.CHANGE_NICKNAME.html
    #[cfg(feature="methods")]
    #[inline]
    pub fn edit_nickname(&self, new_nickname: Option<&str>) -> Result<()> {
        rest::edit_nickname(self.id.0, new_nickname)
    }

    /// Finds a role by Id within the guild.
    #[cfg(feature="methods")]
    pub fn find_role<R: Into<RoleId>>(&self, role_id: R) -> Option<&Role> {
        self.roles.get(&role_id.into())
    }

    /// Returns a formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
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
    #[cfg(feature="methods")]
    pub fn search<F>(&self, f: F) -> Result<SearchResult>
        where F: FnOnce(Search) -> Search {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        rest::search_guild_messages(self.id.0, &[], f(Search::default()).0)
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
    #[cfg(feature="methods")]
    pub fn search_channels<F>(&self, channel_ids: &[ChannelId], f: F)
        -> Result<SearchResult> where F: FnOnce(Search) -> Search {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        let ids = channel_ids.iter().map(|x| x.0).collect::<Vec<u64>>();

        rest::search_guild_messages(self.id.0, &ids, f(Search::default()).0)
    }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    #[cfg(feature="methods")]
    pub fn splash_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[cfg(feature="methods")]
    #[inline]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> {
        rest::get_guild_webhooks(self.id.0)
    }
}

impl Guild {
    #[cfg(all(feature="cache", feature="methods"))]
    fn has_perms(&self, mut permissions: Permissions) -> Result<bool> {
        let member = match self.get_member(CACHE.read().unwrap().user.id) {
            Some(member) => member,
            None => return Err(Error::Client(ClientError::ItemMissing)),
        };

        let perms = self.permissions_for(ChannelId(self.id.0), member.user.id);
        permissions.remove(perms);

        Ok(permissions.is_empty())
    }

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
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// Returns a [`ClientError::DeleteMessageDaysAmount`] if the number of
    /// days' worth of messages to delete is over the maximum.
    ///
    /// [`ClientError::DeleteMessageDaysAmount`]: ../client/enum.ClientError.html#variant.DeleteMessageDaysAmount
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[cfg(feature="methods")]
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

        rest::ban_user(self.id.0, user.into().0, delete_message_days)
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
    #[cfg(feature="methods")]
    pub fn bans(&self) -> Result<Vec<Ban>> {
        #[cfg(feature="cache")]
        {
            let req = permissions::BAN_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::get_bans(self.id.0)
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
    /// [Manage Channels]: permissions/constants.MANAGE_CHANNELS.html
    #[cfg(feature="methods")]
    pub fn create_channel(&mut self, name: &str, kind: ChannelType)
        -> Result<Channel> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_CHANNELS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        let map = ObjectBuilder::new()
            .insert("name", name)
            .insert("type", kind.name())
            .build();

        rest::create_channel(self.id.0, map)
    }

    /// Creates a new [`Role`] in the guild with the data set, if any.
    ///
    /// See the documentation for [`Context::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Context::create_role`]: ../client/struct.Context.html#method.create_role
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constants.MANAGE_ROLES.html
    #[cfg(feature="methods")]
    pub fn create_role<F>(&self, f: F) -> Result<Role>
        where F: FnOnce(EditRole) -> EditRole {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_ROLES;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        let role = rest::create_role(self.id.0)?;
        let map = f(EditRole::new(&role)).0.build();

        rest::edit_role(self.id.0, role.id.0, map)
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Guild> {
        let mut map = into_map(value)?;

        let id = remove(&mut map, "id").and_then(GuildId::decode)?;

        let public_channels = {
            let mut public_channels = HashMap::new();

            let vals = decode_array(remove(&mut map, "channels")?,
                |v| GuildChannel::decode_guild(v, id))?;

            for public_channel in vals {
                public_channels.insert(public_channel.id, public_channel);
            }

            public_channels
        };

        Ok(Guild {
            afk_channel_id: opt(&mut map, "afk_channel_id", ChannelId::decode)?,
            afk_timeout: req!(remove(&mut map, "afk_timeout")?.as_u64()),
            channels: public_channels,
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


    /// Deletes the current guild if the current account is the owner of the
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
    #[cfg(feature="methods")]
    pub fn delete(&self) -> Result<PartialGuild> {
        #[cfg(feature="cache")]
        {
            if self.owner_id != CACHE.read().unwrap().user.id {
                let req = permissions::MANAGE_GUILD;

                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::delete_guild(self.id.0)
    }

    /// Edits the current guild with new data where specified. See the
    /// documentation for [`Context::edit_guild`] on how to use this.
    ///
    /// **Note**: Requires the current user to have the [Manage Guild]
    /// permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Context::edit_guild`]: ../client/struct.Context.html#method.edit_guild
    /// [Manage Guild]: permissions/constants.MANAGE_GUILD.html
    #[cfg(feature="methods")]
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditGuild) -> EditGuild {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_GUILD;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        let map = f(EditGuild::default()).0.build();

        match rest::edit_guild(self.id.0, map) {
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
    #[cfg(feature="methods")]
    pub fn edit_nickname(&self, new_nickname: Option<&str>) -> Result<()> {
        #[cfg(feature="cache")]
        {
            let req = permissions::CHANGE_NICKNAME;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::edit_nickname(self.id.0, new_nickname)
    }

    /// Attempts to retrieve a [`GuildChannel`] with the given Id.
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    pub fn get_channel<C: Into<ChannelId>>(&self, channel_id: C)
        -> Option<&GuildChannel> {
        self.channels.get(&channel_id.into())
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
    #[cfg(feature="methods")]
    pub fn get_invites(&self) -> Result<Vec<RichInvite>> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_GUILD;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::get_guild_invites(self.id.0)
    }

    /// Attempts to retrieve the given user's member instance in the guild.
    pub fn get_member<U: Into<UserId>>(&self, user_id: U) -> Option<&Member> {
        self.members.get(&user_id.into())
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
            .iter()
            .find(|&(_member_id, member)| {
                let name_matches = member.user.name == name;
                let discrim_matches = match discrim {
                    Some(discrim) => member.user.discriminator == discrim,
                    None => true,
                };

                name_matches && discrim_matches
            }).or(self.members.iter().find(|&(_member_id, member)| {
                member.nick.as_ref().map_or(false, |nick| nick == name)
            })).map(|(_member_id, member)| member)
    }

    /// Returns the formatted URL of the guild's icon, if one exists.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }

    /// Checks if the guild is 'large'. A guild is considered large if it has
    /// more than 250 members.
    pub fn is_large(&self) -> bool {
        self.members.len() > 250
    }

    /// Leaves the guild.
    #[cfg(feature="methods")]
    pub fn leave(&self) -> Result<PartialGuild> {
        rest::leave_guild(self.id.0)
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
                      member.user.id,
                      self.id,
                      role);
            }
        }

        // Administrators have all permissions in any channel.
        if permissions.contains(ADMINISTRATOR) {
            return Permissions::all();
        }

        if let Some(channel) = self.channels.get(&channel_id) {
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
    #[cfg(feature="methods")]
    pub fn prune_count(&self, days: u16) -> Result<GuildPrune> {
        #[cfg(feature="cache")]
        {
            let req = permissions::KICK_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        let map = ObjectBuilder::new()
            .insert("days", days)
            .build();

        rest::get_guild_prune_count(self.id.0, map)
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
    #[cfg(feature="methods")]
    pub fn search<F>(&self, f: F) -> Result<SearchResult>
        where F: FnOnce(Search) -> Search {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        rest::search_guild_messages(self.id.0, &[], f(Search::default()).0)
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
    #[cfg(feature = "methods")]
    pub fn search_channels<F>(&self, channel_ids: &[ChannelId], f: F)
        -> Result<SearchResult> where F: FnOnce(Search) -> Search {
        #[cfg(feature="cache")]
        {
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }

        let ids = channel_ids.iter().map(|x| x.0).collect::<Vec<u64>>();

        rest::search_guild_messages(self.id.0, &ids, f(Search::default()).0)
    }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    #[cfg(feature="methods")]
    pub fn splash_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
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
    #[cfg(feature="methods")]
    pub fn start_prune(&self, days: u16) -> Result<GuildPrune> {
        #[cfg(feature="cache")]
        {
            let req = permissions::KICK_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        let map = ObjectBuilder::new()
            .insert("days", days)
            .build();

        rest::start_guild_prune(self.id.0, map)
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
    #[cfg(feature="methods")]
    pub fn unban<U: Into<UserId>>(&self, user: U) -> Result<()> {
        #[cfg(feature="cache")]
        {
            let req = permissions::BAN_MEMBERS;

            if !self.has_perms(req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::remove_ban(self.id.0, user.into().0)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[cfg(feature="methods")]
    #[inline]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> {
        rest::get_guild_webhooks(self.id.0)
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
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn add_role<R: Into<RoleId>>(&mut self, role_id: R) -> Result<()> {
        let role_id = role_id.into();

        if self.roles.contains(&role_id) {
            return Ok(());
        }

        let guild_id = self.find_guild()?;

        match rest::add_member_role(guild_id.0, self.user.id.0, role_id.0) {
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
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn add_roles(&mut self, role_ids: &[RoleId]) -> Result<()> {
        let guild_id = self.find_guild()?;
        self.roles.extend_from_slice(role_ids);

        let map = EditMember::default().roles(&self.roles).0.build();

        match rest::edit_member(guild_id.0, self.user.id.0, map) {
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
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn ban(&self, delete_message_days: u8) -> Result<()> {
        let guild_id = self.find_guild()?;

        rest::ban_user(guild_id.0,
                       self.user.id.0,
                       delete_message_days)
    }

    /// Determines the member's colour.
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn colour(&self) -> Option<Colour> {
        let default = Colour::default();
        let guild_id = match self.find_guild() {
            Ok(guild_id) => guild_id,
            Err(_why) => return None,
        };

        let cache = CACHE.read().unwrap();
        let guild = match cache.guilds.get(&guild_id) {
            Some(guild) => guild,
            None => return None,
        };

        let mut roles = self.roles
            .iter()
            .filter_map(|id| guild.roles.get(id))
            .collect::<Vec<&Role>>();
        roles.sort_by(|a, b| b.cmp(a));

        roles.iter().find(|r| r.colour.value != default.value).map(|r| r.colour)
    }

    /// Calculates the member's display name.
    ///
    /// The nickname takes priority over the member's username if it exists.
    pub fn display_name(&self) -> &str {
        self.nick.as_ref().unwrap_or(&self.user.name)
    }

    /// Returns the DiscordTag of a Member, taking possible nickname into account.
    #[cfg(feature="methods")]
    pub fn distinct(&self) -> String {
        format!("{}#{}", self.display_name(), self.discriminator)
    }

    /// Edits the member with the given data. See [`Context::edit_member`] for
    /// more information.
    ///
    /// See [`EditMember`] for the permission(s) required for separate builder
    /// methods, as well as usage of this.
    ///
    /// [`Context::edit_member`]: ../client/struct.Context.html#method.edit_member
    /// [`EditMember`]: ../builder/struct.EditMember.html
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn edit<F>(&self, f: F) -> Result<()>
        where F: FnOnce(EditMember) -> EditMember {
        let guild_id = self.find_guild()?;
        let map = f(EditMember::default()).0.build();

        rest::edit_member(guild_id.0, self.user.id.0, map)
    }

    /// Finds the Id of the [`Guild`] that the member is in.
    ///
    /// [`Guild`]: struct.Guild.html
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn find_guild(&self) -> Result<GuildId> {
        CACHE.read()
            .unwrap()
            .guilds
            .values()
            .find(|guild| {
                guild.members
                    .iter()
                    .any(|(_member_id, member)| {
                        let joined_at = member.joined_at == self.joined_at;
                        let user_id = member.user.id == self.user.id;

                        joined_at && user_id
                    })
            })
            .map(|x| x.id)
            .ok_or(Error::Client(ClientError::GuildNotFound))
    }

    /// Removes a [`Role`] from the member, editing its roles in-place if the
    /// request was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn remove_role<R: Into<RoleId>>(&mut self, role_id: R) -> Result<()> {
        let role_id = role_id.into();

        if !self.roles.contains(&role_id) {
            return Ok(());
        }

        let guild_id = self.find_guild()?;

        match rest::remove_member_role(guild_id.0, self.user.id.0, role_id.0) {
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
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn remove_roles(&mut self, role_ids: &[RoleId]) -> Result<()> {
        let guild_id = self.find_guild()?;
        self.roles.retain(|r| !role_ids.contains(r));

        let map = EditMember::default().roles(&self.roles).0.build();

        match rest::edit_member(guild_id.0, self.user.id.0, map) {
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
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn roles(&self) -> Option<Vec<Role>> {
        CACHE.read().unwrap()
            .guilds
            .values()
            .find(|g| g.members
                .values()
                .any(|m| m.user.id == self.user.id && m.joined_at == *self.joined_at))
            .map(|g| g.roles
                .values()
                .filter(|r| self.roles.contains(&r.id))
                .cloned()
                .collect())
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
        fmt::Display::fmt(&self.user.mention(), f)
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
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn delete(&self) -> Result<()> {
        let guild_id = self.find_guild()?;

        rest::delete_role(guild_id.0, self.id.0)
    }

    /// Searches the cache for the guild that owns the role.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::GuildNotFound`] if a guild is not in the cache
    /// that contains the role.
    ///
    /// [`ClientError::GuildNotFound`]: ../client/enum.ClientError.html#variant.GuildNotFound
    #[cfg(all(feature="cache", feature="methods"))]
    pub fn find_guild(&self) -> Result<GuildId> {
        CACHE.read()
            .unwrap()
            .guilds
            .values()
            .find(|guild| guild.roles.contains_key(&RoleId(self.id.0)))
            .map(|x| x.id)
            .ok_or(Error::Client(ClientError::GuildNotFound))
    }

    /// Check that the role has the given permission.
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
