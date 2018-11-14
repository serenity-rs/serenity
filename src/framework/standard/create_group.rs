pub use super::command::{
    Command,
    CommandGroup,
    CommandOptions,
    Error as CommandError
};
pub(crate) use super::command::CommandOrAlias;
pub use super::{
    create_help_command::CreateHelpCommand,
    create_command::{CreateCommand, FnOrCommand},
    Args,
    Check,
};

use client::Context;
use model::{
    channel::Message,
    Permissions,
};
use std::sync::Arc;

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
        let mut cmd = CreateCommand::default()
            .required_permissions(self.0.required_permissions)
            .dm_only(self.0.dm_only)
            .guild_only(self.0.guild_only)
            .help_available(self.0.help_available)
            .owners_only(self.0.owners_only)
            .owner_privileges(self.0.owner_privileges);

        if let Some(ref bucket) = self.0.bucket {
            cmd = cmd.bucket(bucket);
        }
        cmd.0.allowed_roles = self.0.allowed_roles.clone();
        cmd
    }

    /// Adds a command to group.
    pub fn command<F>(mut self, command_name: &str, f: F) -> Self
        where F: FnOnce(CreateCommand) -> CreateCommand {
        let cmd = f(self.build_command()).finish();

        for n in &cmd.options().aliases {

            if let Some(ref prefixes) = self.0.prefixes {

                for prefix in prefixes {
                    self.0.commands.insert(
                        format!("{} {}", prefix, n.to_string()),
                        CommandOrAlias::Alias(format!("{} {}", prefix, command_name.to_string())),
                    );
                }
            } else {
                self.0.commands.insert(
                    n.to_string(),
                    CommandOrAlias::Alias(command_name.to_string()),
                );
            }
        }

        self.0.commands.insert(
            command_name.to_string(),
            CommandOrAlias::Command(cmd),
        );

        self
    }

    /// Adds a command to group with a simplified API.
    /// You can return Err(From::from(string)) if there's an error.
    pub fn on(self, name: &str,
            f: fn(&mut Context, &Message, Args) -> Result<(), CommandError>) -> Self {
        self.cmd(name, f)
    }

    /// Like [`on`], but accepts a `Command` directly.
    ///
    /// [`on`]: #method.on
    pub fn cmd<C: Command + 'static>(mut self, name: &str, c: C) -> Self {
        let cmd: Arc<Command> = Arc::new(c);

        for n in &cmd.options().aliases {

            if let Some(ref prefixes) = self.0.prefixes {

                for prefix in prefixes {
                    self.0.commands.insert(
                        format!("{} {}", prefix, n.to_string()),
                        CommandOrAlias::Alias(format!("{} {}", prefix, name.to_string())),
                    );
                }
            } else {
                self.0.commands.insert(
                    n.to_string(),
                    CommandOrAlias::Alias(name.to_string()),
                );
            }
        }

        self.0
            .commands
            .insert(name.to_string(), CommandOrAlias::Command(Arc::clone(&cmd)));

        cmd.init();

        self
    }

    /// If prefix is set, it will be required before all command names.
    /// For example, if bot prefix is "~" and group prefix is "image"
    /// we'd call a subcommand named "hibiki" by sending "~image hibiki".
    ///
    /// **Note**: serenity automatically puts a space after group prefix.
    ///
    /// **Note**: It's suggested to call this first when making a group.
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.0.prefixes = Some(vec![prefix.to_string()]);

        self
    }

    /// Sets prefixes to respond to. Each can be a string slice of any
    /// non-zero length.
    ///
    /// **Note**: serenity automatically puts a space after group prefix.
    ///
    /// **Note**: It's suggested to call this first when making a group.
    pub fn prefixes<T: ToString, I: IntoIterator<Item=T>>(mut self, prefixes: I) -> Self {
        self.0.prefixes = Some(prefixes.into_iter().map(|prefix| prefix.to_string()).collect());

        self
    }

    /// Adds a ratelimit bucket.
    pub fn bucket(mut self, bucket: &str) -> Self {
        self.0.bucket = Some(bucket.to_string());

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

    /// Whether owners shall bypass buckets, missing permissions,
    /// wrong channels, missing roles, and checks.
    pub fn owner_privileges(mut self, owner_privileges: bool) -> Self {
        self.0.owner_privileges = owner_privileges;

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
    #[cfg(feature = "cache")]
    pub fn allowed_roles<T: ToString, It: IntoIterator<Item=T>>(mut self, allowed_roles: It) -> Self {
        self.0.allowed_roles = allowed_roles.into_iter().map(|x| x.to_string()).collect();

        self
    }

    /// Adds a "check" to a group, which checks whether or not the groups's
    /// commands should be called.
    ///
    /// **Note**: These checks are bypassed for commands sent by the application owner.
    pub fn check<F>(mut self, check: F) -> Self
        where F: Fn(&mut Context, &Message, &mut Args, &CommandOptions) -> bool
                     + Send
                     + Sync
                     + 'static {
        self.0.checks.push(Check::new(check));

        self
    }

    /// Adds a command for a group that will be executed if no command-name
    /// has been passed.
    pub fn default_cmd<C: Command + 'static>(mut self, c: C) -> Self {
        c.init();

        let cmd_with_group_options = self.build_command().cmd(c).finish();
        let cmd_finished = CommandOrAlias::Command(cmd_with_group_options);
        self.0.default_command = Some(cmd_finished);

        self
    }

    /// Sets a description for the group that will be displayed if only
    /// one specific group is requested via help.
    pub fn desc(mut self, text: &str) -> Self {
        self.0.description = Some(text.to_string());

        self
    }
}
