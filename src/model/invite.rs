use super::*;
use ::client::rest;
use ::internal::prelude::*;
use ::utils::builder::CreateInvite;
use ::utils;

#[cfg(feature="cache")]
use super::permissions;
#[cfg(feature="cache")]
use super::utils as model_utils;

/// Information about an invite code.
///
/// Information can not be accessed for guilds the current user is banned from.
#[derive(Clone, Debug, Deserialize)]
pub struct Invite {
    /// The unique code for the invite.
    pub code: String,
    /// A representation of the minimal amount of information needed about the
    /// [`GuildChannel`] being invited to.
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    pub channel: InviteChannel,
    /// a representation of the minimal amount of information needed about the
    /// [`Guild`] being invited to.
    ///
    /// [`Guild`]: struct.Guild.html
    pub guild: InviteGuild,
}

impl Invite {
    /// Creates an invite for a [`GuildChannel`], providing a builder so that
    /// fields may optionally be set.
    ///
    /// See the documentation for the [`CreateInvite`] builder for information
    /// on how to use this and the default values that it provides.
    ///
    /// Requires the [Create Invite] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have the required [permission].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`CreateInvite`]: ../utils/builder/struct.CreateInvite.html
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [Create Invite]: permissions/constant.CREATE_INVITE.html
    /// [permission]: permissions/index.html
    pub fn create<C, F>(channel_id: C, f: F) -> Result<RichInvite>
        where C: Into<ChannelId>, F: FnOnce(CreateInvite) -> CreateInvite {
        let channel_id = channel_id.into();

        #[cfg(feature="cache")]
        {
            let req = permissions::CREATE_INVITE;

            if !model_utils::user_has_perms(channel_id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::create_invite(channel_id.0, &f(CreateInvite::default()).0)
    }

    /// Deletes the invite.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ClientError::InvalidPermissions`]
    /// if the current user does not have the required [permission].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    /// [permission]: permissions/index.html
    pub fn delete(&self) -> Result<Invite> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_GUILD;

            if !model_utils::user_has_perms(self.channel.id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::delete_invite(&self.code)
    }

    /// Gets the information about an invite.
    pub fn get(code: &str) -> Result<Invite> {
        rest::get_invite(utils::parse_invite(code))
    }
}

/// A inimal information about the channel an invite points to.
#[derive(Clone, Debug, Deserialize)]
pub struct InviteChannel {
    pub id: ChannelId,
    pub name: String,
    #[serde(rename="type")]
    pub kind: ChannelType,
}

/// A minimal amount of information about the guild an invite points to.
#[derive(Clone, Debug, Deserialize)]
pub struct InviteGuild {
    pub id: GuildId,
    pub icon: Option<String>,
    pub name: String,
    pub splash_hash: Option<String>,
}

impl InviteGuild {
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
    #[cfg(feature="cache")]
    #[inline]
    pub fn shard_id(&self) -> u64 {
        self.id.shard_id()
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
    #[cfg(not(feature="cache"))]
    #[inline]
    pub fn shard_id(&self, shard_count: u64) -> u64 {
        self.id.shard_id(shard_count)
    }
}

/// Detailed information about an invite.
/// This information can only be retrieved by anyone with the [Manage Guild]
/// permission. Otherwise, a minimal amount of information can be retrieved via
/// the [`Invite`] struct.
///
/// [`Invite`]: struct.Invite.html
/// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
#[derive(Clone, Debug, Deserialize)]
pub struct RichInvite {
    /// A representation of the minimal amount of information needed about the
    /// channel being invited to.
    pub channel: InviteChannel,
    /// The unique code for the invite.
    pub code: String,
    /// When the invite was created.
    pub created_at: String,
    /// A representation of the minimal amount of information needed about the
    /// guild being invited to.
    pub guild: InviteGuild,
    /// The user that created the invite.
    pub inviter: User,
    /// The maximum age of the invite in seconds, from when it was created.
    pub max_age: u64,
    /// The maximum number of times that an invite may be used before it expires.

    /// Note that this does not supercede the [`max_age`] value, if the value of
    /// [`temporary`] is `true`. If the value of `temporary` is `false`, then the
    /// invite _will_ self-expire after the given number of max uses.

    /// If the value is `0`, then the invite is permanent.
    ///
    /// [`max_age`]: #structfield.max_age
    /// [`temporary`]: #structfield.temporary
    pub max_uses: u64,
    /// Indicator of whether the invite self-expires after a certain amount of
    /// time or uses.
    pub temporary: bool,
    /// The amount of times that an invite has been used.
    pub uses: u64,
}

impl RichInvite {
    /// Deletes the invite.
    ///
    /// Refer to [`rest::delete_invite`] for more information.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` feature is enabled, then this returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required [permission].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Invite::delete`]: struct.Invite.html#method.delete
    /// [`rest::delete_invite`]: ../client/rest/fn.delete_invite.html
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    /// [permission]: permissions/index.html
    pub fn delete(&self) -> Result<Invite> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_GUILD;

            if !model_utils::user_has_perms(self.channel.id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::delete_invite(&self.code)
    }
}
