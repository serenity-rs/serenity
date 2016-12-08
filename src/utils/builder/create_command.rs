use std::default::Default;
use ::client::Context;
use ::model::Message;

pub use ext::framework::command::Command;
pub use ext::framework::command::CommandType;

pub struct CreateCommand(pub Command);

impl CreateCommand {
    /// Closure or function that's called when a command is called.
    pub fn exec<F>(mut self, func: F) -> Self
        where F: Fn(&Context, &Message, Vec<String>) + Send + Sync + 'static {
        self.0 = Command {
            exec: CommandType::Basic(Box::new(func)),
            desc: self.0.desc,
            usage: self.0.usage,
            use_quotes: self.0.use_quotes
        };

        self
    }

    /// Description, used by other commands.
    pub fn desc(mut self, desc: &str) -> Self {
        self.0 = Command {
            exec: self.0.exec,
            desc: Some(desc.to_string()),
            usage: self.0.usage,
            use_quotes: self.0.use_quotes
        };

        self
    }

    /// Command usage schema, used by other commands.
    pub fn usage(mut self, usage: &str) -> Self {
        self.0 = Command {
            exec: self.0.exec,
            desc: self.0.desc,
            usage: Some(usage.to_string()),
            use_quotes: self.0.use_quotes
        };

        self
    }

    /// Whether arguments should be parsed using quote parser or not.
    pub fn use_quotes(mut self, use_quotes: bool) -> Self {
        self.0 = Command {
            exec: self.0.exec,
            desc: self.0.desc,
            usage: self.0.usage,
            use_quotes: use_quotes
        };

        self
    }
}

impl Default for Command {
    fn default() -> Command {
        Command {
            exec: CommandType::Basic(Box::new(|_, _, _| {})),
            desc: None,
            usage: None,
            use_quotes: true
        }
    }
}
