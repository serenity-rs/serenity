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

#[macro_export]
macro_rules! command {
    ($fname:ident($c:ident, $m:ident, $a:ident, $($name:ident: $t:ty),*) $b:block) => {
        fn $fname($c: Context, $m: Message, $a: Vec<String>) {
            let mut i = $a.iter();

            $(
                let $name = match i.next() {
                    Some(v) => match v.parse::<$t>() {
                        Ok(v) => v,
                        Err(_why) => return,
                    },
                    None => return,
                };
            )*

            drop(i);

            $b
        }
    }
}

/// The type of command being received.
///
/// The [`Mention`] variant is emitted if the bot is being commanded via a
/// mention (`<@USER_ID>` or `<@!USER_ID>`). This can only be emitted if
/// [`Configuration::on_mention`] is set to `true`.
///
/// The [`Prefix`] variant is emitted if a message starts with the prefix set
/// via [`Configuration::prefix`].
///
/// [`Mention`]: #variant.Mention
/// [`Prefix`]: #variant.Prefix
// This is public due to being leaked by [`command::positions`], which is used
// in [`Framework::dispatch`]. It therefore is hidden from the docs, due to
// having no use to users.
//
// [`Framework::dispatch`]: struct.Framework.html#method.dispatch
// [`command::positions`]: command/fn.positions.html
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub enum CommandType {
    /// This is emitted if the bot is being commanded via a mention
    /// (`<@USER_ID>` or `<@!USER_ID>`). This can only be emitted if
    /// [`Configuration::on_mention`] is set to `true`.
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
    /// Whether the framework has been "initialized".
    ///
    /// The framework is initialized once one of the following occurs:
    ///
    /// - configuration has been set;
    /// - a command handler has been set;
    /// - a command check has been set.
    ///
    /// This is used internally to determine whether or not - in addition to
    /// dispatching to the [`Client::on_message`] handler - to have the
    /// framework check if a [`Event::MessageCreate`] should be processed by
    /// itself.
    ///
    /// [`Client::on_message`]: ../../client/struct.Client.html#method.on_message
    /// [`Event::MessageCreate`]: ../../model/event/enum.Event.html#variant.MessageCreate
    pub initialized: bool,
}

impl Framework {
    /// Configures the framework, setting non-default values. All fields are
    /// optional. Refer to [`Configuration::default`] for more information on
    /// the default values.
    ///
    /// # Examples
    ///
    /// Configuring the framework for a [`Client`], setting the [`depth`] to 3,
    /// [allowing whitespace], and setting the [`prefix`] to `"~"`:
    ///
    /// ```rust,no_run
    /// use serenity::Client;
    /// use std::env;
    ///
    /// let mut client = Client::login_bot(&env::var("DISCORD_TOKEN").unwrap());
    /// client.with_framework(|f| f
    ///     .configure(|c| c
    ///         .depth(3)
    ///         .allow_whitespace(true)
    ///         .prefix("~")));
    /// ```
    ///
    /// [`Client`]: ../../client/struct.Client.html
    /// [`Configuration::default`]: struct.Configuration.html#method.default
    /// [`depth`]: struct.Configuration.html#method.depth
    /// [`prefix`]: struct.Configuration.html#method.prefix
    /// [allowing whitespace]: struct.Configuration.html#method.allow_whitespace
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
