use std::sync::Arc;
use super::{Args, Configuration};
use client::Context;
use model::{Message, Permissions};
use std::collections::HashMap;

pub type Check = Fn(&mut Context, &Message, &mut Args, &Arc<Command>) -> bool + 'static;
pub type Exec = Fn(&mut Context, &Message, Args) -> Result<(), String> + 'static;
pub type Help = Fn(&mut Context,
                   &Message,
                   HashMap<String, Arc<CommandGroup>>,
                   Args)
                   -> Result<(), String>
                    + 'static;
pub type BeforeHook = Fn(&mut Context, &Message, &str) -> bool + 'static;
pub type AfterHook = Fn(&mut Context, &Message, &str, Result<(), String>) + 'static;
pub(crate) type InternalCommand = Arc<Command>;
pub type PrefixCheck = Fn(&mut Context, &Message) -> Option<String> + 'static;

pub enum CommandOrAlias {
    Alias(String),
    Command(InternalCommand),
}

/// Command function type. Allows to access internal framework things inside
/// your commands.
pub enum CommandType {
    StringResponse(String),
    Basic(Box<Exec>),
    WithCommands(Box<Help>),
}

#[derive(Default)]
pub struct CommandGroup {
    pub prefix: Option<String>,
    pub commands: HashMap<String, CommandOrAlias>,
}

/// Command struct used to store commands internally.
pub struct Command {
    /// A set of checks to be called prior to executing the command. The checks
    /// will short-circuit on the first check that returns `false`.
    pub checks: Vec<Box<Check>>,
    /// Function called when the command is called.
    pub exec: CommandType,
    /// Ratelimit bucket.
    pub bucket: Option<String>,
    /// Command description, used by other commands.
    pub desc: Option<String>,
    /// Example arguments, used by other commands.
    pub example: Option<String>,
    /// Command usage schema, used by other commands.
    pub usage: Option<String>,
    /// Minumum amount of arguments that should be passed.
    pub min_args: Option<i32>,
    /// Maximum amount of arguments that can be passed.
    pub max_args: Option<i32>,
    /// Permissions required to use this command.
    pub required_permissions: Permissions,
    /// Whether command should be displayed in help list or not, used by other commands.
    pub help_available: bool,
    /// Whether command can be used only privately or not.
    pub dm_only: bool,
    /// Whether command can be used only in guilds or not.
    pub guild_only: bool,
    /// Whether command can only be used by owners or not.
    pub owners_only: bool,
    pub(crate) aliases: Vec<String>,
}

impl Command {
    pub fn new<F>(f: F) -> Self
        where F: Fn(&mut Context, &Message, Args) -> Result<(), String> + 'static {
        Command {
            aliases: Vec::new(),
            checks: Vec::default(),
            exec: CommandType::Basic(Box::new(f)),
            desc: None,
            usage: None,
            example: None,
            dm_only: false,
            bucket: None,
            guild_only: false,
            help_available: true,
            min_args: None,
            max_args: None,
            owners_only: false,
            required_permissions: Permissions::empty(),
        }
    }
}

pub fn positions(ctx: &mut Context, msg: &Message, conf: &Configuration) -> Option<Vec<usize>> {
    if !conf.prefixes.is_empty() || conf.dynamic_prefix.is_some() {
        // Find out if they were mentioned. If not, determine if the prefix
        // was used. If not, return None.
        let mut positions: Vec<usize> = vec![];

        if let Some(mention_end) = find_mention_end(&msg.content, conf) {
            positions.push(mention_end);
        } else if let Some(ref func) = conf.dynamic_prefix {
            if let Some(x) = func(ctx, msg) {
                if msg.content.starts_with(&x) {
                    positions.push(x.len());
                }
            } else {
                for n in &conf.prefixes {
                    if msg.content.starts_with(n) {
                        positions.push(n.len());
                    }
                }
            }
        } else {
            for n in &conf.prefixes {
                if msg.content.starts_with(n) {
                    positions.push(n.len());
                }
            }
        };

        if positions.is_empty() {
            return None;
        }

        if conf.allow_whitespace {
            let pos = *unsafe { positions.get_unchecked(0) };

            positions.insert(0, pos + 1);
        }

        Some(positions)
    } else if conf.on_mention.is_some() {
        find_mention_end(&msg.content, conf).map(|mention_end| {
            let mut positions = vec![mention_end];

            if conf.allow_whitespace {
                positions.insert(0, mention_end + 1);
            }

            positions
        })
    } else {
        None
    }
}

fn find_mention_end(content: &str, conf: &Configuration) -> Option<usize> {
    conf.on_mention.as_ref().and_then(|mentions| {
        mentions
            .iter()
            .find(|mention| content.starts_with(&mention[..]))
            .map(|m| m.len())
    })
}
