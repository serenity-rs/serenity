pub use super::command::{Command, CommandGroup, CommandType};
pub(crate) use super::command::CommandOrAlias;
pub use super::create_command::CreateCommand;
pub use super::Args;

use std::default::Default;
use std::sync::Arc;
use client::Context;
use model::Message;

/// Used to create command groups
///
/// # Examples
///
/// Create group named Information where all commands are prefixed with info,
/// and add one command named "name". For example, if prefix is "~", we say "~info name"
/// to call the "name" command.
///
/// ```rust,ignore
/// framework.group("Information", |g| g
///     .prefix("info")
///     .command("name", |c| c
///         .exec_str("Hakase")))
/// ```
#[derive(Default)]
pub struct CreateGroup(pub CommandGroup);

impl CreateGroup {
    /// Adds a command to group.
    pub fn command<F>(mut self, command_name: &str, f: F) -> Self
        where F: FnOnce(CreateCommand) -> CreateCommand {
        let cmd = f(CreateCommand(Command::default())).0;

        for n in &cmd.aliases {
            if let Some(ref prefix) = self.0.prefix {
                self.0.commands.insert(
                    format!("{} {}", prefix, n.to_owned()),
                    CommandOrAlias::Alias(
                        format!("{} {}", prefix, command_name.to_string()),
                    ),
                );
            } else {
                self.0.commands.insert(
                    n.to_owned(),
                    CommandOrAlias::Alias(command_name.to_string()),
                );
            }
        }

        self.0.commands.insert(
            command_name.to_owned(),
            CommandOrAlias::Command(Arc::new(cmd)),
        );

        self
    }

    /// Adds a command to group with simplified API.
    /// You can return Err(string) if there's an error.
    pub fn on<F>(mut self, command_name: &str, f: F) -> Self
        where F: Fn(&mut Context, &Message, Args) -> Result<(), String> + Send + Sync + 'static {
        let cmd = Arc::new(Command::new(f));

        self.0.commands.insert(
            command_name.to_owned(),
            CommandOrAlias::Command(cmd),
        );

        self
    }

    /// If prefix is set, it will be required before all command names.
    /// For example, if bot prefix is "~" and group prefix is "image"
    /// we'd call a subcommand named "hibiki" by sending "~image hibiki".
    ///
    /// **Note**: serenity automatically puts a space after group prefix.
    ///
    /// **Note**: It's suggested to call this first when making a group.
    pub fn prefix(mut self, desc: &str) -> Self {
        self.0.prefix = Some(desc.to_owned());

        self
    }
}
