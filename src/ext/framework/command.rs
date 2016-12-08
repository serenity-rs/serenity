use std::sync::Arc;
use super::Configuration;
use ::client::Context;
use ::model::Message;

/// Command function type. In future this will allow us to pass additional
/// internal information to commands.
pub enum CommandType {
    Basic(Box<Fn(&Context, &Message, Vec<String>) + Send + Sync + 'static>)
    // Todo: WithHelp(Fn(&Context, &Message, HashMap<String, Command>, Vec<String>))
}

/// Command struct used to store commands internally.
pub struct Command {
    /// Function called when the command is called.
    pub exec: CommandType,
    /// Command description, used by other commands.
    pub desc: Option<String>,
    /// Command usage schema, used by other commands.
    pub usage: Option<String>,
    /// Whether arguments should be parsed using quote parser or not.
    pub use_quotes: bool
}

#[doc(hidden)]
pub type InternalCommand = Arc<Command>;

pub fn positions(content: &str, conf: &Configuration) -> Option<Vec<usize>> {
    if let Some(ref prefix) = conf.prefix {
        // Find out if they were mentioned. If not, determine if the prefix
        // was used. If not, return None.
        let mut positions = if let Some(mention_end) = find_mention_end(content, conf) {
            vec![mention_end]
        } else if content.starts_with(prefix) {
            vec![prefix.len()]
        } else {
            return None;
        };

        if conf.allow_whitespace {
            let pos = *unsafe {
                positions.get_unchecked(0)
            };

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
