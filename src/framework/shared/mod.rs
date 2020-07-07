use std::error::Error as StdError;

pub mod args;

pub type CommandError = Box<dyn StdError + Send + Sync>;
pub type CommandResult = std::result::Result<(), CommandError>;