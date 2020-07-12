/// An alternative to the [`StandardFramework`]
/// much less configurability, but less complex and no procedural macros
///
/// [`StandardFramework`]: ../standard/index.html

use super::Framework;
use std::collections::HashMap;
use std::borrow::Cow;
use async_trait::async_trait;
use log::warn;
use crate::client::Context;
use crate::model::channel::Message;

use futures::future::BoxFuture;

pub use super::shared::{
    args::{Args, Delimiter, Error as ArgError, Iter, RawArguments},
    CommandResult
};

pub mod traits;

use self::traits::*;

type Command = Box<dyn for<'fut> Fn(&'fut Context, &'fut Message, Args) -> BoxFuture<'fut, CommandResult> + Send + Sync>;
type BeforeFn = Box<dyn for<'fut> Fn(&'fut Context, &'fut Message, &'fut str) -> BoxFuture<'fut, bool> + Send + Sync>;
type AfterFn = Box<dyn for <'fut> Fn(&'fut Context, &'fut Message, &'fut str, CommandResult) -> BoxFuture<'fut, ()> + Send + Sync>;
type UnrecognizedCommand = Box<dyn for<'fut> Fn(&'fut Context, &'fut Message, &'fut str, Args) -> BoxFuture<'fut, ()> + Send + Sync>;
type NormalMessage = Box<dyn for<'fut> Fn(&'fut Context, &'fut Message) -> BoxFuture<'fut, ()> + Send + Sync>;
type PrefixOnly = Box<dyn for<'fut> Fn(&'fut Context, &'fut Message) -> BoxFuture<'fut, ()> + Send + Sync>;
type HelpCommand = Box<dyn for<'fut> Fn(&'fut Context, &'fut Message, Args, &'fut [&'fut str]) -> BoxFuture<'fut, CommandResult> + Send + Sync + 'static>;

pub struct SimpleFramework {
    prefix: String,
    commands: HashMap<String, Command>,
    //a Vec only to interface with the Args struct easier
    delimiters: Vec<Delimiter>,
    case_insensitive: bool,
    before_cmd: Option<BeforeFn>,
    after_cmd: Option<AfterFn>,
    default_cmd: Option<UnrecognizedCommand>,
    normal_message_fn: Option<NormalMessage>,
    prefix_only_cmd: Option<PrefixOnly>,
    help_cmd: Option<HelpCommand>,
}

impl Default for SimpleFramework {
    fn default() -> Self {
        SimpleFramework {
            prefix: String::from("!"),
            delimiters: vec![Delimiter::Single(' ')],
            case_insensitive: false,
            commands: HashMap::new(),
            before_cmd: None,
            after_cmd: None,
            default_cmd: None,
            normal_message_fn: None,
            prefix_only_cmd: None,
            help_cmd: None,
        }
    }
}

impl SimpleFramework {

    #[inline]
    pub fn new() -> SimpleFramework {
        SimpleFramework::default()
    }

    /// Inserts a command into the framework
    /// 
    /// # Example
    ///
    /// Inserts a basic ping command into the framework
    ///
    /// ```rust,no_run
    /// # use serenity::client::{Client, Context};
    /// # use serenity::framework::simple::{SimpleFramework, Args, CommandResult};
    /// # use serenity::model::channel::Message;
    ///
    /// async fn ping(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    ///     msg.channel_id.say(&ctx.http, "pong!").await?;
    ///
    ///     Ok(())
    /// }
    ///
    /// let framework = SimpleFramework::new()
    ///                 .add("ping", ping);
    /// ```
    pub fn add<F>(mut self, name: &str, cmd: F) -> Self 
    where F: for<'r, 's> AsyncFn3<&'r Context, &'s Message, Args, Output = CommandResult> + Send + Sync + 'static {
    //F: Fn(&Context, &Message, Args) -> T + Send + Sync + 'static {
        self.commands.insert(name.to_owned(), Box::new(move |ctx, msg, args| Box::pin(cmd.call(ctx, msg, args))));
        self
    }

    /// sets the prefix which the simple framework will look for
    /// defaults to "!"
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_owned();
        self
    }

    /// Sets the function to run after each command
    /// it's passed the name of the command used and
    /// the `CommandResult` returned by the command
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use serenity::client::{Client, Context};
    /// # use serenity::framework::simple::{SimpleFramework, CommandResult};
    /// # use serenity::model::channel::Message;
    ///
    /// async fn after_fn(_ctx: &Context, _msg: &Message, cmd_name: &str, res: CommandResult) {
    ///     if let Err(why) = res {
    ///         eprintln!("The command {} returned the following error: {:?}", cmd_name, why);
    ///     }
    /// }
    ///
    /// let framework = SimpleFramework::new()
    ///                 .after(after_fn);
    /// ```
    pub fn after<F>(mut self, after: F) -> Self
    where F: for<'r, 's, 't>
    AsyncFn4<&'r Context, &'s Message, &'t str, CommandResult, Output = ()>
    + Send + Sync + 'static {
        self.after_cmd = Some(Box::new(move |ctx, msg, cmd_name, res|  Box::pin(after.call(ctx, msg, cmd_name, res))));
        self
    }

    /// Sets the function to run before each command
    /// it's passed the name of the command used and
    /// the returned boolean determines if the command
    /// should then be run
    pub fn before<F>(mut self, before: F) -> Self
    where F: for<'r, 's, 't>
    AsyncFn3<&'r Context, &'s Message, &'t str, Output = bool>
    + Send + Sync + 'static {
        self.before_cmd = Some(Box::new(move |ctx, msg, cmd_name| Box::pin(before.call(ctx, msg, cmd_name))));
        self
    }

    /// Sets the function to run if a prefex is detected
    /// but no known command was found
    pub fn unrecognized_cmd<F>(mut self, default_cmd: F) -> Self
    where F: for<'r, 's, 't> 
    AsyncFn4<&'r Context, &'s Message, &'t str, Args, Output = ()>
    + Send + Sync + 'static {
        self.default_cmd = Some(Box::new(move |ctx, msg, cmd_name, args| Box::pin(default_cmd.call(ctx, msg, cmd_name, args))));
        self
    }

    /// sets the function to run if only a prefix was given
    pub fn prefix_only<F>(mut self, prefix_only: F) -> Self
    where F: for<'a, 'b> AsyncFn2<&'a Context, &'b Message, Output = ()> + Send + Sync + 'static {
        self.prefix_only_cmd = Some(Box::new(move |ctx, msg| Box::pin(prefix_only.call(ctx, msg))));
        self
    }

    /// Sets the function to run if no prefix was found on a message
    pub fn normal_message<F>(mut self, normal_msg: F) -> Self
    where F: for<'a, 'b> AsyncFn2<&'a Context, &'b Message, Output = ()> + Send + Sync + 'static {
        self.normal_message_fn =  Some(Box::new(move |ctx, msg| Box::pin(normal_msg.call(ctx, msg))));
        self
    }

    /// Sets the delimiter between a command's name and any
    /// arguments it may have recieived, defaults to a space
    pub fn delimiter<T: Into<Delimiter>>(mut self, new_delimiter: T) -> Self {
        self.delimiters.clear();
        self.delimiters.push(new_delimiter.into());
        self
    }

    /// Default help sends a list of all command names
    pub fn with_default_help(self) -> Self {
        self.help(default_plaintext_help)
    }

    /// Sets the function to run if {prefix}help is detected
    /// the last argument is an array of all the command names
    pub fn help<F>(mut self, help_fn: F) -> Self
    where F: for<'r, 's, 't0, 't1, 't2>
    AsyncFn4<&'r Context, &'s Message, Args, &'t1[&'t2 str], Output = CommandResult>
    + Send + Sync + 'static {
        self.help_cmd = Some(Box::new(move |ctx, msg, args, command_list| Box::pin(help_fn.call(ctx, msg, args, command_list))));
        self
    }

    /// Stores all command names as lowercase strings, and converts to lowercase before checking for a command
    /// Note: This has the side-effect of converting all existing command names to lowercase when set to true
    pub fn case_insensitivity(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        if self.case_insensitive {
            let mut new_map = HashMap::new();
            let command_iter = self.commands.drain();
            for (k, v) in command_iter {
                new_map.insert(k.to_lowercase(), v);
            }
            self.commands = new_map;
        }
        self
    }

    async fn send_help(&self, ctx: &Context, msg: &Message, args: Args) {
        let help_cmd = self.help_cmd.as_ref().expect("Should not get here");
        let mut cmd_list = self.commands.keys().map(|name| name.as_ref()).collect::<Vec<&str>>();
        cmd_list.sort_unstable();
        if self.run_before_cmd(ctx, msg, "help").await {
            let res = help_cmd(ctx, msg, args, &cmd_list).await;
            self.run_after_cmd(ctx, msg, "help", res).await;
        }
    }

    async fn run_cmd(&self, ctx: &Context, msg: &Message, cmd_name: &str, args: Args) {

        if self.help_cmd.is_some() && cmd_name == "help" {
            self.send_help(ctx, msg, args).await;
            return;
        }

        if let Some(cmd) = self.commands.get(cmd_name) {
            if self.run_before_cmd(ctx, msg, cmd_name).await {
                let res = cmd(ctx, msg, args).await;
                self.run_after_cmd(ctx, msg, cmd_name, res).await;
            }
            // else unrecognized command
        } else if let Some(unrecongnized) = &self.default_cmd {
            unrecongnized(ctx, msg, cmd_name, args).await;
        } else if let Some(normal_message) = &self.normal_message_fn {
            // no unrecognized command fn set, so run the normal message cmd instead
            normal_message(ctx, msg).await;
        }
    }

    async fn run_before_cmd(&self, ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
        // returns true if the result of running the before fn is true, also returns true if there is no before fn set
        if let Some(before) = &self.before_cmd {
            before(ctx, msg, cmd_name).await
        } else {
            true
        }
    }

    // should be the following signature
    //`async fn run_after_cmd(&self, ctx: &Context, msg: &Message, cmd_name: &str, res: Command Result)`
    // however, see rustc issue #63033. For now, this is a valid workaround
    fn run_after_cmd<'a>(&'a self, ctx: &'a Context, msg: &'a Message, cmd_name: &'a str, res: CommandResult) -> impl std::future::Future<Output = ()> + 'a {
        // runs the after fn if it's set, no-op if there isn't one
        async move {
            if let Some(after) = &self.after_cmd {
                after(ctx, msg, cmd_name, res).await;
            }
        }
    }

    async fn run_prefix_only_cmd(&self, ctx: &Context, msg: &Message) {
        if let Some(prefix_only) = &self.prefix_only_cmd {
            prefix_only(ctx, msg).await;
            return;
        }
        // if no prefix only command is set, run normal message instead in this case
        if let Some(normal_message) = &self.normal_message_fn {
            normal_message(ctx, msg).await;
        }
    }

    async fn run_normal_message(&self, ctx: &Context, msg: &Message) {
        if let Some(normal_message) = &self.normal_message_fn {
            normal_message(ctx, msg).await;
        }
    }

    fn parse_cmd_name_and_args<'c>(&self, text: &'c str) -> (Cow<'c, str>, Args) {
        let text = match &self.delimiters[0] {
            Delimiter::Multiple(delim) => text.splitn(2, delim).collect::<Vec<&str>>(),
            Delimiter::Single(delim) => text.splitn(2, *delim).collect::<Vec<&str>>(),
        };
        
        let cmd = if self.case_insensitive {
            Cow::Owned(text[0].to_lowercase())
        } else {
            Cow::Borrowed(text[0])
        };

        // if no arguments were given, create an Args from an empty string
        let args = match text.get(1) {
            Some(val) => Args::new(val, &self.delimiters),
            None => Args::new("", &self.delimiters),
        };

        (cmd, args)
    }

}

#[async_trait]
impl Framework for SimpleFramework {
    async fn dispatch(&self, ctx: Context, msg: Message) {

        let text = msg.content.trim_start();

        if !text.starts_with(&self.prefix) {
            self.run_normal_message(&ctx, &msg).await;
            return;
        }

        let cmd_and_args = &text[self.prefix.len()..];

        if cmd_and_args.is_empty() {
            self.run_prefix_only_cmd(&ctx, &msg).await;
            return;
        }

        let (cmd, args) = self.parse_cmd_name_and_args(cmd_and_args);
        self.run_cmd(&ctx, &msg, &cmd, args).await;
    }
}

async fn default_plaintext_help(ctx: &Context, msg: &Message, _: Args, command_list: &[&str]) -> CommandResult {
    let help_text = format!(
        "Here is a list of commands:\n{}",
        command_list.join("\n")
    );
    
    if let Err(why) = msg.channel_id.say(ctx, help_text).await {
        warn!("Failed to send help message because: {:?}", why);
        return Err(why.into());
    }
    Ok(())
}