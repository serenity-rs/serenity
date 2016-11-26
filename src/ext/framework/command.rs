use std::sync::Arc;
use super::{CommandType, Configuration};
use ::client::Context;
use ::model::Message;

#[doc(hidden)]
pub type Command = Fn(Context, Message, Vec<String>) + Send + Sync;
#[doc(hidden)]
pub type InternalCommand = Arc<Command>;

pub fn positions(content: &str, conf: &Configuration)
    -> Option<(Vec<usize>, CommandType)> {
    if let Some(ref prefix) = conf.prefix {
        // Find out if they were mentioned. If not, determine if the prefix
        // was used. If not, return None.
        let (mut positions, kind) = if let Some(mention_end) = find_mention_end(content, conf) {
            (vec![mention_end], CommandType::Mention)
        } else if content.starts_with(prefix) {
            (vec![prefix.len()], CommandType::Prefix)
        } else {
            return None;
        };

        if conf.allow_whitespace {
            let pos = *unsafe {
                positions.get_unchecked(0)
            };

            positions.insert(0, pos + 1);
        }

        Some((positions, kind))
    } else if conf.on_mention.is_some() {
        match find_mention_end(content, conf) {
            Some(mention_end) => {
                let mut positions = vec![mention_end];

                if conf.allow_whitespace {
                    positions.insert(0, mention_end + 1);
                }

                Some((positions, CommandType::Mention))
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
