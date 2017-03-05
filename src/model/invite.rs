use super::{Invite, RichInvite};
use ::client::rest;
use ::internal::prelude::*;
use ::model::ChannelId;
use ::utils::builder::CreateInvite;
use ::utils;

#[cfg(feature="cache")]
use super::permissions;
#[cfg(feature="cache")]
use super::utils as model_utils;

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

        rest::create_invite(channel_id.0, &f(CreateInvite::default()).0.build())
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
