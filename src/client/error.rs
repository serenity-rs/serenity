use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// An error returned from the [`Client`].
///
/// This is always wrapped within the library's generic [`Error::Client`]
/// variant.
///
/// [`Client`]: struct.Client.html
/// [`Error`]: ../enum.Error.html
/// [`Error::Client`]: ../enum.Error.html#variant.Client
/// [`GuildId::ban`]: ../model/struct.GuildId.html#method.ban
#[allow(enum_variant_names)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    /// When the token provided is invalid. This is returned when validating a
    /// token through the [`validate_token`] function.
    ///
    /// [`validate_token`]: fn.validate_token.html
    InvalidToken,
    /// When a shard has completely failed to reboot after resume and/or
    /// reconnect attempts.
    ShardBootFailure,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidToken => "The provided token was invalid",
            Error::ShardBootFailure => "Failed to (re-)boot a shard",
        }
    }
}
