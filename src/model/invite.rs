use super::{Invite, RichInvite};
use ::client::http;
use ::prelude::*;
use super::{permissions, utils};

impl Invite {
    /// Accepts the invite.
    ///
    /// **Note**: This will fail if you are already in the [`Guild`], or are
    /// banned. A ban is equivilant to an IP ban.
    ///
    /// [`Guild`]: struct.Guild.html
    pub fn accept(&self) -> Result<Invite> {
        http::accept_invite(&self.code)
    }

    /// Deletes the invite.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required [permission].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn delete(&self) -> Result<Invite> {
        let req = permissions::MANAGE_GUILD;

        if !try!(utils::user_has_perms(self.channel.id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::delete_invite(&self.code)
    }
}

impl RichInvite {
    /// Accepts the invite.
    ///
    /// Refer to the documentation for [`Invite::accept`] for restrictions on
    /// accepting an invite.
    ///
    /// [`Invite::accept`]: struct.Invite.html#method.accept
    pub fn accept(&self) -> Result<Invite> {
        http::accept_invite(&self.code)
    }

    /// Deletes the invite.
    ///
    /// Refer to the documentation for [`Invite::delete`] for restrictions on
    /// deleting an invite.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required [permission].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Invite::delete`]: struct.Invite.html#method.delete
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    pub fn delete(&self) -> Result<Invite> {
        let req = permissions::MANAGE_GUILD;

        if !try!(utils::user_has_perms(self.channel.id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::delete_invite(&self.code)
    }
}
