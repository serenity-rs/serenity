pub use super::command::{Command, CommandGroup, CommandType};
pub(crate) use super::command::CommandOrAlias;
pub use super::create_command::CreateCommand;
pub use super::Args;

use std::default::Default;
use std::sync::Arc;
use client::Context;
use model::{Message, Permissions};
use std::collections::HashMap;

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
    fn build_command(&self) -> CreateCommand {
        let mut cmd = CreateCommand(Command::default())
            .required_permissions(self.0.required_permissions)
            .dm_only(self.0.dm_only)
            .guild_only(self.0.guild_only)
            .help_available(self.0.help_available)
            .owners_only(self.0.owners_only);

        if let Some(ref bucket) = self.0.bucket {
            cmd = cmd.bucket(&bucket);
        }
        cmd.0.allowed_roles = self.0.allowed_roles.clone();
        cmd
    }

    /// Adds a command to group.
    pub fn command<F>(mut self, command_name: &str, f: F) -> Self
        where F: FnOnce(CreateCommand) -> CreateCommand {
        let cmd = f(self.build_command()).0;

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

    /// Adds a ratelimit bucket.
    pub fn bucket(mut self, bucket: &str) -> Self {
        self.0.bucket = Some(bucket.to_owned());

        self
    }

    /// Whether command can be used only privately or not.
    pub fn dm_only(mut self, dm_only: bool) -> Self {
        self.0.dm_only = dm_only;

        self
    }

    /// Whether command can be used only in guilds or not.
    pub fn guild_only(mut self, guild_only: bool) -> Self {
        self.0.guild_only = guild_only;

        self
    }

    /// Whether command should be displayed in help list or not, used by other commands.
    pub fn help_available(mut self, help_available: bool) -> Self {
        self.0.help_available = help_available;

        self
    }

    /// Whether command can be used only privately or not.
    pub fn owners_only(mut self, owners_only: bool) -> Self {
        self.0.owners_only = owners_only;

        self
    }

    /// The permissions that a user must have in the contextual channel in order
    /// for the command to be processed.
    pub fn required_permissions(mut self, permissions: Permissions) -> Self {
        self.0.required_permissions = permissions;

        self
    }

    /// Sets roles that are allowed to use the command.
    pub fn allowed_roles(mut self, allowed_roles: Vec<&str>) -> Self {
        self.0.allowed_roles = allowed_roles.iter().map(|x| x.to_string()).collect();

        self
    }
}

impl Default for CommandGroup {
    fn default() -> CommandGroup {
        CommandGroup {
            prefix: None,
            commands: HashMap::new(),
            bucket: None,
            required_permissions: Permissions::empty(),
            dm_only: false,
            guild_only: false,
            help_available: true,
            owners_only: false,
            allowed_roles: Vec::new(),
        }
    }
}
