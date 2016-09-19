use super::{Invite, RichInvite};
use ::client::http;
use ::prelude::*;

impl Invite {
    /// Accepts an invite.
    ///
    /// Refer to the documentation for [`Context::accept_invite`] for
    /// restrictions on accepting an invite.
    ///
    /// [`Context::accept_invite`]: ../client/struct.Context.html#method.accept_invite
    pub fn accept(&self) -> Result<Invite> {
        http::accept_invite(&self.code)
    }

    /// Deletes an invite.
    ///
    /// Refer to the documentation for [`Context::delete_invite`] for more
    /// information.
    ///
    /// [`Context::delete_invite`]: ../client/struct.Context.html#method.delete_invite
    pub fn delete(&self) -> Result<Invite> {
        http::delete_invite(&self.code)
    }
}

impl RichInvite {
    /// Accepts an invite.
    ///
    /// Refer to the documentation for [`Context::accept_invite`] for
    /// restrictions on accepting an invite.
    ///
    /// [`Context::accept_invite`]: ../client/struct.Context.html#method.accept_invite
    pub fn accept(&self) -> Result<Invite> {
        http::accept_invite(&self.code)
    }

    /// Deletes an invite.
    ///
    /// Refer to the documentation for [`Context::delete_invite`] for more
    /// information.
    ///
    /// [`Context::delete_invite`]: ../client/struct.Context.html#method.delete_invite
    pub fn delete(&self) -> Result<Invite> {
        http::delete_invite(&self.code)
    }
}
