use std::error::Error as StdError;
use std::fmt;

/// An error returned from the [`Client`].
///
/// This is always wrapped within the library's generic [`Error::Client`]
/// variant.
///
/// [`Client`]: super::Client
/// [`Error::Client`]: crate::Error::Client
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// When a shard has completely failed to reboot after resume and/or
    /// reconnect attempts.
    ShardBootFailure,
    /// When all shards that the client is responsible for have shutdown with an
    /// error.
    Shutdown,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShardBootFailure => f.write_str("Failed to (re-)boot a shard"),
            Self::Shutdown => f.write_str("The clients shards shutdown"),
        }
    }
}

impl StdError for Error {}
