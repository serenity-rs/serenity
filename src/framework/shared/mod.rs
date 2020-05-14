pub mod args;


pub struct CommandError(pub String);
impl<T: std::fmt::Display> From<T> for CommandError {
    #[inline]
    fn from(d: T) -> Self {
        CommandError(d.to_string())
    }
}

pub type CommandResult = std::result::Result<(), CommandError>;