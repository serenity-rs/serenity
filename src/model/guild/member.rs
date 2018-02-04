use chrono::{DateTime, FixedOffset};
use futures::future;
use model::prelude::*;
use std::cell::RefCell;
use super::super::WrappedClient;
use super::deserialize_user;
use ::FutureResult;

#[cfg(all(feature = "builder", feature = "cache", feature = "model"))]
use builder::EditMember;
#[cfg(all(feature = "cache", feature = "model"))]
use internal::prelude::*;
#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use utils::Colour;

/// A trait for allowing both u8 or &str or (u8, &str) to be passed into the `ban` methods in `Guild` and `Member`.
pub trait BanOptions {
    fn dmd(&self) -> u8 { 0 }
    fn reason(&self) -> &str { "" }
}

impl BanOptions for u8 {
    fn dmd(&self) -> u8 { *self }
}

impl BanOptions for str {
    fn reason(&self) -> &str { self }
}

impl<'a> BanOptions for &'a str {
    fn reason(&self) -> &str { self }
}

impl BanOptions for String {
    fn reason(&self) -> &str { self }
}

impl<'a> BanOptions for (u8, &'a str) {
    fn dmd(&self) -> u8 { self.0 }

    fn reason(&self) -> &str { self.1 }
}

impl BanOptions for (u8, String) {
    fn dmd(&self) -> u8 { self.0 }

    fn reason(&self) -> &str { &self.1 }
}

/// Information about a member of a guild.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Member {
    /// Indicator of whether the member can hear in voice channels.
    pub deaf: bool,
    /// The unique Id of the guild that the member is a part of.
    pub guild_id: GuildId,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<DateTime<FixedOffset>>,
    /// Indicator of whether the member can speak in voice channels.
    pub mute: bool,
    /// The member's nickname, if present.
    ///
    /// Can't be longer than 32 characters.
    pub nick: Option<String>,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: Vec<RoleId>,
    /// Attached User struct.
    #[serde(deserialize_with = "deserialize_user",
            serialize_with = "serialize_user")]
    pub user: Rc<RefCell<User>>,
    #[serde(skip)]
    pub(crate) client: WrappedClient,
}

#[cfg(feature = "model")]
impl Member {
    /// Adds a [`Role`] to the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature = "cache")]
    pub fn add_role<R: Into<RoleId>>(&mut self, role_id: R)
        -> FutureResult<()> {
        let role_id = role_id.into();

        if self.roles.contains(&role_id) {
            return Box::new(future::ok(()));
        }

        ftryopt!(self.client).http.add_member_role(
            self.guild_id.0,
            self.user.borrow().id.0,
            role_id.0,
        )
    }

    /// Adds one or multiple [`Role`]s to the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature = "cache")]
    pub fn add_roles(&mut self, role_ids: &[RoleId]) -> FutureResult<()> {
        let mut roles = self.roles.clone();
        roles.extend(role_ids);

        ftryopt!(self.client).http.edit_member(
            self.guild_id.0,
            self.user.borrow().id.0,
            |f| f.roles(roles),
        )
    }

    /// Ban the member from its guild, deleting the last X number of
    /// days' worth of messages.
    ///
    /// **Note**: Requires the [Ban Members] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the guild could not be
    /// found.
    ///
    /// [`ModelError::GuildNotFound`]: enum.ModelError.html#variant.GuildNotFound
    ///
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[cfg(feature = "cache")]
    pub fn ban<BO: BanOptions>(&self, ban_options: &BO) -> FutureResult<()> {
        let dmd = ban_options.dmd();

        if dmd > 7 {
            return Box::new(future::err(Error::Model(
                ModelError::DeleteMessageDaysAmount(dmd),
            )));
        }

        let reason = ban_options.reason();

        if reason.len() > 512 {
            return Box::new(future::err(Error::ExceededLimit(
                reason.to_string(),
                512,
            )));
        }

        ftryopt!(self.client).http.ban_user(
            self.guild_id.0,
            self.user.borrow().id.0,
            dmd,
            &*reason,
        )
    }

    /// Determines the member's colour.
    #[cfg(all(feature = "cache", feature = "utils"))]
    pub fn colour(&self) -> Option<Colour> {
        let client = self.client.as_ref()?;
        let cache = client.cache.try_borrow().ok()?;
        let guild = cache.guild(self.guild_id)?;
        let guild = guild.borrow();

        let mut roles = self.roles
            .iter()
            .filter_map(|role_id| guild.roles.get(role_id))
            .collect::<Vec<&Rc<RefCell<Role>>>>();
        roles.sort_by(|a, b| b.cmp(a));

        let default = Colour::default();

        roles
            .iter()
            .find(|r| r.borrow().colour.0 != default.0)
            .map(|r| r.borrow().colour)
    }

    /// Returns the "default channel" of the guild for the member.
    /// (This returns the first channel that can be read by the member, if there isn't
    /// one returns `None`)
    #[cfg(feature = "cache")]
    pub fn default_channel(&self) -> Option<Rc<RefCell<GuildChannel>>> {
        let cache = self.client.as_ref()?.cache.borrow();
        let guild = cache.guild(self.guild_id)?;
        let reader = guild.borrow();

        for (cid, channel) in &reader.channels {
            if reader.permissions_in(*cid, self.user.borrow().id).read_messages() {
                return Some(Rc::clone(channel));
            }
        }

        None
    }

    /// Calculates the member's display name.
    ///
    /// The nickname takes priority over the member's username if it exists.
    #[inline]
    pub fn display_name(&self) -> Cow<String> {
        self.nick
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| {
                Cow::Owned(unsafe { (*self.user.as_ptr()).name.clone() })
            })
    }

    /// Returns the DiscordTag of a Member, taking possible nickname into account.
    #[inline]
    pub fn distinct(&self) -> String {
        unsafe {
            let user = &*self.user.as_ptr();

            format!(
                "{}#{}",
                self.display_name(),
                user.discriminator,
            )
        }
    }

    /// Edits the member with the given data. See [`Guild::edit_member`] for
    /// more information.
    ///
    /// See [`EditMember`] for the permission(s) required for separate builder
    /// methods, as well as usage of this.
    ///
    /// [`Guild::edit_member`]: ../model/guild/struct.Guild.html#method.edit_member
    /// [`EditMember`]: ../builder/struct.EditMember.html
    #[cfg(feature = "cache")]
    pub fn edit<F: FnOnce(EditMember) -> EditMember>(&self, f: F)
        -> FutureResult<()> {
        ftryopt!(self.client).http.edit_member(
            self.guild_id.0,
            self.user.borrow().id.0,
            f,
        )
    }

    /// Retrieves the ID and position of the member's highest role in the
    /// hierarchy, if they have one.
    ///
    /// This _may_ return `None` if the user has roles, but they are not present
    /// in the cache for cache inconsistency reasons.
    ///
    /// The "highest role in hierarchy" is defined as the role with the highest
    /// position. If two or more roles have the same highest position, then the
    /// role with the lowest ID is the highest.
    ///
    /// # Deadlocking
    ///
    /// This function will deadlock if you have a write lock to the member's
    /// guild.
    #[cfg(feature = "cache")]
    pub fn highest_role_info(&self) -> Option<(RoleId, i64)> {
        let cache = self.client.as_ref()?.cache.try_borrow().ok()?;
        let guild = cache.guild(self.guild_id)?;
        let reader = guild.borrow();

        let mut highest = None;

        for role_id in &self.roles {
            let role = reader.roles
                .get(&role_id)
                .and_then(|x| x.try_borrow().ok());

            if let Some(role) = role {
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
    /// match member.kick() {
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
    /// [`ModelError::GuildNotFound`]: enum.ModelError.html#variant.GuildNotFound
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [Kick Members]: permissions/constant.KICK_MEMBERS.html
    pub fn kick(&self) -> FutureResult<()> {
        let client = ftryopt!(self.client);

        #[cfg(feature = "cache")]
        {
            let cache = ftry!(client.cache.try_borrow());

            if let Some(guild) = cache.guilds.get(&self.guild_id) {
                let req = Permissions::KICK_MEMBERS;
                let reader = ftry!(guild.try_borrow());

                if !reader.has_perms(req) {
                    return Box::new(future::err(Error::Model(
                        ModelError::InvalidPermissions(req),
                    )));
                }

                ftry!(reader.check_hierarchy(ftry!(self.user.try_borrow()).id));
            }
        }

        let user_id = ftry!(self.user.try_borrow()).id.0;

        ftryopt!(self.client).http.kick_member(self.guild_id.0, user_id)
    }

    /// Returns the guild-level permissions for the member.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // assuming there's a `member` variable gotten from anything.
    /// println!("The permission bits for the member are: {}",
    /// member.permissions().expect("permissions").bits);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::GuildNotFound`] if the guild the member's in could not be
    /// found in the cache.
    ///
    /// And/or returns [`ModelError::ItemMissing`] if the "default channel" of the guild is not
    /// found.
    ///
    /// [`ModelError::GuildNotFound`]: enum.ModelError.html#variant.GuildNotFound
    /// [`ModelError::ItemMissing`]: enum.ModelError.html#variant.ItemMissing
    #[cfg(feature = "cache")]
    pub fn permissions(&self) -> Result<Permissions> {
        let client = self.client.as_ref().ok_or_else(|| {
            Error::Model(ModelError::ClientNotPresent)
        })?;
        let cache = client.cache.try_borrow()?;
        let guild = match cache.guilds.get(&self.guild_id) {
            Some(guild) => guild,
            None => return Err(From::from(ModelError::GuildNotFound)),
        };

        let guild = guild.try_borrow()?;

        Ok(guild.member_permissions(self.user.try_borrow()?.id))
    }

    /// Removes a [`Role`] from the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature = "cache")]
    pub fn remove_role<R: Into<RoleId>>(&mut self, role_id: R) -> FutureResult<()> {
        let role_id = role_id.into();

        if !self.roles.contains(&role_id) {
            return Box::new(future::ok(()));
        }

        ftryopt!(self.client).http.remove_member_role(
            self.guild_id.0,
            self.user.borrow().id.0,
            role_id.0,
        )
    }

    /// Removes one or multiple [`Role`]s from the member.
    ///
    /// **Note**: Requires the [Manage Roles] permission.
    ///
    /// [`Role`]: struct.Role.html
    /// [Manage Roles]: permissions/constant.MANAGE_ROLES.html
    #[cfg(feature = "cache")]
    pub fn remove_roles(&mut self, role_ids: &[RoleId]) -> FutureResult<()> {
        let mut roles = self.roles.clone();
        roles.retain(|r| !role_ids.contains(r));

        ftryopt!(self.client).http.edit_member(
            self.guild_id.0,
            self.user.borrow().id.0,
            |f| f.roles(roles),
        )
    }

    /// Retrieves the full role data for the user's roles.
    ///
    /// This is shorthand for manually searching through the CACHE.
    ///
    /// If role data can not be found for the member, then `None` is returned.
    #[cfg(feature = "cache")]
    pub fn roles(&self) -> Option<Vec<Rc<RefCell<Role>>>> {
        let client = self.client.as_ref()?;
        let cache = client.cache.try_borrow().ok()?;

        cache.guilds.get(&self.guild_id).and_then(|guild| {
            let guild = guild.try_borrow().ok()?;

            let roles = guild
                .roles
                .values()
                .filter(|role| {
                    let role = match role.try_borrow() {
                        Ok(role) => role,
                        Err(_) => return false,
                    };

                    self.roles.contains(&role.id)
                })
                .cloned()
                .collect();

            Some(roles)
        })
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
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`User`]: struct.User.html
    /// [Ban Members]: permissions/constant.BAN_MEMBERS.html
    #[cfg(feature = "cache")]
    pub fn unban(&self) -> FutureResult<()> {
        ftryopt!(self.client).http.remove_ban(
            self.guild_id.0,
            self.user.borrow().id.0,
        )
    }
}
