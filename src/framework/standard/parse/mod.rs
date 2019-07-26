use super::*;
use crate::client::Context;
use crate::model::channel::Message;
use uwl::{StrExt, UnicodeStream};

pub mod map;

use map::*;

use std::borrow::Cow;

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
    let mut stream = UnicodeStream::new(&msg.content);
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
        stream.increment(prefix.len());

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

fn check_discrepancy(
    ctx: &Context,
    msg: &Message,
    config: &Configuration,
    options: &impl CommonOptions,
) -> Result<(), DispatchError> {
    if options.owners_only() && !config.owners.contains(&msg.author.id) {
        return Err(DispatchError::OnlyForOwners);
    }

    if options.only_in() == OnlyIn::Dm && !msg.is_private() {
        return Err(DispatchError::OnlyForDM);
    }

    if (!config.allow_dm || options.only_in() == OnlyIn::Guild) && msg.is_private() {
        return Err(DispatchError::OnlyForGuilds);
    }

    #[cfg(feature = "cache")]
    {
        if let Some(guild_id) = msg.guild_id {
            let guild = match guild_id.to_guild_cached(&ctx) {
                Some(g) => g,
                None => return Ok(()),
            };

            let guild = guild.read();

            let perms = guild.permissions_in(msg.channel_id, msg.author.id);

            if !perms.contains(*options.required_permissions())
                && !(options.owner_privilege() && config.owners.contains(&msg.author.id))
            {
                return Err(DispatchError::LackingPermissions(
                    *options.required_permissions(),
                ));
            }

            if let Some(member) = guild.members.get(&msg.author.id) {
                if !perms.administrator() && !has_correct_roles(options, &guild, &member) {
                    return Err(DispatchError::LackingRole);
                }
            }
        }
    }

    Ok(())
}

#[inline]
fn to_lowercase<'a>(config: &Configuration, s: &'a str) -> Cow<'a, str> {
    if config.case_insensitive {
        Cow::Owned(s.to_lowercase())
    } else {
        Cow::Borrowed(s)
    }
}

fn try_parse<'msg, M: ParseMap>(
    stream: &mut UnicodeStream<'msg>,
    map: &M,
    by_space: bool,
    f: impl Fn(&str) -> String,
) -> (String, Option<M::Storage>) {
    if by_space {
        let n = f(stream.peek_until(|s| s.is_whitespace()));

        let o = map.get(&n);

        (n, o)
    } else {
        let mut n = f(stream.peek_for(map.max_length()));
        let mut o = None;

        for _ in 0..(map.max_length() - map.min_length()) {
            o = map.get(&n);

            if o.is_some() {
                break;
            }

            n.pop();
        }

        (n, o)
    }
}

fn parse_cmd(
    stream: &mut UnicodeStream<'_>,
    ctx: &Context,
    msg: &Message,
    config: &Configuration,
    map: &CommandMap,
) -> Result<&'static Command, ParseError> {
    let (n, r) = try_parse(stream, map, config.by_space, |s| to_lowercase(config, s).to_string());

    if config.disabled_commands.contains(&n) {
        return Err(From::from(DispatchError::CommandDisabled(n)));
    }

    if let Some((cmd, map)) = r {
        stream.increment(n.len());

        if config.with_whitespace.commands {
            stream.take_while(|s| s.is_whitespace());
        }

        check_discrepancy(ctx, msg, config, &cmd.options)?;

        if map.is_empty() {
            return Ok(cmd);
        }

        return match parse_cmd(stream, ctx, msg, config, &map) {
            Err(ParseError::UnrecognisedCommand(Some(_))) => Ok(cmd),
            res => res,
        };
    }

    Err(ParseError::UnrecognisedCommand(Some(n.to_string())))
}

fn parse_group<'msg>(
    stream: &mut UnicodeStream<'_>,
    ctx: &Context,
    msg: &Message,
    config: &Configuration,
    map: &GroupMap,
) -> Result<(&'static CommandGroup, Arc<CommandMap>), ParseError> {
    let (n, o) = try_parse(stream, map, config.by_space, ToString::to_string);

    if let Some((group, map, commands)) = o {
        stream.increment(n.len());

        if config.with_whitespace.groups {
            stream.take_while(|s| s.is_whitespace());
        }

        check_discrepancy(ctx, msg, config, &group.options)?;

        if map.is_empty() {
            return Ok((group, commands));
        }

        return match parse_group(stream, ctx, msg, config, &map) {
            Err(ParseError::UnrecognisedCommand(None)) => Ok((group, commands)),
            res => res,
        };
    }

    Err(ParseError::UnrecognisedCommand(None))
}

#[inline]
fn handle_command<'msg>(
    stream: &mut UnicodeStream<'msg>,
    ctx: &Context,
    msg: &Message,
    config: &Configuration,
    map: &CommandMap,
    group: &'static CommandGroup,
) -> Result<Invoke<'msg>, ParseError> {
    match parse_cmd(stream, ctx, msg, config, map) {
        Ok(command) => Ok(Invoke::Command {
            group,
            command,
            args: stream.rest(),
        }),
        Err(err) => match group.options.default_command {
            Some(command) => Ok(Invoke::Command {
                group,
                command,
                args: stream.rest(),
            }),
            None => Err(err),
        },
    }
}

#[inline]
fn handle_group<'msg>(
    stream: &mut UnicodeStream<'msg>,
    ctx: &Context,
    msg: &Message,
    config: &Configuration,
    map: &GroupMap,
) -> Result<Invoke<'msg>, ParseError> {
    parse_group(stream, ctx, msg, config, map)
        .and_then(|(group, map)| handle_command(stream, ctx, msg, config, &map, group))
}

#[derive(Debug)]
pub enum ParseError {
    UnrecognisedCommand(Option<String>),
    DispatchFailure(DispatchError),
}

impl From<DispatchError> for ParseError {
    #[inline]
    fn from(err: DispatchError) -> Self {
        ParseError::DispatchFailure(err)
    }
}

pub fn parse_command<'a>(
    ctx: &Context,
    msg: &'a Message,
    message: &'a str,
    groups: &[(&'static CommandGroup, Map)],
    config: &Configuration,
    help_was_set: Option<&[&'static str]>,
) -> Result<Invoke<'a>, ParseError> {
    let mut stream = UnicodeStream::new(message);
    stream.take_while(|s| s.is_whitespace());

    // We take precedence over commands named help command's name.
    if let Some(names) = help_was_set {
        for name in names {
            let n = to_lowercase(config, stream.peek_for(name.chars().count()));

            if name == &n {
                stream.increment(n.len());

                stream.take_while(|s| s.is_whitespace());

                let args = stream.rest();

                return Ok(Invoke::Help { name, args });
            }
        }
    }

    let mut last = Err(ParseError::UnrecognisedCommand(None));

    for (group, map) in groups {
        match map {
            // Includes [group] itself.
            Map::WithPrefixes(map) => {
                let res = handle_group(&mut stream, ctx, msg, config, map);

                if res.is_ok() {
                    return res;
                }

                last = res;
            }
            Map::Prefixless(subgroups, commands) => {
                let res = handle_group(&mut stream, ctx, msg, config, subgroups);

                if res.is_ok() {
                    check_discrepancy(ctx, msg, config, &group.options)?;

                    return res;
                }

                let res = handle_command(&mut stream, ctx, msg, config, commands, group);

                if res.is_ok() {
                    check_discrepancy(ctx, msg, config, &group.options)?;

                    return res;
                }

                last = res;
            }
        }
    }

    last
}

#[derive(Debug)]
pub enum Invoke<'a> {
    Command {
        group: &'static CommandGroup,
        command: &'static Command,
        args: &'a str,
    },
    Help {
        name: &'static str,
        args: &'a str,
    },
}
