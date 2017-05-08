use std::sync::Arc;
use super::Configuration;
use ::client::Context;
use ::model::Message;
use ::model::Permissions;
use std::collections::HashMap;

pub type Check = Fn(&mut Context, &Message) -> bool + Send + Sync + 'static;
pub type Exec = Fn(&mut Context, &Message, Vec<String>) -> Result<(), String> + Send + Sync + 'static;
pub type Help = Fn(&mut Context, &Message, HashMap<String, Arc<CommandGroup>>, &[String]) -> Result<(), String> + Send + Sync + 'static;
pub type BeforeHook = Fn(&mut Context, &Message, &String) -> bool + Send + Sync + 'static;
pub type AfterHook = Fn(&mut Context, &Message, &String, Result<(), String>) + Send + Sync + 'static;
#[doc(hidden)]
pub type InternalCommand = Arc<Command>;
pub type PrefixCheck = Fn(&mut Context) -> Option<String> + Send + Sync + 'static;

#[doc(hidden)]
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
    /// Whether arguments should be parsed using quote parser or not.
    pub use_quotes: bool,
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
    #[doc(hidden)]
    pub aliases: Vec<String>,
}

impl Command {
    pub fn new<F>(f: F) -> Self
        where F: Fn(&mut Context, &Message, Vec<String>) -> Result<(), String> + Send + Sync + 'static {
        Command {
            aliases: Vec::new(),
            checks: Vec::default(),
            exec: CommandType::Basic(Box::new(f)),
            desc: None,
            usage: None,
            example: None,
            use_quotes: false,
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

pub fn positions(ctx: &mut Context, content: &str, conf: &Configuration) -> Option<Vec<usize>> {
    if !conf.prefixes.is_empty() || conf.dynamic_prefix.is_some() {
        // Find out if they were mentioned. If not, determine if the prefix
        // was used. If not, return None.
        let mut positions: Vec<usize> = vec![];

        if let Some(mention_end) = find_mention_end(content, conf) {
            positions.push(mention_end);
        } else if let Some(ref func) = conf.dynamic_prefix {
            if let Some(x) = func(ctx) {
                if content.starts_with(&x) {
                    positions.push(x.len());
                }
            } else {
                for n in conf.prefixes.clone() {
                    if content.starts_with(&n) {
                        positions.push(n.len());
                    }
                }
            }
        } else {
            for n in conf.prefixes.clone() {
                if content.starts_with(&n) {
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
        match find_mention_end(content, conf) {
            Some(mention_end) => {
                let mut positions = vec![mention_end];

                if conf.allow_whitespace {
                    positions.insert(0, mention_end + 1);
                }

                Some(positions)
            },
            None => None,
        }
    } else {
        None
    }
}

fn find_mention_end(content: &str, conf: &Configuration) -> Option<usize> {
    if let Some(ref mentions) = conf.on_mention {
        for mention in mentions {
            if !content.starts_with(&mention[..]) {
                continue;
            }

            return Some(mention.len());
        }
    }

    None
}
