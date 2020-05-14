use super::Framework;
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;

use async_trait::async_trait;

use crate::client::Context;
use crate::model::channel::Message;




pub struct CommandError(pub String);
impl<T: std::fmt::Display> From<T> for CommandError {
    #[inline]
    fn from(d: T) -> Self {
        CommandError(d.to_string())
    }
}

type CommandResult = std::result::Result<(), CommandError>;

type Command = Box<dyn Fn(&Context, &Message, &[&str]) -> Pin<Box<dyn Future<Output = CommandResult> + Send + Sync>> + Send + Sync>;
type BeforeFn = Box<dyn Fn(&Context, &Message, &str) -> Pin<Box<dyn Future<Output = bool> + Send + Sync>> + Send + Sync>;
type AfterFn = Box<dyn Fn(&Context, &Message, &str, CommandResult) -> Pin<Box<dyn Future<Output = ()>  + Send + Sync>> + Send + Sync>;
type UnrecognizedCommand = Box<dyn Fn(&Context, &Message, &str, &[&str]) -> Pin<Box<dyn Future<Output = ()>  + Send + Sync>> + Send + Sync>;
type NormalMessage = Box<dyn Fn(&Context, &Message) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>;
type PrefixOnly = Box<dyn Fn(&Context, &Message) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>;

pub struct SimpleFramework {
    prefix: String,
    commands: HashMap<String, Command>,
    delimiter: String,
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
            delimiter: String::from(" "),
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
    F: Fn(&Context, &Message, &[&str]) -> T + Send + Sync {
        self.commands.insert(name.to_owned(), Box::new(move |ctx, msg, args| Box::pin(cmd(ctx, msg, args))));
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
    F: Fn(&Context, &Message, &str, &[&str]) -> T + Send + Sync {
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

    pub fn delimiter(mut self, new_delimiter: &str) -> Self {
        self.delimiter = new_delimiter.to_owned();
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

    async fn run_cmd(&self, ctx: &Context, msg: &Message, cmd_name: &str, args: &[&str]) {
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

}

#[async_trait]
impl Framework for SimpleFramework {
    async fn dispatch(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with(&self.prefix) {            
            let cmd_and_args = msg.content.splitn(2, &self.prefix).collect::<Vec<&str>>();
            if cmd_and_args[1].is_empty() {

                if let Some(prefix_only) = &self.prefix_only_cmd {
                    prefix_only(&ctx, &msg).await;
                    return;
                }

            } else {
                let tokens = cmd_and_args[1].split(&self.delimiter).collect::<Vec<&str>>();
                
                let cmd_name = if self.case_insensitive {
                    tokens[0].to_lowercase()
                } else {
                    tokens[0].to_owned()
                };
                self.run_cmd(&ctx, &msg, &cmd_name, &tokens[1..]).await;

            }

        } else if let Some(normal_msg_fn) = &self.normal_message_fn {
            normal_msg_fn(&ctx, &msg).await;
        }
        
    }
}