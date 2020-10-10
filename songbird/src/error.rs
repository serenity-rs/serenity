#[cfg(feature = "serenity")]
use futures::channel::mpsc::TrySendError;
#[cfg(feature = "serenity")]
use serenity::gateway::InterMessage;
#[cfg(feature = "twilight")]
use twilight_gateway::shard::CommandError;

#[cfg(feature = "gateway")]
/// Error returned when a manager or call handler is
/// unable to send messages over Discord's gateway.
pub enum JoinError {
	NoSender,
	#[cfg(feature = "serenity")]
	Serenity(TrySendError<InterMessage>),
	#[cfg(feature = "twilight")]
	Twilight(CommandError),
}

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
pub type JoinResult<T> = Result<T, JoinError>;

#[cfg(feature = "driver")]
pub use crate::driver::Error as ConnectionError;
