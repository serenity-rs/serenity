use super::{Command, CommandGroup, CommandOptions, Configuration};
use crate::client::Context;
use crate::model::channel::Message;
use uwl::{StrExt, StringStream};

#[derive(Debug, Clone, PartialEq)]
pub enum Prefix<'a> {
    Punct(&'a str),
    Mention(&'a str),
    None,
}

pub fn parse_prefix<'a>(
    ctx: &mut Context,
    msg: &'a Message,
    config: &Configuration,
) -> (Prefix<'a>, &'a str) {
    let mut stream = StringStream::new(&msg.content);
    stream.take_while(|s| s.is_whitespace());

    if let Some(ref mention) = config.on_mention {
        if let Ok(id) = stream.parse("<@(!){}>") {
            if id.is_numeric() && mention == id {
                stream.take_while(|s| s.is_whitespace());
                return (Prefix::Mention(id), stream.rest());
            }
        }
    }

    let mut prefix = None;
    if !config.prefixes.is_empty() || !config.dynamic_prefixes.is_empty() {
        for f in &config.dynamic_prefixes {
            if let Some(p) = f(ctx, msg) {
                let pp = stream.peek_for(p.chars().count());

                if p == pp {
                    prefix = Some(pp);
                    break;
                }
            }
        }

        for p in &config.prefixes {
            // If `dynamic_prefixes` succeeded, don't iterate through the normal prefixes.
            if prefix.is_some() {
                break;
            }

            let pp = stream.peek_for(p.chars().count());

            if p == pp {
                prefix = Some(pp);
                break;
            }
        }
    }

    if let Some(prefix) = prefix {
        let pos = stream.offset();
        stream.set(pos + prefix.len());

        if config.with_whitespace.prefixes {
            stream.take_while(|s| s.is_whitespace());
        }

        let args = stream.rest();

        return (Prefix::Punct(prefix), args.trim());
    }

    if config.with_whitespace.prefixes {
        stream.take_while(|s| s.is_whitespace());
    }

    let args = stream.rest();
    (Prefix::None, args.trim())
}

struct PrefixIterator<'a, 'b: 'a, 'c> {
    stream: &'a mut StringStream<'b>,
    group: &'static CommandGroup,
    prefix: Option<&'b str>,
    config: &'c Configuration,
}

impl<'a, 'b, 'c> PrefixIterator<'a, 'b, 'c> {
    fn next(&mut self) -> bool {
        // Nothing to work with.
        if !self.group.has_prefixes() {
            self.prefix = None;
            return false;
        }

        // First, check if the subgroups' prefixes were used.
        // And on success, change the current group to the matching group.
        for sub in self.group.sub {
            for p in sub.options.prefixes {
                let pp = self.stream.peek_for(p.chars().count());

                if *p == pp {
                    self.prefix = Some(pp);
                    self.group = *sub;
                    let pos = self.stream.offset();
                    self.stream.set(pos + pp.len());

                    if self.config.with_whitespace.groups {
                        self.stream.take_while(|s| s.is_whitespace());
                    }

                    return true;
                }
            }
        }

        // Then check if this group's prefixes were used.
        for p in self.group.options.prefixes {
            let pp = self.stream.peek_for(p.chars().count());

            if *p == pp {
                self.prefix = Some(pp);
                let pos = self.stream.offset();
                self.stream.set(pos + pp.len());

                if self.config.with_whitespace.groups {
                    self.stream.take_while(|s| s.is_whitespace());
                }

                return true;
            }
        }

        self.prefix = None;

        false
    }
}

impl CommandGroup {
    #[inline]
    fn get_command(&self, name: &str) -> Option<&'static Command> {
        self.commands
            .iter()
            .find(|c| c.options.names.contains(&name))
            .map(|c| *c)
    }

    #[inline]
    fn command_names(&self) -> impl Iterator<Item = &'static str> {
        self.commands
            .iter()
            .flat_map(|c| c.options.names.iter().map(|c| *c))
    }

    fn has_prefixes(&self) -> bool {
        for sub in self.sub {
            if !sub.has_prefixes() {
                return false;
            }
        }

        !self.options.prefixes.is_empty()
    }
}

impl CommandOptions {
    #[inline]
    fn get_sub(&self, name: &str) -> Option<&'static Command> {
        self.sub
            .iter()
            .find(|c| c.options.names.contains(&name))
            .map(|c| *c)
    }

    #[inline]
    fn command_names(&self) -> impl Iterator<Item = &'static str> {
        self.sub
            .iter()
            .flat_map(|c| c.options.names.iter().map(|c| *c))
    }
}

pub(crate) fn parse_command<'a>(
    msg: &'a str,
    prefix: Prefix<'a>,
    groups: &[&'static CommandGroup],
    config: &Configuration,
    help_was_set: bool,
) -> Result<Invoke<'a>, Option<&'a str>> {
    let mut stream = StringStream::new(msg);
    stream.take_while(|s| s.is_whitespace());

    let mut unrecognised = None;

    // We take precedence over commands named `help`.
    if help_was_set && stream.eat("help") {
        stream.take_while(|s| s.is_whitespace());

        let args = stream.rest();

        return Ok(Invoke::Help { prefix, args });
    }

    for group in groups {
        let mut group: &'static CommandGroup = *group;
        let pos = stream.offset();

        let mut gprefix = None;

        // New block because of the lifetime constraint to `StringStream`.
        {
            // Iterate through the possible prefixes of a group and its subgroups.
            let mut it = PrefixIterator {
                stream: &mut stream,
                group,
                config,
                prefix: None,
            };

            while it.next() {
                gprefix = it.prefix.take();
            }

            group = it.group;
        }

        // the group has prefixes defined but none were provided.
        if gprefix.is_none() && group.has_prefixes() {
            unsafe { stream.set_unchecked(pos) };
            continue;
        }

        let pos = stream.offset();

        for name in group.command_names() {
            let n = stream.peek_for(name.chars().count());
            let equals = if config.case_insensitive {
                let n = n.to_lowercase();
                n == name && !config.disabled_commands.contains(&n)
            } else {
                n == name && !config.disabled_commands.contains(n)
            };

            if equals {
                let pos = stream.offset();
                stream.set(pos + n.len());

                if config.with_whitespace.commands {
                    stream.take_while(|s| s.is_whitespace());
                }

                let mut command = group.get_command(&n).unwrap();

                // Go through all the possible subcommands.
                fn iterate_commands<'a>(
                    stream: &mut StringStream<'a>,
                    config: &Configuration,
                    command: &mut &'static Command,
                ) {
                    for name in command.options.command_names() {
                        let n = stream.peek_for(name.chars().count());
                        let equals = if config.case_insensitive {
                            let n = n.to_lowercase();
                            n == name && !config.disabled_commands.contains(&n)
                        } else {
                            n == name && !config.disabled_commands.contains(n)
                        };

                        if equals {
                            let pos = stream.offset();
                            stream.set(pos + n.len());

                            if config.with_whitespace.commands {
                                stream.take_while(|s| s.is_whitespace());
                            }

                            *command = command.options.get_sub(&n).unwrap();
                            iterate_commands(stream, config, command);
                        }
                    }
                }

                iterate_commands(&mut stream, &config, &mut command);

                unrecognised.take();

                let args = stream.rest();

                return Ok(Invoke::Command {
                    prefix,
                    gprefix,
                    group,
                    command,
                    args,
                });
            } else {
                unrecognised = Some(n);
            }
        }

        unsafe { stream.set_unchecked(pos) };

        // Only execute the default command if a group prefix is present.
        if gprefix.is_some() {
            if let Some(command) = group.options.default_command {
                let args = stream.rest();

                return Ok(Invoke::Command {
                    prefix,
                    gprefix,
                    group,
                    command,
                    args,
                });
            }
        }
    }

    Err(unrecognised)
}

#[derive(Debug)]
pub enum Invoke<'a> {
    Command {
        prefix: Prefix<'a>,
        // Group prefix
        gprefix: Option<&'a str>,

        group: &'static CommandGroup,
        command: &'static Command,
        args: &'a str,
    },
    Help {
        prefix: Prefix<'a>,
        args: &'a str,
    },
}
