use super::{Invite, RichInvite};
use ::client::http;
use ::internal::prelude::*;
use super::{permissions, utils};

#[cfg(feature = "state")]
use ::client::STATE;

impl Invite {
    /// Accepts the invite, placing the current user in the [`Guild`] that the
    /// invite was for. This will fire the [`Client::on_guild_create`] handler
    /// once the associated event is received.
    ///
    /// **Note**: This will fail if you are already in the `Guild`, or are
    /// banned. A ban is equivilant to an IP ban.
    ///
    /// **Note**: Requires that the current user be a user account. Bots can not
    /// accept invites. Instead they must be accepted via OAuth2 authorization
    /// links. These are in the format of:
    ///
    /// `https://discordapp.com/oauth2/authorize?client_id=CLIENT_ID&scope=bot`
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidOperationAsBot`] if the current user is
    /// a bot user.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Client::on_guild_create`]: ../client/struct.Client.html#method.on_guild_create
    /// [`Guild`]: struct.Guild.html
    #[cfg(feature="methods")]
    pub fn accept(&self) -> Result<Invite> {
        feature_state_enabled! {{
            if STATE.lock().unwrap().user.bot {
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
    /// invite was for. This will fire the [`Client::on_guild_create`] handler
    /// once the associated event is received.
    ///
    /// Refer to the documentation for [`Invite::accept`] for restrictions on
    /// accepting an invite.
    ///
    /// **Note**: Requires that the current user be a user account.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidOperationAsBot`] if the current user is
    /// a bot user.
    ///
    /// [`ClientError::InvalidOperationAsBot`]: enum.ClientError.html#variant.InvalidOperationAsBot
    /// [`Invite::accept`]: struct.Invite.html#method.accept
    #[cfg(feature="methods")]
    pub fn accept(&self) -> Result<Invite> {
        feature_state_enabled! {{
            if STATE.lock().unwrap().user.bot {
                return Err(Error::Client(ClientError::InvalidOperationAsBot));
            }
        }}

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
    #[cfg(feature="methods")]
    pub fn delete(&self) -> Result<Invite> {
        let req = permissions::MANAGE_GUILD;

        if !try!(utils::user_has_perms(self.channel.id, req)) {
            return Err(Error::Client(ClientError::InvalidPermissions(req)));
        }

        http::delete_invite(&self.code)
    }
}
