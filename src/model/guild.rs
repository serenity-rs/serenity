use serde_json::builder::ObjectBuilder;
use std::collections::HashMap;
use std::{fmt, mem};
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
    warn_field
};
use super::*;
use ::utils::builder::{EditGuild, EditMember, EditRole};
use ::client::{STATE, http};
use ::internal::prelude::*;
use ::utils::{Colour, decode_array};

impl From<Guild> for GuildContainer {
    fn from(guild: Guild) -> GuildContainer {
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
    /// Finds the [`Guild`] that owns the emoji by looking through the state.
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn find_guild_id(&self) -> Option<GuildId> {
        STATE.lock()
            .unwrap()
            .guilds
            .values()
            .find(|guild| guild.emojis.contains_key(&self.id))
            .map(|guild| guild.id)
    }

    /// Deletes the emoji.
    ///
    /// **Note**: The [Manage Emojis] permission is required.
    ///
    /// **Note**: Only user accounts may use this method.
    ///
    /// [Manage Emojis]: permissions/constant.MANAGE_EMOJIS.html
    pub fn delete(&self) -> Result<()> {
        match self.find_guild_id() {
            Some(guild_id) => http::delete_emoji(guild_id.0, self.id.0),
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
    pub fn edit(&mut self, name: &str) -> Result<()> {
        match self.find_guild_id() {
            Some(guild_id) => {
                let map = ObjectBuilder::new()
                    .insert("name", name)
                    .build();

                match http::edit_emoji(guild_id.0, self.id.0, map) {
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
}

impl fmt::Display for Emoji {
    /// Formats the emoji into a string that will cause Discord clients to
    /// render the emoji.
    ///
    /// This is in the format of: `<:NAME:EMOJI_ID>`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(f.write_str("<:"));
        try!(f.write_str(&self.name));
        try!(fmt::Write::write_char(f, ':'));
        try!(fmt::Display::fmt(&self.id, f));
        fmt::Write::write_char(f, '>')
    }
}

impl GuildInfo {
    /// Returns the formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn_concat!("/icons/{}/{}.jpg"), self.id, icon))
    }
}

impl Guild {
    /// Finds a role by Id within the guild.
    pub fn find_role<R: Into<RoleId>>(&self, role_id: R) -> Option<&Role> {
        self.roles.get(&role_id.into())
    }

    /// Edits the current user's nickname for the guild.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// **Note**: Requires the [Change Nickname] permission.
    ///
    /// [Change Nickname]: permissions/constant.CHANGE_NICKNAME.html
    #[inline]
    pub fn edit_nickname(&self, new_nickname: Option<&str>) -> Result<()> {
        http::edit_nickname(self.id.0, new_nickname)
    }

    /// Returns a formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|icon|
            format!(cdn_concat!("/icons/{}/{}.jpg"), self.id, icon))
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> {
        http::get_guild_webhooks(self.id.0)
    }
}

impl LiveGuild {
    fn has_perms(&self, mut permissions: Permissions) -> Result<bool> {
        let member = match self.get_member(STATE.lock().unwrap().user.id) {
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
    /// Ban a member for 4 days:
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
    pub fn ban<U: Into<UserId>>(&self, user: U, delete_message_days: u8)
        -> Result<()> {
        if delete_message_days > 7 {
            return Err(Error::Client(ClientError::DeleteMessageDaysAmount(delete_message_days)));
        }

        let req = permissions::BAN_MEMBERS;

        if !try!(self.has_perms(req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::ban_user(self.id.0, user.into().0, delete_message_days)
    }

    /// Retrieves a list of [`Ban`]s for the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// [`Ban`]: struct.Ban.html
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    pub fn bans(&self) -> Result<Vec<Ban>> {
        let req = permissions::BAN_MEMBERS;

        if !try!(self.has_perms(req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::get_bans(self.id.0)
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
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// [`Channel`]: struct.Channel.html
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Channels]: permissions/constants.MANAGE_CHANNELS.html
    pub fn create_channel(&mut self, name: &str, kind: ChannelType)
        -> Result<Channel> {
        let req = permissions::MANAGE_CHANNELS;

        if !try!(self.has_perms(req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        let map = ObjectBuilder::new()
            .insert("name", name)
            .insert("type", kind.name())
            .build();

        http::create_channel(self.id.0, map)
    }

    /// Creates a new [`Role`] in the guild with the data set, if any.
    ///
    /// See the documentation for [`Context::create_role`] on how to use this.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Context::create_role`]: ../client/struct.Context.html#method.create_role
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constants.MANAGE_ROLES.html
    pub fn create_role<F>(&self, f: F) -> Result<Role>
        where F: FnOnce(EditRole) -> EditRole {
        let req = permissions::MANAGE_ROLES;

        if !try!(self.has_perms(req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        let role = {
            try!(http::create_role(self.id.0))
        };
        let map = f(EditRole::default()).0.build();

        http::edit_role(self.id.0, role.id.0, map)
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<LiveGuild> {
        let mut map = try!(into_map(value));

        let id = try!(remove(&mut map, "id").and_then(GuildId::decode));

        let public_channels = {
            let mut public_channels = HashMap::new();

            let vals = try!(decode_array(try!(remove(&mut map, "channels")),
                |v| PublicChannel::decode_guild(v, id)));

            for public_channel in vals {
                public_channels.insert(public_channel.id, public_channel);
            }

            public_channels
        };

        missing!(map, LiveGuild {
            afk_channel_id: try!(opt(&mut map, "afk_channel_id", ChannelId::decode)),
            afk_timeout: req!(try!(remove(&mut map, "afk_timeout")).as_u64()),
            channels: public_channels,
            default_message_notifications: req!(try!(remove(&mut map, "default_message_notifications")).as_u64()),
            emojis: try!(remove(&mut map, "emojis").and_then(decode_emojis)),
            features: try!(remove(&mut map, "features").and_then(|v| decode_array(v, Feature::decode_str))),
            icon: try!(opt(&mut map, "icon", into_string)),
            id: id,
            joined_at: try!(remove(&mut map, "joined_at").and_then(into_string)),
            large: req!(try!(remove(&mut map, "large")).as_bool()),
            member_count: req!(try!(remove(&mut map, "member_count")).as_u64()),
            members: try!(remove(&mut map, "members").and_then(decode_members)),
            mfa_level: req!(try!(remove(&mut map, "mfa_level")).as_u64()),
            name: try!(remove(&mut map, "name").and_then(into_string)),
            owner_id: try!(remove(&mut map, "owner_id").and_then(UserId::decode)),
            presences: try!(remove(&mut map, "presences").and_then(decode_presences)),
            region: try!(remove(&mut map, "region").and_then(into_string)),
            roles: try!(remove(&mut map, "roles").and_then(decode_roles)),
            splash: try!(opt(&mut map, "splash", into_string)),
            verification_level: try!(remove(&mut map, "verification_level").and_then(VerificationLevel::decode)),
            voice_states: try!(remove(&mut map, "voice_states").and_then(decode_voice_states)),
        })
    }


    /// Deletes the current guild if the current account is the owner of the
    /// guild.
    ///
    /// **Note**: Requires the current user to be the owner of the guild.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidUser`] if the current user is not the
    /// guild owner.
    ///
    /// [`ClientError::InvalidUser`]: ../client/enum.ClientError.html#variant.InvalidUser
    pub fn delete(&self) -> Result<Guild> {
        if self.owner_id != STATE.lock().unwrap().user.id {
            let req = permissions::MANAGE_GUILD;

            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::delete_guild(self.id.0)
    }

    /// Edits the current guild with new data where specified. See the
    /// documentation for [`Context::edit_guild`] on how to use this.
    ///
    /// **Note**: Requires the current user to have the [Manage Guild]
    /// permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Context::edit_guild`]: ../client/struct.Context.html#method.edit_guild
    /// [Manage Guild]: permissions/constants.MANAGE_GUILD.html
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditGuild) -> EditGuild {
        let req = permissions::MANAGE_GUILD;

        if !try!(self.has_perms(req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        let map = f(EditGuild::default()).0.build();

        match http::edit_guild(self.id.0, map) {
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
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to change their own nickname.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Change Nickname]: permissions/constant.CHANGE_NICKNAME.html
    pub fn edit_nickname(&self, new_nickname: Option<&str>) -> Result<()> {
        let req = permissions::CHANGE_NICKNAME;

        if !try!(self.has_perms(req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::edit_nickname(self.id.0, new_nickname)
    }

    /// Attempts to retrieve a [`PublicChannel`] with the given Id.
    ///
    /// [`PublicChannel`]: struct.PublicChannel.html
    pub fn get_channel<C: Into<ChannelId>>(&self, channel_id: C)
        -> Option<&PublicChannel> {
        self.channels.get(&channel_id.into())
    }

    /// Retrieves the active invites for the guild.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn get_invites(&self) -> Result<Vec<RichInvite>> {
        let req = permissions::MANAGE_GUILD;

        if !try!(self.has_perms(req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::get_guild_invites(self.id.0)
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

            let discrim = match split.1.parse::<u16>() {
                Ok(discrim) => discrim,
                Err(_why) => return None,
            };

            (split.0, Some(discrim))
        } else {
            (&name[..], None::<u16>)
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
            format!(cdn_concat!("/icons/{}/{}.jpg"), self.id, icon))
    }

    /// Checks if the guild is 'large'. A guild is considered large if it has
    /// more than 250 members.
    pub fn is_large(&self) -> bool {
        self.members.len() > 250
    }

    /// Leaves the guild.
    pub fn leave(&self) -> Result<Guild> {
        http::leave_guild(self.id.0)
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
                error!("@everyone role ({}) missing in {}", self.id, self.name);

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
                warn!("perms: {:?} on {:?} has non-existent role {:?}", member.user.id, self.id, role);
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
            warn!("Guild {} does not contain channel {}", self.id, channel_id);
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
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`GuildPrune`]: struct.GuildPrune.html
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    pub fn prune_count(&self, days: u16) -> Result<GuildPrune> {
        let req = permissions::KICK_MEMBERS;

        if !try!(self.has_perms(req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        let map = ObjectBuilder::new()
            .insert("days", days)
            .build();

        http::get_guild_prune_count(self.id.0, map)
    }

    /// Starts a prune of [`Member`]s.
    ///
    /// See the documentation on [`GuildPrune`] for more information.
    ///
    /// **Note**: Requires the [Kick Members] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`GuildPrune`]: struct.GuildPrune.html
    /// [`Member`]: struct.Member.html
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    pub fn start_prune(&self, days: u16) -> Result<GuildPrune> {
        let req = permissions::KICK_MEMBERS;

        if !try!(self.has_perms(req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        let map = ObjectBuilder::new()
            .insert("days", days)
            .build();

        http::start_guild_prune(self.id.0, map)
    }

    /// Unbans the given [`User`] from the guild.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have permission to perform bans.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    pub fn unban<U: Into<UserId>>(&self, user: U) -> Result<()> {
        let req = permissions::BAN_MEMBERS;

        if !try!(self.has_perms(req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::remove_ban(self.id.0, user.into().0)
    }

    /// Retrieves the guild's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> {
        http::get_guild_webhooks(self.id.0)
    }
}

impl Member {
    /// Adds a [`Role`] to the member, editing its roles
    /// in-place if the request was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    pub fn add_role<R: Into<RoleId>>(&mut self, role_id: R) -> Result<()> {
        self.add_roles(&[role_id.into()])
    }

    /// Adds one or multiple [`Role`]s to the member, editing
    /// its roles in-place if the request was successful.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    pub fn add_roles(&mut self, role_ids: &[RoleId]) -> Result<()> {
        let guild_id = try!(self.find_guild());
        self.roles.extend_from_slice(role_ids);

        let map = EditMember::default().roles(&self.roles).0.build();

        match http::edit_member(guild_id.0, self.user.id.0, map) {
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
    pub fn ban(&self, delete_message_days: u8) -> Result<()> {
        let guild_id = try!(self.find_guild());

        http::ban_user(guild_id.0,
                       self.user.id.0,
                       delete_message_days)
    }

    /// Calculates the member's display name.
    ///
    /// The nickname takes priority over the member's username if it exists.
    pub fn display_name(&self) -> &str {
        self.nick.as_ref().unwrap_or(&self.user.name)
    }

    /// Edits the member with the given data. See [`Context::edit_member`] for
    /// more information.
    ///
    /// See [`EditMember`] for the permission(s) required for separate builder
    /// methods, as well as usage of this.
    ///
    /// [`Context::edit_member`]: ../client/struct.Context.html#method.edit_member
    /// [`EditMember`]: ../builder/struct.EditMember.html
    pub fn edit<F>(&self, f: F) -> Result<()>
        where F: FnOnce(EditMember) -> EditMember {
        let guild_id = try!(self.find_guild());
        let map = f(EditMember::default()).0.build();

        http::edit_member(guild_id.0, self.user.id.0, map)
    }

    /// Finds the Id of the [`Guild`] that the member is in.
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn find_guild(&self) -> Result<GuildId> {
        STATE.lock()
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

    /// Removes a [`Role`] from the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    pub fn remove_role<R: Into<RoleId>>(&mut self, role_id: R) -> Result<()> {
        self.remove_roles(&[role_id.into()])
    }

    /// Removes one or multiple [`Role`]s from the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    pub fn remove_roles(&mut self, role_ids: &[RoleId]) -> Result<()> {
        let guild_id = try!(self.find_guild());
        self.roles.retain(|r| !role_ids.contains(r));

        let map = EditMember::default().roles(&self.roles).0.build();

        match http::edit_member(guild_id.0, self.user.id.0, map) {
            Ok(()) => Ok(()),
            Err(why) => {
                self.roles.extend_from_slice(role_ids);

                Err(why)
            },
        }
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

impl PossibleGuild<LiveGuild> {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Self> {
        let mut value = try!(into_map(value));
        if remove(&mut value, "unavailable").ok().and_then(|v| v.as_bool()).unwrap_or(false) {
            remove(&mut value, "id").and_then(GuildId::decode).map(PossibleGuild::Offline)
        } else {
            LiveGuild::decode(Value::Object(value)).map(PossibleGuild::Online)
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

impl PossibleGuild<Guild> {
    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Self> {
        let mut value = try!(into_map(value));
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
            PossibleGuild::Offline(id) => id,
            PossibleGuild::Online(ref live_guild) => live_guild.id,
        }
    }
}

impl Role {
    /// Generates a colour representation of the role. See
    /// [the documentation] on Colour for more information.
    ///
    /// [the documentation]: ../utils/struct.Colour.html
    pub fn colour(&self) -> Colour {
        Colour::new(self.colour as u32)
    }

    /// Deletes the role.
    ///
    /// **Note** Requires the [Manage Roles] permission.
    ///
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    pub fn delete(&self) -> Result<()> {
        let guild_id = try!(self.find_guild());

        http::delete_role(guild_id.0, self.id.0)
    }

    /// Searches the state for the guild that owns the role.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::GuildNotFound`] if a guild is not in the state
    /// that contains the role.
    ///
    /// [`ClientError::GuildNotFound`]: ../client/enum.ClientError.html#variant.GuildNotFound
    pub fn find_guild(&self) -> Result<GuildId> {
        STATE.lock()
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
    /// precisely equivilant to the given permissions. If you need only check
    /// that the role has at least the given permissions, pass `false`.
    pub fn has_permissions(&self, permissions: Permissions, precise: bool)
        -> bool {
        if precise {
            self.permissions == permissions
        } else {
            self.permissions.contains(permissions)
        }
    }

    /// Return a `Mention` which will ping members of the role.
    pub fn mention(&self) -> Mention {
        self.id.mention()
    }
}

impl fmt::Display for Role {
    /// Format a mention for the role, pinging its members.
    // This is in the format of: `<@&ROLE_ID>`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.mention(), f)
    }
}
