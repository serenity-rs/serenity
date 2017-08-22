pub use super::{Args, Command, CommandGroup, CommandType};

use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use client::Context;
use model::{Message, Permissions};

pub struct CreateCommand(pub Command);

impl CreateCommand {
    /// Adds multiple aliases.
    pub fn batch_known_as(mut self, names: Vec<&str>) -> Self {
        self.0.aliases.extend(
            names.into_iter().map(|n| n.to_owned()),
        );

        self
    }

    /// Adds a ratelimit bucket.
    pub fn bucket(mut self, bucket: &str) -> Self {
        self.0.bucket = Some(bucket.to_owned());

        self
    }

    /// Adds a "check" to a command, which checks whether or not the command's
    /// function should be called.
    ///
    /// # Examples
    ///
    /// Ensure that the user who created a message, calling a "ping" command,
    /// is the owner.
    ///
    /// ```rust,no_run
    /// # use serenity::prelude::EventHandler;
    /// # struct Handler;
    /// # impl EventHandler for Handler {}
    /// use serenity::client::{Client, Context};
    /// use serenity::framework::standard::{Args, Command, StandardFramework};
    /// use serenity::model::Message;
    /// use std::env;
    /// use std::sync::Arc;
    ///
    /// let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap(), Handler);
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .configure(|c| c.prefix("~"))
    ///     .command("ping", |c| c
    ///         .check(owner_check)
    ///         .desc("Replies to a ping with a pong")
    ///         .exec(ping)));
    ///
    /// fn ping(_context: &mut Context, message: &Message, _args: Args) -> Result<(),
    /// String> {
    ///     let _ = message.channel_id.say("Pong!");
    ///
    ///     Ok(())
    /// }
    ///
    /// fn owner_check(_context: &mut Context, message: &Message, _: &mut Args, _:
    /// &Arc<Command>) -> bool {
    ///     // replace with your user ID
    ///     message.author.id == 7
    /// }
    /// ```
    pub fn check<F>(mut self, check: F) -> Self
        where F: Fn(&mut Context, &Message, &mut Args, &Arc<Command>) -> bool
                     + Send
                     + Sync
                     + 'static {
        self.0.checks.push(Box::new(check));

        self
    }

    /// Description, used by other commands.
    pub fn desc(mut self, desc: &str) -> Self {
        self.0.desc = Some(desc.to_owned());

        self
    }

    /// Whether command can be used only privately or not.
    pub fn dm_only(mut self, dm_only: bool) -> Self {
        self.0.dm_only = dm_only;

        self
    }

    /// Example arguments, used by other commands.
    pub fn example(mut self, example: &str) -> Self {
        self.0.example = Some(example.to_owned());

        self
    }

    /// A function that can be called when a command is received.
    /// You can return `Err(string)` if there's an error.
    ///
    /// See [`exec_str`] if you _only_ need to return a string on command use.
    ///
    /// [`exec_str`]: #method.exec_str
    pub fn exec<F>(mut self, func: F) -> Self
        where F: Fn(&mut Context, &Message, Args) -> Result<(), String> + Send + Sync + 'static {
        self.0.exec = CommandType::Basic(Box::new(func));

        self
    }

    /// Sets a function that's called when a command is called that can access
    /// the internal HashMap of commands, used specifically for creating a help
    /// command.
    ///
    /// You can return `Err(string)` if there's an error.
    pub fn exec_help<F>(mut self, f: F) -> Self
        where F: Fn(&mut Context,
                    &Message,
                    HashMap<String, Arc<CommandGroup>>,
                    Args)
                    -> Result<(), String>
                     + 'static {
        self.0.exec = CommandType::WithCommands(Box::new(f));

        self
    }

    /// Sets a string to be sent in the channel of context on command. This can
    /// be useful for an `about`, `invite`, `ping`, etc. command.
    ///
    /// # Examples
    ///
    /// Create a command named "ping" that returns "Pong!":
    ///
    /// ```rust,ignore
    /// client.with_framework(|f| f
    ///     .command("ping", |c| c.exec_str("Pong!")));
    /// ```
    pub fn exec_str(mut self, content: &str) -> Self {
        self.0.exec = CommandType::StringResponse(content.to_owned());

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

    /// Adds an alias, allowing users to use the command under a different name.
    pub fn known_as(mut self, name: &str) -> Self {
        self.0.aliases.push(name.to_owned());

        self
    }

    /// Maximum amount of arguments that can be passed.
    pub fn max_args(mut self, max_args: i32) -> Self {
        self.0.max_args = Some(max_args);

        self
    }

    /// Minumum amount of arguments that should be passed.
    pub fn min_args(mut self, min_args: i32) -> Self {
        self.0.min_args = Some(min_args);

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

    /// Command usage schema, used by other commands.
    pub fn usage(mut self, usage: &str) -> Self {
        self.0.usage = Some(usage.to_owned());

        self
    }
}

impl Default for Command {
    fn default() -> Command {
        Command {
            aliases: Vec::new(),
            checks: Vec::default(),
            exec: CommandType::Basic(Box::new(|_, _, _| Ok(()))),
            desc: None,
            usage: None,
            example: None,
            min_args: None,
            bucket: None,
            max_args: None,
            required_permissions: Permissions::empty(),
            dm_only: false,
            guild_only: false,
            help_available: true,
            owners_only: false,
        }
    }
}
