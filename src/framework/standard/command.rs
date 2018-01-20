use client::Context;
use model::channel::Message;
use model::Permissions;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use utils::Colour;
use super::{Args, Configuration, HelpBehaviour};

pub type Check = Fn(&mut Context, &Message, &mut Args, &CommandOptions) -> bool
                     + Send
                     + Sync
                     + 'static;

pub type HelpFunction = fn(&mut Context, &Message, &HelpOptions, HashMap<String, Arc<CommandGroup>>, &Args)
                   -> Result<(), Error>;

pub struct Help(pub HelpFunction, pub Arc<HelpOptions>);

impl Debug for Help {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "fn()")
    }
}

impl HelpCommand for Help {
    fn execute(&self, c: &mut Context, m: &Message, ho: &HelpOptions,hm: HashMap<String, Arc<CommandGroup>>, a: &Args) -> Result<(), Error> {
        (self.0)(c, m, ho, hm, a)
    }
}

pub type BeforeHook = Fn(&mut Context, &Message, &str) -> bool + Send + Sync + 'static;
pub type AfterHook = Fn(&mut Context, &Message, &str, Result<(), Error>) + Send + Sync + 'static;
pub(crate) type InternalCommand = Arc<Command>;
pub type PrefixCheck = Fn(&mut Context, &Message) -> Option<String> + Send + Sync + 'static;

pub enum CommandOrAlias {
    Alias(String),
    Command(InternalCommand),
}

impl fmt::Debug for CommandOrAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommandOrAlias::Alias(ref s) => f.debug_tuple("CommandOrAlias::Alias").field(&s).finish(),
            _ => Ok(())
        }
    }
}

/// An error from a command.
#[derive(Clone, Debug)]
pub struct Error(pub String);

// TODO: Have seperate `From<(&)String>` and `From<&str>` impls via specialization
impl<D: fmt::Display> From<D> for Error {
    fn from(d: D) -> Self {
        Error(d.to_string())
    }
}

#[derive(Debug)]
pub struct CommandGroup {
    pub prefix: Option<String>,
    pub commands: HashMap<String, CommandOrAlias>,
    /// Some fields taken from Command
    pub bucket: Option<String>,
    pub required_permissions: Permissions,
    pub allowed_roles: Vec<String>,
    pub help_available: bool,
    pub dm_only: bool,
    pub guild_only: bool,
    pub owners_only: bool,
    pub help: Option<Arc<Help>>,
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
            help: None,
        }
    }
}

pub struct CommandOptions {
    /// A set of checks to be called prior to executing the command. The checks
    /// will short-circuit on the first check that returns `false`.
    pub checks: Vec<Box<Check>>,
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
    /// Roles allowed to use this command.
    pub allowed_roles: Vec<String>,
    /// Whether command should be displayed in help list or not, used by other commands.
    pub help_available: bool,
    /// Whether command can be used only privately or not.
    pub dm_only: bool,
    /// Whether command can be used only in guilds or not.
    pub guild_only: bool,
    /// Whether command can only be used by owners or not.
    pub owners_only: bool,
    /// Other names that can be used to call this command instead.
    pub aliases: Vec<String>,
}

#[derive(Debug)]
pub struct HelpOptions {
    /// Suggests a command's name.
    pub suggestion_text: String,
    /// If no help is available, this text will be displayed.
    pub no_help_available_text: String,
    /// How to use a command, `{usage_label}: {command_name} {args}`
    pub usage_label: String,
    /// Actual sample label, `{usage_sample_label}: {command_name} {args}`
    pub usage_sample_label: String,
    /// Text labeling ungrouped commands, `{ungrouped_label}: ...`
    pub ungrouped_label: String,
    /// Text labeling the start of the description.
    pub description_label: String,
    /// Text labeling grouped commands, `{grouped_label} {group_name}: ...`
    pub grouped_label: String,
    /// Text labeling a command's alternative names (aliases).
    pub aliases_label: String,
    /// Text specifying that a command is only usable in a guild.
    pub guild_only_text: String,
    /// Text specifying that a command is only usable in via DM.
    pub dm_only_text: String,
    /// Text specifying that a command can be used via DM and in guilds.
    pub dm_and_guild_text: String,
    /// Text expressing that a command is available.
    pub available_text: String,
    /// Error-message once a command could not be found.
    /// Output-example (without whitespace between both substitutions: `{command_not_found_text}{command_name}`
    /// `{command_name}` describes user's input as in: `{prefix}help {command_name}`.
    pub command_not_found_text: String,
    /// Explains the user on how to use access a single command's details.
    pub individual_command_tip: String,
    /// Explains reasoning behind striked commands, see fields requiring `HelpBehaviour` for further information.
    /// If `HelpBehaviour::Strike` is unused, this field will evaluate to `None` during creation
    /// inside of `CreateHelpCommand`.
    pub striked_commands_tip: Option<String>,
    /// Announcing a group's prefix as in: {group_prefix} {prefix}.
    pub group_prefix: String,
    /// If a user lacks required roles, this will treat how these commands will be displayed.
    pub lacking_role: HelpBehaviour,
    /// If a user lacks permissions, this will treat how these commands will be displayed.
    pub lacking_permissions: HelpBehaviour,
    /// If a user is using the help-command in a channel where a command is not available,
    /// this behaviour will be executed.
    pub wrong_channel: HelpBehaviour,
    /// Colour help-embed will use upon encountering an error.
    pub embed_error_colour: Colour,
    /// Colour help-embed will use if no error occured.
    pub embed_success_colour: Colour,
}

pub trait HelpCommand: Send + Sync + 'static {
    fn execute(&self, &mut Context, &Message, &HelpOptions, HashMap<String, Arc<CommandGroup>>, &Args) -> Result<(), Error>;

    fn options(&self) -> Arc<CommandOptions> {
        Arc::clone(&DEFAULT_OPTIONS)
    }
}

impl HelpCommand for Arc<HelpCommand> {
    fn execute(&self, c: &mut Context, m: &Message, ho: &HelpOptions, hm: HashMap<String, Arc<CommandGroup>>, a: &Args) -> Result<(), Error> {
        (**self).execute(c, m, ho, hm, a)
    }
}

impl Default for HelpOptions {
    fn default() -> HelpOptions {
        HelpOptions {
            suggestion_text: "Did you mean {}?".to_string(),
            no_help_available_text: "**Error**: No help available.".to_string(),
            usage_label: "Usage".to_string(),
            usage_sample_label: "Sample usage".to_string(),
            ungrouped_label: "Ungrouped".to_string(),
            grouped_label: "Group".to_string(),
            aliases_label: "Aliases".to_string(),
            description_label: "Description".to_string(),
            guild_only_text: "Only in guilds".to_string(),
            dm_only_text: "Only in DM".to_string(),
            dm_and_guild_text: "In DM and guilds".to_string(),
            available_text: "Available".to_string(),
            command_not_found_text: "**Error**: Command `{}` not found.".to_string(),
            individual_command_tip: "To get help with an individual command, pass its \
                 name as an argument to this command.".to_string(),
            group_prefix: "Prefix".to_string(),
            striked_commands_tip: Some(String::new()),
            lacking_role: HelpBehaviour::Strike,
            lacking_permissions: HelpBehaviour::Strike,
            wrong_channel: HelpBehaviour::Strike,
            embed_error_colour: Colour::dark_red(),
            embed_success_colour: Colour::rosewater(),
        }
    }
}


lazy_static! {
    static ref DEFAULT_OPTIONS: Arc<CommandOptions> = Arc::new(CommandOptions::default());
}

/// A framework command.
pub trait Command: Send + Sync + 'static {
    fn execute(&self, &mut Context, &Message, Args) -> Result<(), Error>;

    fn options(&self) -> Arc<CommandOptions> {
        Arc::clone(&DEFAULT_OPTIONS)
    }

    /// Called when the command gets registered.
    fn init(&self) {}

    /// "before" middleware. Is called alongside the global middleware in the framework.
    fn before(&self, &mut Context, &Message) -> bool { true }

    /// "after" middleware. Is called alongside the global middleware in the framework.
    fn after(&self, &mut Context, &Message, &Result<(), Error>) { }
}

impl Command for Arc<Command> {
    fn execute(&self, c: &mut Context, m: &Message, a: Args) -> Result<(), Error> {
        (**self).execute(c, m, a)
    }

    fn options(&self) -> Arc<CommandOptions> {
        (**self).options()
    }

    fn init(&self) {
        (**self).init()
    }

    fn before(&self, c: &mut Context, m: &Message) -> bool {
        (**self).before(c, m)
    }

    fn after(&self, c: &mut Context, m: &Message, res: &Result<(), Error>) {
        (**self).after(c, m, res)
    }
}

impl<F> Command for F where F: Fn(&mut Context, &Message, Args) -> Result<(), Error>
    + Send
    + Sync
    + ?Sized
    + 'static {
    fn execute(&self, c: &mut Context, m: &Message, a: Args) -> Result<(), Error> {
        (*self)(c, m, a)
    }
}

impl fmt::Debug for CommandOptions {
    // TODO: add CommandOptions::checks somehow?
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("CommandOptions")
            .field("bucket", &self.bucket)
            .field("desc", &self.desc)
            .field("example", &self.example)
            .field("usage", &self.usage)
            .field("min_args", &self.min_args)
            .field("required_permissions", &self.required_permissions)
            .field("allowed_roles", &self.allowed_roles)
            .field("help_available", &self.help_available)
            .field("dm_only", &self.dm_only)
            .field("guild_only", &self.guild_only)
            .field("owners_only", &self.owners_only)
            .finish()
    }
}

impl Default for CommandOptions {
    fn default() -> CommandOptions {
        CommandOptions {
            aliases: Vec::new(),
            checks: Vec::default(),
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
            allowed_roles: Vec::new(),
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
            return Some(positions);
        } else if let Some(ref func) = conf.dynamic_prefix {
            if let Some(x) = func(ctx, msg) {
                if msg.content.starts_with(&x) {
                    positions.push(x.chars().count());
                }
            } else {
                for n in &conf.prefixes {
                    if msg.content.starts_with(n) {
                        positions.push(n.chars().count());
                    }
                }
            }
        } else {
            for n in &conf.prefixes {
                if msg.content.starts_with(n) {
                    positions.push(n.chars().count());
                }
            }
        };

        if positions.is_empty() {
            return None;
        }

        let pos = *unsafe { positions.get_unchecked(0) };

        if conf.allow_whitespace {
            positions.insert(0, find_end_of_prefix_with_whitespace(&msg.content, pos).unwrap_or(pos));
        } else if find_end_of_prefix_with_whitespace(&msg.content, pos).is_some() {
            return None;
        }

        Some(positions)
    } else if conf.on_mention.is_some() {
        find_mention_end(&msg.content, conf).map(|mention_end| {
            vec![mention_end] // This can simply be returned without trying to find the end whitespaces as trim will remove it later
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

// Finds the end of the first continuous block of whitespace after the prefix
fn find_end_of_prefix_with_whitespace(content: &str, position: usize) -> Option<usize> {
    let content_len = content.len();
    if position >= content_len { return None; }

    let mut i = 0;
    let chars = content.chars().skip(position);
    for char in chars {
        match char {
            // \t \n \r [space]
            '\t' | '\n' | '\r' | ' ' => i += 1,
            _ => return if i == 0 { None } else { Some(position + i) }
        }
    }
    Some(content.len())
}
