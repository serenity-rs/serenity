use super::Framework;
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;
use std::borrow::Cow;
use async_trait::async_trait;
use log::warn;
use crate::client::Context;
use crate::model::channel::Message;

pub use super::shared::{
    args::{Args, Delimiter, Error as ArgError, Iter, RawArguments},
    CommandResult
};

type Command = Box<dyn Fn(&Context, &Message, Args) -> Pin<Box<dyn Future<Output = CommandResult> + Send>> + Send + Sync>;
type BeforeFn = Box<dyn Fn(&Context, &Message, &str) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>;
type AfterFn = Box<dyn Fn(&Context, &Message, &str, CommandResult) -> Pin<Box<dyn Future<Output = ()>  + Send>> + Send + Sync>;
type UnrecognizedCommand = Box<dyn Fn(&Context, &Message, &str, Args) -> Pin<Box<dyn Future<Output = ()>  + Send>> + Send + Sync>;
type NormalMessage = Box<dyn Fn(&Context, &Message) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
type PrefixOnly = Box<dyn Fn(&Context, &Message) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
type HelpCommand = Box<dyn Fn(&Context, &Message, &str, &[&str]) -> Pin<Box<dyn Future<Output = CommandResult> + Send>> + Send + Sync>;

pub struct SimpleFramework {
    prefix: String,
    commands: HashMap<String, Command>,
    //a Vec Only to interface with the Args struct easier
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
    /// # use serenity::prelude::*;
    /// # use serenity::client::{Client, Context};
    /// # use serenity::framework::simple::{SimpleFramework, Args, CommandResult};
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
    pub fn add<T, F>(mut self, name: &str, cmd: &'static F) -> Self 
    where T: 'static + Future<Output=CommandResult> + Send,
    F: Fn(&Context, &Message, Args) -> T + Send + Sync {
        self.commands.insert(name.to_owned(), Box::new(move |ctx, msg, args| Box::pin(cmd(ctx, msg, args))));
        self
    }

    /// sets the prefix which the simple framework will look for
    /// defaults to '!'
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_owned();
        self
    }

    /// Sets the function to run after each command
    /// it's passed the name of the command used and
    /// the `CommandResult` returned by the command
    pub fn after<T, F>(mut self, after: &'static F) -> Self
    where T: 'static + Future<Output=()> + Send, 
    F: Fn(&Context, &Message, &str, CommandResult) -> T + Send + Sync {
        self.after_cmd = Some(Box::new(move |ctx, msg, cmd_name, res|  Box::pin(after(ctx, msg, cmd_name, res))));
        self
    }

    /// Sets the function to run before each command
    /// it's passed teh name of the command used and
    /// the returned boolean determines if the command
    /// should then be run
    pub fn before<T, F>(mut self, before: &'static F) -> Self
    where T: 'static + Future<Output = bool> + Send,
    F: Fn(&Context, &Message, &str) -> T + Send + Sync {
        self.before_cmd = Some(Box::new(move |ctx, msg, cmd_name| Box::pin(before(ctx, msg, cmd_name))));
        self
    }

    /// sets the function to run if a prefex is detected
    /// but no known command was found
    pub fn unrecognized_cmd<T, F>(mut self, default_cmd: &'static F) -> Self
    where T: 'static + Future<Output = ()> + Send,
    F: Fn(&Context, &Message, &str, Args) -> T + Send + Sync {
        self.default_cmd = Some(Box::new(move |ctx, msg, cmd_name, args| Box::pin(default_cmd(ctx, msg, cmd_name, args))));
        self
    }

    /// sets the function to run if only a prefix was given
    pub fn prefix_only<T, F>(mut self, prefix_only: &'static F) -> Self
    where T: 'static + Future<Output = ()> + Send,
    F: Fn(&Context, &Message) -> T + Send + Sync {
        self.prefix_only_cmd = Some(Box::new(move |ctx, msg| Box::pin(prefix_only(ctx, msg))));
        self
    }

    /// sets the function to run if no prefix was found on a message
    pub fn normal_message<T, F>(mut self, normal_msg: &'static F) -> Self
    where T: 'static + Future<Output = ()> + Send,
    F: Fn(&Context, &Message) -> T + Send + Sync {
        self.normal_message_fn =  Some(Box::new(move |ctx, msg| Box::pin(normal_msg(ctx, msg))));
        self
    }

    /// sets the delimiter between a command's name and any
    /// arguments it may have recieived, defaults to a space
    pub fn delimiter<T: Into<Delimiter>>(mut self, new_delimiter: T) -> Self {
        self.delimiters.clear();
        self.delimiters.push(new_delimiter.into());
        self
    }

    pub fn with_default_plaintext_help(mut self) -> Self {
        self.help(&default_plaintext_help)
    }

    pub fn help<T, F>(mut self, help_fn: &'static F) -> Self
    where T: 'static + Future<Output = CommandResult> + Send,
    F: Fn(&Context, &Message, &str, &[&str]) -> T + Send + Sync {
        self.help_cmd = Some(Box::new(|ctx, msg, prefix, command_list| Box::pin(help_fn(ctx, msg, prefix, command_list))));
        self
    }

    /// Stores all command names as lowercase strings, and converts to lowercase before checking for a command
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

    async fn send_help(&self, ctx: &Context, msg: &Message) {
        let help_cmd = self.help_cmd.as_ref().expect("Should not get here");
        let mut cmd_list = self.commands.keys().map(|name| name.as_ref()).collect::<Vec<&str>>();
        cmd_list.sort_unstable();
        if self.run_before_cmd(ctx, msg, "help").await {
            let res = help_cmd(ctx, msg, &self.prefix, &cmd_list).await;
            self.run_after_cmd(ctx, msg, "help", res).await;
        }
    }

    async fn run_cmd(&self, ctx: &Context, msg: &Message, cmd_name: &str, args: Args) {

        if self.help_cmd.is_some() && cmd_name == "help" {
            self.send_help(ctx, msg).await;
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

    async fn run_after_cmd(&self, ctx: &Context, msg: &Message, cmd_name: &str, res: CommandResult) {
        // runs the after fn if it's set, no-op if there isn't one
        if let Some(after) = &self.after_cmd {
            after(ctx, msg, cmd_name, res).await;
        }
    }

    async fn run_prefix_only_cmd(&self, ctx: &Context, msg: &Message) {
        if let Some(prefix_only) = &self.prefix_only_cmd {
            prefix_only(ctx, msg).await;
            return;
        }
        //if no prefix only command is set, run normal message instead in this case
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
        let args = Args::new(text[1], &self.delimiters);
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

async fn default_plaintext_help(ctx: &Context, msg: &Message, prefix: &str, command_list: &[&str]) -> CommandResult {
    let mut help_text = String::from("Here is a list of commands:\n");
    help_text = command_list.iter().fold(
        help_text, 
        |mut text, cmd| {
            text.push_str(prefix);
            text.push_str(cmd);
            text.push('\n');
            text
        });
    if let Err(why) = msg.channel_id.say(ctx, &help_text).await {
        warn!("Failed to send help message because: {:?}", why);
    }
    Ok(())
}