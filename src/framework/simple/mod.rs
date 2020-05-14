use super::Framework;
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;
use std::borrow::Cow;

use async_trait::async_trait;

use crate::client::Context;
use crate::model::channel::Message;

pub use super::shared::{
    args::{Args, Delimiter, Error as ArgError, Iter, RawArguments},
    CommandResult
};

type Command = Box<dyn Fn(&Context, &Message, Args) -> Pin<Box<dyn Future<Output = CommandResult> + Send + Sync>> + Send + Sync>;
type BeforeFn = Box<dyn Fn(&Context, &Message, &str) -> Pin<Box<dyn Future<Output = bool> + Send + Sync>> + Send + Sync>;
type AfterFn = Box<dyn Fn(&Context, &Message, &str, CommandResult) -> Pin<Box<dyn Future<Output = ()>  + Send + Sync>> + Send + Sync>;
type UnrecognizedCommand = Box<dyn Fn(&Context, &Message, &str, Args) -> Pin<Box<dyn Future<Output = ()>  + Send + Sync>> + Send + Sync>;
type NormalMessage = Box<dyn Fn(&Context, &Message) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>;
type PrefixOnly = Box<dyn Fn(&Context, &Message) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>;

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
        }
    }
}

impl SimpleFramework {
    pub fn new() -> SimpleFramework {
        SimpleFramework::default()
    }

    pub fn add<T, F>(mut self, name: &str, cmd: &'static F) -> Self 
    where T: 'static + Future<Output=CommandResult> + Send + Sync,
    F: Fn(&Context, &Message, Args) -> T + Send + Sync {
        self.commands.insert(name.to_owned(), Box::new(move |ctx, msg, args| Box::pin(cmd(ctx, msg, args))));
        self
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_owned();
        self
    }

    pub fn after<T, F>(mut self, after: &'static F) -> Self
    where T: 'static + Future<Output=()> + Send + Sync, 
    F: Fn(&Context, &Message, &str, CommandResult) -> T + Send + Sync {
        self.after_cmd = Some(Box::new(move |ctx, msg, cmd_name, res|  Box::pin(after(ctx, msg, cmd_name, res))));
        self
    }

    pub fn before<T, F>(mut self, before: &'static F) -> Self
    where T: 'static + Future<Output = bool> + Send + Sync,
    F: Fn(&Context, &Message, &str) -> T + Send + Sync {
        self.before_cmd = Some(Box::new(move |ctx, msg, cmd_name| Box::pin(before(ctx, msg, cmd_name))));
        self
    }

    pub fn unrecognized_cmd<T, F>(mut self, default_cmd: &'static F) -> Self
    where T: 'static + Future<Output = ()> + Send + Sync,
    F: Fn(&Context, &Message, &str, Args) -> T + Send + Sync {
        self.default_cmd = Some(Box::new(move |ctx, msg, cmd_name, args| Box::pin(default_cmd(ctx, msg, cmd_name, args))));
        self
    }

    pub fn prefix_only<T, F>(mut self, prefix_only: &'static F) -> Self
    where T: 'static + Future<Output = ()> + Send + Sync,
    F: Fn(&Context, &Message) -> T + Send + Sync {
        self.prefix_only_cmd = Some(Box::new(move |ctx, msg| Box::pin(prefix_only(ctx, msg))));
        self
    }

    pub fn normal_message<T, F>(mut self, normal_msg: &'static F) -> Self
    where T: 'static + Future<Output = ()> + Send + Sync,
    F: Fn(&Context, &Message) -> T + Send + Sync {
        self.normal_message_fn =  Some(Box::new(move |ctx, msg| Box::pin(normal_msg(ctx, msg))));
        self
    }

    pub fn delimiter<T: Into<Delimiter>>(mut self, new_delimiter: T) -> Self {
        self.delimiters.clear();
        self.delimiters.push(new_delimiter.into());
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

    async fn run_cmd(&self, ctx: &Context, msg: &Message, cmd_name: &str, args: Args) {
        if let Some(cmd) = self.commands.get(cmd_name) {
            if self.run_before_cmd(ctx, msg, cmd_name).await {
                let res = cmd(ctx, msg, args).await;
                self.run_after_cmd(ctx, msg, cmd_name, res).await;
            }
        } else if let Some(unrecongnized) = &self.default_cmd {
            unrecongnized(ctx, msg, cmd_name, args).await;
        }
    }

    /// returns true if the result of running the before fn is true, also returns true if there is no before fn set
    async fn run_before_cmd(&self, ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
        if let Some(before) = &self.before_cmd {
            before(ctx, msg, cmd_name).await
        } else {
            true
        }
    }

    /// runs the after fn if it's set, no-op if there isn't one
    async fn run_after_cmd(&self, ctx: &Context, msg: &Message, cmd_name: &str, res: CommandResult) {
        if let Some(after) = &self.after_cmd {
            after(ctx, msg, cmd_name, res).await;
        }
    }

    async fn run_prefix_only_cmd(&self, ctx: &Context, msg: &Message) {
        if let Some(prefix_only) = &self.prefix_only_cmd {
            prefix_only(ctx, msg).await;
        }
    }

    async fn run_normal_message(&self, ctx: &Context, msg: &Message) {
        if let Some(normal_message) = &self.normal_message_fn {
            normal_message(ctx, msg).await;
        }
    }


    fn parse_cmd_name_and_args<'a>(&self, text: &'a str) -> (Cow<'a, str>, Args) {
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
        } else {
            let cmd_and_args = &text[self.prefix.len()..];
            if cmd_and_args.is_empty() {
                self.run_prefix_only_cmd(&ctx, &msg).await;
            } else {
                let (cmd, args) = self.parse_cmd_name_and_args(cmd_and_args);
                self.run_cmd(&ctx, &msg, &cmd, args).await;
            }
        }
    }
}