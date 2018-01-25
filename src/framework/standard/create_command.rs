pub use super::{Args, Command, CommandGroup, CommandOptions, CommandError};

use client::Context;
use model::channel::Message;
use model::Permissions;
use std::sync::Arc;

pub enum FnOrCommand {
    Fn(fn(&mut Context, &Message, Args) -> Result<(), CommandError>),
    Command(Arc<Command>),
    CommandWithOptions(Arc<Command>),
}

impl Default for FnOrCommand {
    fn default() -> Self {
        FnOrCommand::Fn(|_, _, _| Ok(()))
    }
}

type Init = Fn() + Send + Sync + 'static;
type Before = Fn(&mut Context, &Message) -> bool + Send + Sync + 'static;
type After = Fn(&mut Context, &Message, &Result<(), CommandError>) + Send + Sync + 'static;

#[derive(Default)]
pub struct Handlers {
    init: Option<Arc<Init>>,
    before: Option<Arc<Before>>,
    after: Option<Arc<After>>,
}

#[derive(Default)]
pub struct CreateCommand(pub CommandOptions, pub FnOrCommand, pub Handlers);

impl CreateCommand {
    /// Adds multiple aliases.
    pub fn batch_known_as<T: ToString, It: IntoIterator<Item=T>>(mut self, names: It) -> Self {
        self.0
            .aliases
            .extend(names.into_iter().map(|n| n.to_string()));

        self
    }

    /// Adds a ratelimit bucket.
    pub fn bucket(mut self, bucket: &str) -> Self {
        self.0.bucket = Some(bucket.to_string());

        self
    }

    /// Adds a "check" to a command, which checks whether or not the command's
    /// function should be called.
    ///
    /// These checks are bypassed for commands sent by the application owner.
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
    /// use serenity::framework::standard::{
    ///     Args,
    ///     CommandOptions,
    ///     CommandError,
    ///     StandardFramework,
    /// };
    /// use serenity::model::channel::Message;
    /// use std::env;
    ///
    /// let token = env::var("DISCORD_TOKEN").unwrap();
    /// let mut client = Client::new(&token, Handler).unwrap();
    ///
    /// client.with_framework(StandardFramework::new()
    ///     .configure(|c| c.prefix("~"))
    ///     .command("ping", |c| c
    ///         .check(owner_check)
    ///         .desc("Replies to a ping with a pong")
    ///         .exec(ping)));
    ///
    /// fn ping(_context: &mut Context, message: &Message, _args: Args) -> Result<(),
    /// CommandError> {
    ///     message.channel_id.say("Pong!")?;
    ///
    ///     Ok(())
    /// }
    ///
    /// fn owner_check(_context: &mut Context, message: &Message, _: &mut Args, _:
    /// &CommandOptions) -> bool {
    ///     // replace with your user ID
    ///     message.author.id == 7
    /// }
    /// ```
    pub fn check<F>(mut self, check: F) -> Self
        where F: Fn(&mut Context, &Message, &mut Args, &CommandOptions) -> bool
                     + Send
                     + Sync
                     + 'static {
        self.0.checks.push(Box::new(check));

        self
    }

    /// Description, used by other commands.
    pub fn desc(mut self, desc: &str) -> Self {
        self.0.desc = Some(desc.to_string());

        self
    }

    /// Whether command can be used only privately or not.
    pub fn dm_only(mut self, dm_only: bool) -> Self {
        self.0.dm_only = dm_only;

        self
    }

    /// Example arguments, used by other commands.
    pub fn example(mut self, example: &str) -> Self {
        self.0.example = Some(example.to_string());

        self
    }

    /// A function that can be called when a command is received.
    /// You can return `Err(string)` if there's an error.
    pub fn exec(mut self, func: fn(&mut Context, &Message, Args) -> Result<(), CommandError>) -> Self {
        self.1 = FnOrCommand::Fn(func);

        self
    }

    /// Like [`exec`] but accepts a `Command` directly.
    ///
    /// [`exec`]: #method.exec
    pub fn cmd<C: Command + 'static>(mut self, c: C) -> Self {
        self.1 = FnOrCommand::Command(Arc::new(c));

        self
    }

    /// Like [`cmd`] but says to the builder to use this command's options instead of its own.
    ///
    /// [`cmd`]: #method.cmd
    pub fn cmd_with_options<C: Command + 'static>(mut self, c: C) -> Self {
        self.1 = FnOrCommand::CommandWithOptions(Arc::new(c));

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
        self.0.aliases.push(name.to_string());

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

    /// Exact number of arguments that should be passed.
    pub fn num_args(mut self, num_args: i32) -> Self {
        self.0.min_args = Some(num_args);
        self.0.max_args = Some(num_args);

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
        self.0.usage = Some(usage.to_string());

        self
    }

    /// Sets roles that are allowed to use the command.
    pub fn allowed_roles<T: ToString, It: IntoIterator<Item=T>>(mut self, allowed_roles: It) -> Self {
        self.0.allowed_roles = allowed_roles.into_iter().map(|x| x.to_string()).collect();

        self
    }

    /// Sets an initialise middleware to be called upon the command's actual registration.
    /// 
    /// This is similiar to implementing the `init` function on `Command`.
    pub fn init<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.2.init = Some(Arc::new(f));

        self
    }

    /// Sets a before middleware to be called before the command's execution.
    /// 
    /// This is similiar to implementing the `before` function on `Command`.
    pub fn before<F: Send + Sync + 'static>(mut self, f: F) -> Self 
        where F: Fn(&mut Context, &Message) -> bool {
        self.2.before = Some(Arc::new(f));

        self
    }

    /// Sets an after middleware to be called after the command's execution.
    /// 
    /// This is similiar to implementing the `after` function on `Command`.
    pub fn after<F: Send + Sync + 'static>(mut self, f: F) -> Self 
        where F: Fn(&mut Context, &Message, &Result<(), CommandError>) {
        self.2.after = Some(Arc::new(f));

        self
    }

    pub(crate) fn finish(self) -> Arc<Command> {
        struct A<C: Command>(Arc<CommandOptions>, C, Handlers);

        impl<C: Command> Command for A<C> {
            fn execute(&self, c: &mut Context, m: &Message, a: Args) -> Result<(), CommandError> {
                self.1.execute(c, m, a)
            }

            fn options(&self) -> Arc<CommandOptions> { Arc::clone(&self.0) }

            fn init(&self) {
                if let Some(ref init) = self.2.init {
                    (init)();
                }
            }

            fn before(&self, c: &mut Context, m: &Message) -> bool {
                if let Some(ref before) = self.2.before {
                    return (before)(c, m);
                }

                true
            }

            fn after(&self, c: &mut Context, m: &Message, res: &Result<(), CommandError>) {
                if let Some(ref after) = self.2.after {
                    (after)(c, m, res);
                }
            }
        }

        let CreateCommand(options, fc, handlers) = self;

        match fc {
            FnOrCommand::Fn(func) => {
                Arc::new(A(Arc::new(options), func, handlers))
            },
            FnOrCommand::Command(cmd) => {
                Arc::new(A(Arc::new(options), cmd, handlers))
            },
            FnOrCommand::CommandWithOptions(cmd) => cmd,
        }
    }
}

