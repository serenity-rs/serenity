pub use ext::framework::command::{Command, CommandType, CommandGroup};
pub use ext::framework::create_command::CreateCommand;

use std::collections::HashMap;
use std::default::Default;
use ::client::Context;
use ::model::Message;
use ::model::Permissions;
use std::sync::Arc;

pub struct CreateGroup(pub CommandGroup);

impl CreateGroup {
    /// If prefix is set, it will be required before all command names.
    pub fn prefix(mut self, desc: &str) -> Self {
        self.0.prefix = Some(desc.to_owned());

        self
    }

    /// Adds a command to group.
    pub fn command<F, S>(mut self, command_name: S, f: F) -> Self
        where F: FnOnce(CreateCommand) -> CreateCommand,
              S: Into<String> {

        let cmd = f(CreateCommand(Command::default())).0;

        self.0.commands.insert(command_name.into(), Arc::new(cmd));

        self
    }

    /// Adds a command to group with simplified API.
    pub fn on<F, S>(mut self, command_name: S, f: F) -> Self
        where F: Fn(&Context, &Message, Vec<String>) + Send + Sync + 'static,
              S: Into<String> {

        self.0.commands.insert(command_name.into(), Arc::new(Command {
            checks: Vec::default(),
            exec: CommandType::Basic(Box::new(f)),
            desc: None,
            usage: None,
            use_quotes: false,
            dm_only: false,
            guild_only: false,
            help_available: true,
            min_args: None,
            max_args: None,
            required_permissions: Permissions::empty()
        }));

        self
    }
}

impl Default for CommandGroup {
    fn default() -> CommandGroup {
        CommandGroup {
            prefix: None,
            commands: HashMap::new()
        }
    }
}
