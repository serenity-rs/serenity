use std::sync::Arc;
use super::{Configuration, Bucket};
use ::client::Context;
use ::model::{Message, Permissions};
use std::collections::HashMap;

pub type Check = Fn(&mut Context, &Message) -> bool + Send + Sync + 'static;
pub type PrefixCheck = Fn(&mut Context) -> Option<String> + Send + Sync + 'static;
pub type BeforeHook = Fn(&mut Context, &Message, &String) -> bool + Send + Sync + 'static;
pub type AfterHook = Fn(&mut Context, &Message, &String, Result<(), String>) + Send + Sync + 'static;

#[derive(Default)]
pub struct CommandGroup<T: Command> {
    pub commands: HashMap<String, T>,
    pub prefix: String,
}

impl<T: Command> CommandGroup<T> {
    fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, command: T) {
        self.commands.insert(command.name(), command);
    }

    fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix;
    }
}

/// A utility macro for named args.
/// Meant to be used with the `serenity_attributes` crate.
#[macro_export]
macro_rules! named_args {
    ($args_name:ident, $($name:ident:$type:ty;)*) => {
        let mut i = $args_name.iter();
        let mut arg_counter = 0;

        $(
            arg_counter += 1;

            let $name = match i.next() {
                Some(v) => match v.parse::<$type>() {
                    Ok(v) => v,
                    Err(_) => return Err(format!("Failed to parse argument #{} of type {:?}",
                                                     arg_counter,
                                                     stringify!($t))),
                    },
                None => return Err(format!("Missing argument #{} of type {:?}",
                                               arg_counter,
                                               stringify!($t))),
            };
        )*
        drop(i);
    }
}

/// The trait that defines the whole functionality (or the main part) of the framework.
pub trait Command {
    #[doc(hide)]
    fn name(&self) -> String { "".to_string() }
    /// The group this commands belongs to.
    fn group(&self) -> String { "Ungrouped".to_string() }
    /// The prefix for the group, e.g `~"abc" command_name`.
    fn group_prefix(&self) -> String { "".to_string() }
    /// A set of checks to be called prior to executing the command. The checks
    /// will short-circuit on the first check that returns `false`.
    fn checks(&self) -> Vec<Box<Check>> { Vec::new() }
    /// Function called when the command is called.
    fn exec(&self, _: &mut Context, _: &Message, _: Vec<String>) -> Result<(), String> { Ok(()) }
    /// Ratelimit bucket.
    fn bucket(&self) -> Option<Bucket> { None }
    /// Command description, used by other commands.
    fn desc(&self) -> Option<String> { None }
    /// Example arguments, used by other commands.
    fn example(&self) -> Option<String> { None }
    /// Command usage schema, used by other commands.
    fn usage(&self) -> Option<String> { None }
    /// Whether arguments should be parsed using quote parser or not.
    fn use_quotes(&self) -> bool { false }
    /// Minumum amount of arguments that should be passed.
    fn min_args(&self) -> Option<i32> { None }
    /// Maximum amount of arguments that can be passed.
    fn max_args(&self) -> Option<i32> { None }
    /// Permissions required to use this command.
    fn required_permissions(&self) -> Permissions { Permissions::empty() }
    /// Whether command should be displayed in help list or not, used by other commands.
    fn help_available(&self) -> bool { true }
    /// Whether command can be used only privately or not.
    fn dm_only(&self) -> bool { false }
    /// Whether command can be used only in guilds or not.
    fn guild_only(&self) -> bool { false }
    /// Whether command can only be used by owners or not.
    fn owners_only(&self) -> bool { false }
    /// Other names that can be used to call this command.
    fn aliases(&self) -> Vec<String> { Vec::new() }
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
        find_mention_end(content, conf).map(|mention_end| {
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
