use std::sync::Arc;
use ::client::Context;
use ::model::Message;

pub type Command = Fn(Context, Message) + Send + Sync;
#[doc(hidden)]
pub type InternalCommand = Arc<Command>;
