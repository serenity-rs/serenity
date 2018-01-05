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

use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::Write;
use super::command::InternalCommand;
use super::{Args, Command, CommandGroup, CommandOrAlias, CommandError};
use client::Context;
use model::{ChannelId, Message};
use utils::Colour;
use framework::standard::{has_correct_roles, has_correct_permissions};

fn error_embed(channel_id: &ChannelId, input: &str) {
    let _ = channel_id.send_message(|m| {
        m.embed(|e| e.colour(Colour::dark_red()).description(input))
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
pub fn has_all_requirements(cmd: &Command, msg: &Message) -> bool {
    if let Some(guild) = msg.guild() {
        let guild = guild.read().unwrap();

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

/// Posts an embed showing each individual command group and its commands.
///
/// # Examples
///
/// Use the command with `exec_help`:
///
/// ```rust
/// # use serenity::prelude::*;
/// # struct Handler;
/// #
/// # impl EventHandler for Handler {}
/// # let mut client = Client::new("token", Handler);
/// #
/// use serenity::framework::standard::{StandardFramework, help_commands};
///
/// client.with_framework(StandardFramework::new()
///     .command("help", |c| c.exec_help(help_commands::with_embeds)));
/// ```
pub fn with_embeds(_: &mut Context,
                   msg: &Message,
                   groups: HashMap<String, Arc<CommandGroup>>,
                   args: Args)
                   -> Result<(), CommandError> {
    if !args.is_empty() {
        let name = args.full();

        for (group_name, group) in groups {
            let mut found: Option<(&String, &InternalCommand)> = None;

            for (command_name, command) in &group.commands {
                let with_prefix = if let Some(ref prefix) = group.prefix {
                    format!("{} {}", prefix, command_name)
                } else {
                    command_name.to_string()
                };

                if name == with_prefix || name == *command_name {
                    match *command {
                        CommandOrAlias::Command(ref cmd) => {
                            if has_all_requirements(cmd, msg) {
                                found = Some((command_name, cmd));
                            } else {
                                break;
                            }
                        },
                        CommandOrAlias::Alias(ref name) => {
                            let actual_command = &group.commands[name];

                            match *actual_command {
                                CommandOrAlias::Command(ref cmd) => {
                                    if has_all_requirements(cmd, msg) {
                                        found = Some((name, cmd));
                                    } else {
                                        break;
                                    }
                                },

                                CommandOrAlias::Alias(ref name) => {
                                    let _ = msg.channel_id.say(&format!("Did you mean {:?}?", name));
                                    return Ok(());
                                },
                            }
                        },
                    }
                }
            }

            if let Some((command_name, command)) = found {
                if !command.help_available {
                    error_embed(&msg.channel_id, "**Error**: No help available.");

                    return Ok(());
                }

                let _ = msg.channel_id.send_message(|m| {
                    m.embed(|e| {
                        let mut embed = e.colour(Colour::rosewater()).title(command_name);

                        if let Some(ref desc) = command.desc {
                            embed = embed.description(desc);
                        }

                        if let Some(ref usage) = command.usage {
                            embed = embed.field(|f| {
                                f.name("Usage")
                                    .value(&format!("`{} {}`", command_name, usage))
                            });
                        }

                        if let Some(ref example) = command.example {
                            embed = embed.field(|f| {
                                f.name("Sample usage")
                                    .value(&format!("`{} {}`", command_name, example))
                            });
                        }

                        if group_name != "Ungrouped" {
                            embed = embed.field(|f| f.name("Group").value(&group_name));
                        }

                        if !command.aliases.is_empty() {
                            let aliases = command.aliases.join(", ");
                            embed = embed.field(|f| f.name("Aliases").value(&aliases));
                        }

                        let available = if command.dm_only {
                            "Only in DM"
                        } else if command.guild_only {
                            "Only in guilds"
                        } else {
                            "In DM and guilds"
                        };

                        embed = embed.field(|f| f.name("Available").value(available));

                        embed
                    })
                });

                return Ok(());
            }
        }

        let error_msg = format!("**Error**: Command `{}` not found.", name);
        error_embed(&msg.channel_id, &error_msg);

        return Ok(());
    }

    let _ = msg.channel_id.send_message(|m| {
        m.embed(|mut e| {
            e = e.colour(Colour::rosewater()).description(
                "To get help with an individual command, pass its \
                 name as an argument to this command.",
            );

            let mut group_names = groups.keys().collect::<Vec<_>>();
            group_names.sort();

            for group_name in group_names {
                let group = &groups[group_name];
                let mut desc = String::new();

                if let Some(ref x) = group.prefix {
                    let _ = write!(desc, "Prefix: {}\n", x);
                }

                let mut has_commands = false;

                let commands = remove_aliases(&group.commands);
                let mut command_names = commands.keys().collect::<Vec<_>>();
                command_names.sort();

                for name in command_names {
                    let cmd = &commands[name];

                    if cmd.help_available && has_all_requirements(cmd, msg) {
                        let _ = write!(desc, "`{}`\n", name);
                        has_commands = true;
                    }
                }

                if has_commands {
                    e = e.field(|f| f.name(group_name).value(&desc));
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
/// ```rust
/// # use serenity::prelude::*;
/// # struct Handler;
/// #
/// # impl EventHandler for Handler {}
/// # let mut client = Client::new("token", Handler);
/// #
/// use serenity::framework::standard::{StandardFramework, help_commands};
///
/// client.with_framework(StandardFramework::new()
///     .command("help", |c| c.exec_help(help_commands::plain)));
/// ```
pub fn plain(_: &mut Context,
             msg: &Message,
             groups: HashMap<String, Arc<CommandGroup>>,
             args: Args)
             -> Result<(), CommandError> {
    if !args.is_empty() {
        let name = args.full();

        for (group_name, group) in groups {
            let mut found: Option<(&String, &Command)> = None;

            for (command_name, command) in &group.commands {
                let with_prefix = if let Some(ref prefix) = group.prefix {
                    format!("{} {}", prefix, command_name)
                } else {
                    command_name.to_string()
                };

                if name == with_prefix || name == *command_name {
                    match *command {
                        CommandOrAlias::Command(ref cmd) => {
                            if has_all_requirements(cmd, msg) {
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
                                    if has_all_requirements(cmd, msg) {
                                        found = Some((name, cmd));
                                    }
                                    else {
                                        break;
                                    }
                                },

                                CommandOrAlias::Alias(ref name) => {
                                    let _ = msg.channel_id.say(&format!("Did you mean {:?}?", name));
                                    return Ok(());
                                },
                            }
                        },
                    }
                }
            }

            if let Some((command_name, command)) = found {
                if !command.help_available {
                    let _ = msg.channel_id.say("**Error**: No help available.");
                    return Ok(());
                }

                let mut result = format!("**{}**\n", command_name);

                if !command.aliases.is_empty() {
                    let aliases = command.aliases.join("`, `");
                    let _ = write!(result, "**Aliases:** `{}`\n", aliases);
                }

                if let Some(ref desc) = command.desc {
                    let _ = write!(result, "**Description:** {}\n", desc);
                }

                if let Some(ref usage) = command.usage {
                    let _ = write!(result, "**Usage:** `{} {}`\n", command_name, usage);
                }

                if let Some(ref example) = command.example {
                    let _ = write!(result, "**Sample usage:** `{} {}`\n", command_name, example);
                }

                if group_name != "Ungrouped" {
                    let _ = write!(result, "**Group:** {}\n", group_name);
                }

                let only = if command.dm_only {
                    "Only in DM"
                } else if command.guild_only {
                    "Only in guilds"
                } else {
                    "In DM and guilds"
                };

                result.push_str("**Available:** ");
                result.push_str(only);
                result.push_str("\n");

                let _ = msg.channel_id.say(&result);

                return Ok(());
            }
        }

        let _ = msg.channel_id
            .say(&format!("**Error**: Command `{}` not found.", name));

        return Ok(());
    }

    let mut result = "**Commands**\nTo get help with an individual command, pass its \
                      name as an argument to this command.\n\n"
        .to_string();

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

            if cmd.help_available && has_all_requirements(cmd, msg) {
                let _ = write!(group_help, "`{}` ", name);
            }
        }

        if !group_help.is_empty() {
            let _ = write!(result, "**{}:** ", group_name);

            if let Some(ref x) = group.prefix {
                let _ = write!(result, "(prefix: `{}`): ", x);
            }

            result.push_str(&group_help);
            result.push('\n');
        }
    }

    let _ = msg.channel_id.say(&result);

    Ok(())
}
