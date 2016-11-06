mod command;
mod configuration;

pub use self::command::Command;
pub use self::configuration::Configuration;

use self::command::InternalCommand;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use ::client::Context;
use ::model::Message;

#[allow(type_complexity)]
#[derive(Default)]
pub struct Framework {
    configuration: Configuration,
    commands: HashMap<String, InternalCommand>,
    checks: HashMap<String, Arc<Fn(&Context, &Message) -> bool + Send + Sync + 'static>>,
    pub initialized: bool,
}

impl Framework {
    pub fn configure<F>(mut self, f: F) -> Self
        where F: FnOnce(Configuration) -> Configuration {
        self.configuration = f(self.configuration);
        self.initialized = true;

        self
    }

    #[doc(hidden)]
    pub fn dispatch(&mut self, context: Context, message: Message) {
        // Determine the point at which the prefix ends, and the command starts.
        let pos = if let Some(ref prefix) = self.configuration.prefix {
            if let Some(mention_ends) = self.find_mention_end(&message.content) {
                mention_ends
            } else if !message.content.starts_with(prefix) {
                return;
            } else {
                prefix.len()
            }
        } else if self.configuration.on_mention.is_some() {
            match self.find_mention_end(&message.content) {
                Some(mention_end) => mention_end,
                None => return,
            }
        } else {
            0
        };

        // Ensure that the message length is at least longer than the prefix
        // length. There's no point in checking further ahead if there's nothing
        // to check.
        if message.content.len() <= pos {
            return;
        }

        let mut built = String::new();

        for i in 0..self.configuration.depth {
            if i > 0 {
                built.push(' ');
            }

            built.push_str(match {
                message.content
                    .split_at(pos)
                    .1
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .get(i)
            } {
                Some(piece) => piece,
                None => return,
            });

            if let Some(command) = self.commands.get(&built) {
                if let Some(check) = self.checks.get(&built) {
                    if !(check)(&context, &message) {
                        return;
                    }
                }

                let command = command.clone();

                thread::spawn(move || {
                    (command)(context, message)
                });

                return;
            }
        }
    }

    pub fn on<F, S>(mut self, command_name: S, f: F) -> Self
        where F: Fn(Context, Message) + Send + Sync + 'static,
              S: Into<String> {
        self.commands.insert(command_name.into(), Arc::new(f));
        self.initialized = true;

        self
    }

    pub fn set_check<F, S>(mut self, command: S, check: F) -> Self
        where F: Fn(&Context, &Message) -> bool + Send + Sync + 'static,
              S: Into<String> {
        self.checks.insert(command.into(), Arc::new(check));
        self.initialized = true;

        self
    }

    fn find_mention_end(&self, content: &str) -> Option<usize> {
        if let Some(ref mentions) = self.configuration.on_mention {
            for mention in mentions {
                if !content.starts_with(&mention[..]) {
                    continue;
                }

                return Some(mention.len() + 1);
            }
        }

        None
    }
}
