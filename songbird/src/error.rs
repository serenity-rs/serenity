//! Driver and gateway error handling.

#[cfg(feature = "serenity")]
use futures::channel::mpsc::TrySendError;
#[cfg(feature = "serenity")]
use serenity::gateway::InterMessage;
#[cfg(feature = "gateway")]
use std::{error::Error, fmt};
#[cfg(feature = "twilight")]
use twilight_gateway::shard::CommandError;

#[cfg(feature = "gateway")]
#[derive(Debug)]
/// Error returned when a manager or call handler is
/// unable to send messages over Discord's gateway.
pub enum JoinError {
    /// No available gateway connection was provided to send
    /// voice state update messages.
    NoSender,
    /// Tried to leave a [`Call`] which was not found.
    ///
    /// [`Call`]: ../struct.Call.html
    NoCall,
    #[cfg(feature = "serenity")]
    /// Serenity-specific WebSocket send error.
    Serenity(TrySendError<InterMessage>),
    #[cfg(feature = "twilight")]
    /// Twilight-specific WebSocket send error.
    Twilight(CommandError),
}

#[cfg(feature = "gateway")]
impl fmt::Display for JoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to Join Voice channel: ")?;
        match self {
            JoinError::NoSender => write!(f, "no gateway destination."),
            JoinError::NoCall => write!(f, "tried to leave a non-existent call."),
            #[cfg(feature = "serenity")]
            JoinError::Serenity(t) => write!(f, "serenity failure {}.", t),
            #[cfg(feature = "twilight")]
            JoinError::Twilight(t) => write!(f, "twilight failure {}.", t),
        }
    }
}

#[cfg(feature = "gateway")]
impl Error for JoinError {}

#[cfg(all(feature = "serenity", feature = "gateway"))]
impl From<TrySendError<InterMessage>> for JoinError {
    fn from(e: TrySendError<InterMessage>) -> Self {
        JoinError::Serenity(e)
    }
}

#[cfg(all(feature = "twilight", feature = "gateway"))]
impl From<CommandError> for JoinError {
    fn from(e: CommandError) -> Self {
        JoinError::Twilight(e)
    }
}

#[cfg(feature = "gateway")]
/// Convenience type for Discord gateway error handling.
pub type JoinResult<T> = Result<T, JoinError>;

#[cfg(feature = "driver")]
pub use crate::driver::connection::error::{Error as ConnectionError, Result as ConnectionResult};
