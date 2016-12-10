use std::sync::Arc;
use super::Configuration;
use ::client::Context;
use ::model::Message;
use std::collections::HashMap;

pub type Check = Fn(&Context, &Message) -> bool + Send + Sync + 'static;
pub type Exec = Fn(&Context, &Message, Vec<String>) + Send + Sync + 'static;
pub type Help = Fn(&Context, &Message, HashMap<String, Arc<Command>>, Vec<String>) + Send + Sync + 'static;
pub type Hook = Fn(&Context, &Message, &String) + Send + Sync + 'static;
#[doc(hidden)]
pub type InternalCommand = Arc<Command>;
pub type PrefixCheck = Fn(&Context) -> Option<String> + Send + Sync + 'static;

/// Command function type. Allows to access internal framework things inside
/// your commands.
pub enum CommandType {
    StringResponse(String),
    Basic(Box<Exec>),
    WithCommands(Box<Help>),
}

/// Command struct used to store commands internally.
pub struct Command {
    /// A set of checks to be called prior to executing the command. The checks
    /// will short-circuit on the first check that returns `false`.
    pub checks: Vec<Box<Check>>,
    /// Function called when the command is called.
    pub exec: CommandType,
    /// Command description, used by other commands.
    pub desc: Option<String>,
    /// Command usage schema, used by other commands.
    pub usage: Option<String>,
    /// Whether arguments should be parsed using quote parser or not.
    pub use_quotes: bool,
}

pub fn positions(ctx: &Context, content: &str, conf: &Configuration) -> Option<Vec<usize>> {
    if !conf.prefixes.is_empty() || conf.dynamic_prefix.is_some() {
        // Find out if they were mentioned. If not, determine if the prefix
        // was used. If not, return None.
        let mut positions: Vec<usize> = vec![];

        if let Some(mention_end) = find_mention_end(content, conf) {
            positions.push(mention_end);
        } else if let Some(ref func) = conf.dynamic_prefix {
            if let Some(x) = func(ctx) {
                positions.push(x.len());
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
