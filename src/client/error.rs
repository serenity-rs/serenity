use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error returned from the [`Client`].
///
/// This is always wrapped within the library's generic [`Error::Client`]
/// variant.
///
/// [`Client`]: super::Client
/// [`Error`]: crate::Error
/// [`Error::Client`]: crate::Error::Client
/// [`GuildId::ban`]: crate::model::id::GuildId::ban
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// When the token provided is invalid. This is returned when validating a
    /// token through the [`validate_token`] function.
    ///
    /// [`validate_token`]: super::validate_token
    InvalidToken,
    /// When a shard has completely failed to reboot after resume and/or
    /// reconnect attempts.
    ShardBootFailure,
    /// When all shards that the client is responsible for have shutdown with an
    /// error.
    Shutdown,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::InvalidToken => f.write_str("The provided token was invalid"),
            Error::ShardBootFailure => f.write_str("Failed to (re-)boot a shard"),
            Error::Shutdown => f.write_str("The clients shards shutdown"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidToken => "The provided token was invalid",
            Error::ShardBootFailure => "Failed to (re-)boot a shard",
            Error::Shutdown => "The clients shards shutdown",
        }
    }
}
