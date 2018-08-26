//! A collection of default help commands for the framework.
//!
//! # Example
//!
//! Using the [`with_embeds`] function to have the framework's help message use
//! embeds:
//!
//! ```rs,no_run
//! use serenity::framework::standard::help_commands;
//! use serenity::Client;
//! use std::env;
//!
//! let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap());
//! use serenity::framework::StandardFramework;
//!
//! client.with_framework(StandardFramework::new()
//!     .command("help", |c| c.exec_help(help_commands::with_embeds)));
//! ```
//!
//! The same can be accomplished with no embeds by substituting `with_embeds`
//! with the [`plain`] function.
//!
//! [`plain`]: fn.plain.html
//! [`with_embeds`]: fn.with_embeds.html

use client::Context;
#[cfg(feature = "cache")]
use framework::standard::{has_correct_roles, has_correct_permissions};
use model::{
    channel::Message,
    id::ChannelId,
};
use std::{
    collections::HashMap,
    hash::BuildHasher,
    sync::Arc,
    fmt::Write,
};
use super::command::InternalCommand;
use super::{
    Args,
    CommandGroup,
    CommandOrAlias,
    HelpOptions,
    CommandOptions,
    CommandError,
    HelpBehaviour,
};
use utils::Colour;

/// Macro to format a command according to a `HelpBehaviour` or
/// continue to the next command-name upon hiding.
macro_rules! format_command_name {
    ($behaviour:expr, $command_name:expr) => {
        match $behaviour {
            &HelpBehaviour::Strike => format!("~~`{}`~~", $command_name),
            &HelpBehaviour::Nothing => format!("`{}`", $command_name),
            &HelpBehaviour::Hide => continue,
        }
    };
}

/// A single group containing its name and all related commands that are eligible
/// in relation of help-settings measured to the user.
#[derive(Clone, Debug, Default)]
pub struct GroupCommandsPair<'a> {
    name: &'a str,
    prefixes: Vec<String>,
    command_names: Vec<String>,
}

/// A single suggested command containing its name and Levenshtein distance
/// to the actual user's searched command name.
#[derive(Clone, Debug, Default)]
struct SuggestedCommandName<'a> {
    name: &'a str,
    levenshtein_distance: usize,
}

/// A single command containing all related pieces of information.
#[derive(Clone, Debug)]
pub struct Command<'a> {
    name: &'a str,
    group_name: &'a str,
    aliases: Vec<String>,
    availability: &'a str,
    description: Option<String>,
    usage: Option<String>,
}

/// Contains possible suggestions in case a command could not be found
/// but are similar enough.
#[derive(Clone, Debug, Default)]
pub struct Suggestions<'a>(Vec<SuggestedCommandName<'a>>);

impl<'a> Suggestions<'a> {
    /// Immutably borrow inner `Vec`.
    #[inline]
    fn as_vec(&self) -> &Vec<SuggestedCommandName> {
        &self.0
    }

    /// Concat names of suggestions with a given `seperator`.
    fn join(&self, seperator: &str) -> String {
        let iter = self.as_vec().iter();
        let size = self.as_vec().iter().fold(0, |total_size, size| total_size + size.name.len());
        let byte_len_of_sep = self.as_vec().len().checked_sub(1).unwrap_or(0) * seperator.len();
        let mut result = String::with_capacity(size + byte_len_of_sep);

        for v in iter {
            result.push_str(&*seperator);
            result.push_str(v.name.borrow());
        }

        result
    }
}

/// Covers possible outcomes of a help-request and
/// yields relevant data in customised textual
/// representation.
#[derive(Clone, Debug)]
pub enum CustomisedHelpData<'a> {
    /// To display suggested commands.
    SuggestedCommands {
        help_description: String,
        suggestions: Suggestions<'a>,
    },
    /// To display groups and their commands by name.
    GroupedCommands {
        help_description: String,
        groups: Vec<GroupCommandsPair<'a>>,
    },
    /// To display one specific command.
    SingleCommand { command: Command<'a> },
    /// To display failure in finding a fitting command.
    NoCommandFound { help_error_message: &'a str },
}

fn error_embed(channel_id: ChannelId, input: &str, colour: Colour) {
    let _ = channel_id.send_message(|m| {
        m.embed(|e| e.colour(colour).description(input))
    });
}

fn remove_aliases(cmds: &HashMap<String, CommandOrAlias>) -> HashMap<&String, &InternalCommand> {
    let mut result = HashMap::new();

    for (n, v) in cmds {
        if let CommandOrAlias::Command(ref cmd) = *v {
            result.insert(n, cmd);
        }
    }

    result
}

/// Checks whether a user is member of required roles
/// and given the required permissions.
#[cfg(feature = "cache")]
pub fn has_all_requirements(cmd: &Arc<CommandOptions>, msg: &Message) -> bool {
    if let Some(guild) = msg.guild() {
        let guild = guild.read();

        if let Some(member) = guild.members.get(&msg.author.id) {

            if let Ok(permissions) = member.permissions() {

                if cmd.allowed_roles.is_empty() {
                    return permissions.administrator() || has_correct_permissions(cmd, msg);
                } else {
                    return permissions.administrator() || (has_correct_roles(cmd, &guild, member) && has_correct_permissions(cmd, msg));
                }
            }
        }
    }
    !cmd.guild_only
}

/// Checks whether a command would be visible, takes permissions, channel sent in,
/// and roles into consideration.
///
/// **Note**: A command is visible when it is either normally displayed or
/// strikethrough upon requested help by a user.
#[cfg(feature = "cache")]
pub fn is_command_visible(command_options: &Arc<CommandOptions>, msg: &Message, help_options: &HelpOptions) -> bool {
    if !command_options.dm_only && !command_options.guild_only
    || command_options.dm_only && msg.is_private()
    || command_options.guild_only && !msg.is_private() {

        if let Some(guild) = msg.guild() {
            let guild = guild.read();

            if let Some(member) = guild.members.get(&msg.author.id) {

                if command_options.help_available {

                    if has_correct_permissions(command_options, msg) {

                        if has_correct_roles(command_options, &guild, &member) {
                            return true;
                        } else {
                            return help_options.lacking_role != HelpBehaviour::Hide;
                        }
                    } else {
                        return help_options.lacking_permissions != HelpBehaviour::Hide;
                    }
                }
            }
        } else if command_options.help_available {
            if has_correct_permissions(command_options, msg) {
                return true;
            } else {
                return help_options.lacking_permissions != HelpBehaviour::Hide;
            }
        }
    } else {
        return help_options.wrong_channel != HelpBehaviour::Hide;
    }

    false
}

/// Posts an embed showing each individual command group and its commands.
///
/// # Examples
///
/// Use the command with `exec_help`:
///
/// ```rust,no_run
/// # use serenity::prelude::*;
/// # struct Handler;
/// #
/// # impl EventHandler for Handler {}
/// # let mut client = Client::new("token", Handler).unwrap();
/// #
/// use serenity::framework::standard::{StandardFramework, help_commands};
///
/// client.with_framework(StandardFramework::new()
///     .help(help_commands::with_embeds));
/// ```
#[cfg(feature = "cache")]
pub fn with_embeds<H: BuildHasher>(
    _: &mut Context,
    msg: &Message,
    help_options: &HelpOptions,
    groups: HashMap<String, Arc<CommandGroup>, H>,
    args: &Args
) -> Result<(), CommandError> {
    if !args.is_empty() {
        let name = args.full();

        for (group_name, group) in groups {
            let mut found: Option<(&String, &InternalCommand)> = None;

            for (command_name, command) in &group.commands {
                let with_prefix = if let Some(ref prefixes) = group.prefixes {
                    format!("{} {}", prefixes.join("`, `"), command_name)
                } else {
                    command_name.to_string()
                };

                if name == with_prefix || name == *command_name {
                    match *command {
                        CommandOrAlias::Command(ref cmd) => {
                            if is_command_visible(&cmd.options(), msg, help_options) {
                                found = Some((command_name, cmd));
                            } else {
                                break;
                            }
                        },
                        CommandOrAlias::Alias(ref name) => {
                            let actual_command = &group.commands[name];

                            match *actual_command {
                                CommandOrAlias::Command(ref cmd) => {
                                    if is_command_visible(&cmd.options(), msg, help_options) {
                                        found = Some((name, cmd));
                                    } else {
                                        break;
                                    }
                                },

                                CommandOrAlias::Alias(ref name) => {
                                    let _ = msg.channel_id.say(help_options.suggestion_text.replace("{}", name));
                                    return Ok(());
                                },
                            }
                        },
                    }
                }
            }

            if let Some((command_name, command)) = found {
                let command = command.options();
                if !command.help_available {
                    error_embed(msg.channel_id, &help_options.no_help_available_text, help_options.embed_error_colour);

                    return Ok(());
                }

                let _ = msg.channel_id.send_message(|m| {
                    m.embed(|e| {
                        let mut embed = e.colour(help_options.embed_success_colour).title(command_name);

                        if let Some(ref desc) = command.desc {
                            embed = embed.description(desc);
                        }

                        if let Some(ref usage) = command.usage {
                            let value = format!("`{} {}`", command_name, usage);

                            embed = embed.field(&help_options.usage_label, value, true);
                        }

                        if let Some(ref example) = command.example {
                            let value = format!("`{} {}`", command_name, example);

                            embed = embed.field(&help_options.usage_sample_label, value, true);
                        }

                        if group_name != "Ungrouped" {
                            embed = embed.field(&help_options.grouped_label, group_name, true);
                        }

                        if !command.aliases.is_empty() {
                            let aliases = command.aliases.join("`, `");

                            embed = embed.field(&help_options.aliases_label, aliases, true);
                        }

                        let available = if command.dm_only {
                            &help_options.dm_only_text
                        } else if command.guild_only {
                            &help_options.guild_only_text
                        } else {
                            &help_options.dm_and_guild_text
                        };

                        embed = embed.field(&help_options.available_text, available, true);

                        embed
                    })
                });

                return Ok(());
            }
        }

        let error_msg = help_options.command_not_found_text.replace("{}", name);
        error_embed(msg.channel_id, &error_msg, help_options.embed_error_colour);

        return Ok(());
    }

    let _ = msg.channel_id.send_message(|m| {
        m.embed(|mut e| {
            let striked_command_tip = if msg.is_private() {
                    &help_options.striked_commands_tip_in_guild
                } else {
                    &help_options.striked_commands_tip_in_dm
                };

            if let &Some(ref striked_command_text) = striked_command_tip {
                e = e.colour(help_options.embed_success_colour).description(
                    format!("{}\n{}", &help_options.individual_command_tip, striked_command_text),
                );
            } else {
                e = e.colour(help_options.embed_success_colour).description(
                    &help_options.individual_command_tip,
                );
            }

            let mut group_names = groups.keys().collect::<Vec<_>>();
            group_names.sort();

            for group_name in group_names {
                let group = &groups[group_name];
                let mut desc = String::new();

                if let Some(ref prefixes) = group.prefixes {
                    let _ = writeln!(desc, "{}: `{}`", &help_options.group_prefix, prefixes.join("`, `"));
                }

                let mut has_commands = false;

                let commands = remove_aliases(&group.commands);
                let mut command_names = commands.keys().collect::<Vec<_>>();
                command_names.sort();

                for name in command_names {
                    let cmd = &commands[name];
                    let cmd = cmd.options();

                    if !cmd.dm_only && !cmd.guild_only || cmd.dm_only && msg.is_private() || cmd.guild_only && !msg.is_private() {

                        if cmd.help_available && has_correct_permissions(&cmd, msg) {

                            if let Some(guild) = msg.guild() {
                                let guild = guild.read();

                                if let Some(member) = guild.members.get(&msg.author.id) {

                                    if has_correct_roles(&cmd, &guild, &member) {
                                        let _ = writeln!(desc, "`{}`", name);
                                        has_commands = true;
                                    } else {
                                        match help_options.lacking_role {
                                            HelpBehaviour::Strike => {
                                                let name = format!("~~`{}`~~", &name);
                                                let _ = writeln!(desc, "{}", name);
                                                has_commands = true;
                                            },
                                                HelpBehaviour::Nothing => {
                                                let _ = writeln!(desc, "`{}`", name);
                                                has_commands = true;
                                            },
                                                HelpBehaviour::Hide => {
                                                continue;
                                            },
                                        }
                                    }
                                }
                            } else {
                                let _ = writeln!(desc, "`{}`", name);
                                has_commands = true;
                            }
                        } else {
                            match help_options.lacking_permissions {
                                HelpBehaviour::Strike => {
                                    let name = format!("~~`{}`~~", &name);
                                    let _ = writeln!(desc, "{}", name);
                                    has_commands = true;
                                },
                                HelpBehaviour::Nothing => {
                                    let _ = writeln!(desc, "`{}`", name);
                                    has_commands = true;
                                },
                                HelpBehaviour::Hide => {
                                    continue;
                                },
                            }
                        }
                    } else {
                        match help_options.wrong_channel {
                            HelpBehaviour::Strike => {
                                let name = format!("~~`{}`~~", &name);
                                let _ = writeln!(desc, "{}", name);
                                has_commands = true;
                            },
                            HelpBehaviour::Nothing => {
                                let _ = writeln!(desc, "`{}`", name);
                                has_commands = true;
                            },
                            HelpBehaviour::Hide => {
                                continue;
                            },
                        }
                    }
                }

                if has_commands {
                    e = e.field(&group_name[..], &desc[..], true);
                }
            }
            e
        })
    });

    Ok(())
}

/// Posts formatted text displaying each individual command group and its commands.
///
/// # Examples
///
/// Use the command with `exec_help`:
///
/// ```rust,no_run
/// # use serenity::prelude::*;
/// # struct Handler;
/// #
/// # impl EventHandler for Handler {}
/// # let mut client = Client::new("token", Handler).unwrap();
/// #
/// use serenity::framework::standard::{StandardFramework, help_commands};
///
/// client.with_framework(StandardFramework::new()
///     .help(help_commands::plain));
/// ```
#[cfg(feature = "cache")]
pub fn plain<H: BuildHasher>(
    _: &mut Context,
    msg: &Message,
    help_options: &HelpOptions,
    groups: HashMap<String, Arc<CommandGroup>, H>,
    args: &Args
) -> Result<(), CommandError> {
    if !args.is_empty() {
        let name = args.full();

        for (group_name, group) in groups {
            let mut found: Option<(&String, &InternalCommand)> = None;

            for (command_name, command) in &group.commands {
                let with_prefix = if let Some(ref prefixes) = group.prefixes {
                    format!("{} {}", prefixes.join("`, `"), command_name)
                } else {
                    command_name.to_string()
                };

                if name == with_prefix || name == *command_name {
                    match *command {
                        CommandOrAlias::Command(ref cmd) => {
                            if is_command_visible(&cmd.options(), msg, help_options) {
                                found = Some((command_name, cmd));
                            }
                            else {
                                break;
                            }
                        },
                        CommandOrAlias::Alias(ref name) => {
                            let actual_command = &group.commands[name];

                            match *actual_command {
                                CommandOrAlias::Command(ref cmd) => {
                                    if is_command_visible(&cmd.options(), msg, help_options) {
                                        found = Some((name, cmd));
                                    }
                                    else {
                                        break;
                                    }
                                },

                                CommandOrAlias::Alias(ref name) => {
                                    let _ = msg.channel_id.say(help_options.suggestion_text.replace("{}", name));
                                    return Ok(());
                                },
                            }
                        },
                    }
                }
            }

            if let Some((command_name, command)) = found {
                let command = command.options();

                if !command.help_available {
                    let _ = msg.channel_id.say(&help_options.no_help_available_text);
                    return Ok(());
                }

                let mut result = format!("**{}**\n", command_name);

                if !command.aliases.is_empty() {
                    let aliases = command.aliases.join("`, `");
                    let _ = writeln!(result, "**{}:** `{}`", help_options.aliases_label, aliases);
                }

                if let Some(ref desc) = command.desc {
                    let _ = writeln!(result, "**{}:** {}", help_options.description_label, desc);
                }

                if let Some(ref usage) = command.usage {
                    let _ = writeln!(result, "**{}:** `{} {}`", help_options.usage_label, command_name, usage);
                }

                if let Some(ref example) = command.example {
                    let _ = writeln!(result, "**{}:** `{} {}`", help_options.usage_sample_label, command_name, example);
                }

                if group_name != "Ungrouped" {
                    let _ = writeln!(result, "**{}:** {}", help_options.grouped_label, group_name);
                }

                let only = if command.dm_only {
                    &help_options.dm_only_text
                } else if command.guild_only {
                    &help_options.guild_only_text
                } else {
                    &help_options.dm_and_guild_text
                };

                result.push_str(&format!("**{}:** ", &help_options.available_text));
                result.push_str(only);
                result.push_str(".\n");

                let _ = msg.channel_id.say(&result);

                return Ok(());
            }
        }

        let _ = msg.channel_id
            .say(&help_options.command_not_found_text.replace("{}", name));

        return Ok(());
    }

    let mut result = "**Commands**\n".to_string();

    let striked_command_tip = if msg.is_private() {
            &help_options.striked_commands_tip_in_guild
        } else {
            &help_options.striked_commands_tip_in_dm
    };

    if let &Some(ref striked_command_text) = striked_command_tip {
        let _ = writeln!(result, "{}\n{}\n", &help_options.individual_command_tip, striked_command_text);
    } else {
        let _ = writeln!(result, "{}\n", &help_options.individual_command_tip);
    }

    let mut group_names = groups.keys().collect::<Vec<_>>();
    group_names.sort();

    for group_name in group_names {
        let group = &groups[group_name];
        let mut group_help = String::new();

        let commands = remove_aliases(&group.commands);
        let mut command_names = commands.keys().collect::<Vec<_>>();
        command_names.sort();

        for name in command_names {
            let cmd = &commands[name];
            let cmd = cmd.options();

            if !cmd.dm_only && !cmd.guild_only || cmd.dm_only && msg.is_private() || cmd.guild_only && !msg.is_private() {
                if cmd.help_available && has_correct_permissions(&cmd, msg) {

                    if let Some(guild) = msg.guild() {
                        let guild = guild.read();

                        if let Some(member) = guild.members.get(&msg.author.id) {

                            if has_correct_roles(&cmd, &guild, &member) {
                                let _ = write!(group_help, "`{}` ", name);
                            } else {
                                match help_options.lacking_role {
                                    HelpBehaviour::Strike => {
                                        let name = format!("~~`{}`~~", &name);
                                        let _ = write!(group_help, "{} ", name);
                                    },
                                    HelpBehaviour::Nothing => {
                                        let _ = write!(group_help, "`{}` ", name);
                                    },
                                    HelpBehaviour::Hide => {
                                        continue;
                                    },
                                }
                            }
                        }
                    } else {
                        let _ = write!(group_help, "`{}` ", name);
                    }
                } else {
                    match help_options.lacking_permissions {
                        HelpBehaviour::Strike => {
                            let name = format!("~~`{}`~~", &name);
                            let _ = write!(group_help, "{} ", name);
                        },
                        HelpBehaviour::Nothing => {
                            let _ = write!(group_help, "`{}` ", name);
                        },
                        HelpBehaviour::Hide => {
                            continue;
                        },
                    }
                }
            } else {
                match help_options.wrong_channel {
                    HelpBehaviour::Strike => {
                        let name = format!("~~`{}`~~", &name);
                        let _ = write!(group_help, "{} ", name);
                    },
                    HelpBehaviour::Nothing => {
                        let _ = write!(group_help, "`{}` ", name);
                    },
                    HelpBehaviour::Hide => {
                        continue;
                    },
                }
            }
        }

        if !group_help.is_empty() {
            let _ = write!(result, "**{}:** ", group_name);

            if let Some(ref prefixes) = group.prefixes {
                let _ = write!(result, "({}: `{}`): ", help_options.group_prefix, prefixes.join("`, `"));
            }

            result.push_str(&group_help);
            result.push('\n');
        }
    }

    let _ = msg.channel_id.say(&result);

    Ok(())
}
