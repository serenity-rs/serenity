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

#[derive(Clone, Copy, Debug)]
pub enum CommandType {
    Mention,
    None,
    Prefix,
}

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
        let res = command::positions(&message.content, &self.configuration);

        let positions = match res {
            Some((positions, _kind)) => positions,
            None => return,
        };

        // Ensure that the message length is at least longer than a prefix
        // length. There's no point in checking further ahead if there's nothing
        // _to_ check.
        if positions.iter().all(|p| message.content.len() <= *p) {
            return;
        }

        for position in positions {
            let mut built = String::new();

            for i in 0..self.configuration.depth {
                if i != 0 {
                    built.push(' ');
                }

                built.push_str(match {
                    message.content
                        .split_at(position)
                        .1
                        .trim()
                        .split_whitespace()
                        .collect::<Vec<&str>>()
                        .get(i)
                } {
                    Some(piece) => piece,
                    None => continue,
                });

                if let Some(command) = self.commands.get(&built) {
                    if let Some(check) = self.checks.get(&built) {
                        if !(check)(&context, &message) {
                            continue;
                        }
                    }

                    let command = command.clone();

                    thread::spawn(move || {
                        let args = message.content[position + built.len()..]
                            .split_whitespace()
                            .map(|arg| arg.to_owned())
                            .collect::<Vec<String>>();

                        (command)(context, message, args)
                    });

                    return;
                }
            }
        }
    }

    pub fn on<F, S>(mut self, command_name: S, f: F) -> Self
        where F: Fn(Context, Message, Vec<String>) + Send + Sync + 'static,
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
}
