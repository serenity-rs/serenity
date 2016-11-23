use super::{Invite, RichInvite};
use ::client::http;
use ::internal::prelude::*;
use super::{permissions, utils};

#[cfg(feature = "cache")]
use ::client::CACHE;

impl Invite {
    /// Accepts the invite, placing the current user in the [`Guild`] that the
    /// invite was for.
    ///
    /// Refer to [`http::accept_invite`] for more information.
    ///
    /// **Note**: This will fail if you are already in the guild, or are banned.
    /// A ban is equivilant to an IP ban.
    ///
    /// **Note**: Requires that the current user be a user account.
    ///
    /// # Errors
    ///
    /// If the `cache` features is enabled, then this returns a
    /// [`ClientError::InvalidOperationAsBot`] if the current user does not have
    /// the required [permission].
    ///
    /// [`ClientError::InvalidOperationAsBot`]: enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Guild`]: struct.Guild.html
    /// [`http::accept_invite`]: ../client/http/fn.accept_invite.html
    /// [permission]: permissions/index.html
    #[cfg(feature="methods")]
    pub fn accept(&self) -> Result<Invite> {
        feature_cache_enabled! {{
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }}

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
    /// [permission]: permissions/index.html
    #[cfg(feature="methods")]
    pub fn delete(&self) -> Result<Invite> {
        let req = permissions::MANAGE_GUILD;

        if !try!(utils::user_has_perms(self.channel.id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::delete_invite(&self.code)
    }
}

impl RichInvite {
    /// Accepts the invite, placing the current user in the [`Guild`] that the
    /// invite was for.
    ///
    /// Refer to [`http::accept_invite`] for more information.
    ///
    /// **Note**: This will fail if you are already in the guild, or are banned.
    /// A ban is equivilant to an IP ban.
    ///
    /// **Note**: Requires that the current user be a user account.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidOperationAsBot`] if the current user is
    /// a bot user.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Guild`]: struct.Guild.html
    /// [`http::accept_invite`]: ../client/http/fn.accept_invite.html
    #[cfg(feature="methods")]
    pub fn accept(&self) -> Result<Invite> {
        feature_cache_enabled! {{
            if CACHE.read().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }}

        http::accept_invite(&self.code)
    }

    /// Deletes the invite.
    ///
    /// Refer to [`http::delete_invite`] for more information.
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
    /// [`http::delete_invite`]: ../client/http/fn.delete_invite.html
    /// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
    /// [permission]: permissions/index.html
    #[cfg(feature="methods")]
    pub fn delete(&self) -> Result<Invite> {
        let req = permissions::MANAGE_GUILD;

        if !try!(utils::user_has_perms(self.channel.id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::delete_invite(&self.code)
    }
}
