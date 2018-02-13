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
use framework::standard::{has_correct_roles, has_correct_permissions};
use model::channel::Message;
use model::id::{ChannelId, UserId};
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;
use std::sync::Arc;
use std::fmt::Write;
use super::command::{InternalCommand};
use super::{Args, CommandGroup, CommandOrAlias, HelpOptions, CommandOptions, CommandError, HelpBehaviour};
use utils::Colour;
use edit_distance;

fn error_embed(channel_id: &ChannelId, input: &str, colour: Colour) {
    let _ = channel_id.send_message(|mut m| {
        m.embed(|mut e| {
            e.colour(colour);
            e.description(input);

            e
        });

        m
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

fn analyse_command_for_suggestion(result: &mut Vec<String>, msg: &Message, command_options: &Arc<CommandOptions>, help_options: &HelpOptions, command_name: &str) {
    if !command_options.dm_only && !command_options.guild_only
    || command_options.dm_only && msg.is_private()
    || command_options.guild_only && !msg.is_private() {

        if let Some(guild) = msg.guild() {
            let guild = guild.read();

            if let Some(member) = guild.members.get(&msg.author.id) {

                if command_options.help_available && has_correct_permissions(command_options, msg) {

                    if has_correct_roles(command_options, &guild, &member) {
                        result.push(format!("`{}`", command_name.clone()));
                    } else {
                        match help_options.lacking_role {
                            HelpBehaviour::Strike => result.push(format!("~~`{}`~~", command_name.clone())),
                            HelpBehaviour::Nothing => result.push(format!("`{}`", command_name.clone())),
                            HelpBehaviour::Hide => (),
                        }
                    }
                }
            } else {
                match help_options.lacking_permissions {
                    HelpBehaviour::Strike => result.push(format!("~~`{}`~~", command_name.clone())),
                    HelpBehaviour::Nothing => result.push(format!("`{}`", command_name.clone())),
                    HelpBehaviour::Hide => (),
                }
            }
        } else {
            result.push(format!("`{}`", command_name.clone()));
        }
    } else {
        match help_options.wrong_channel {
            HelpBehaviour::Strike => result.push(format!("~~`{}`~~", command_name)),
            HelpBehaviour::Nothing => result.push(format!("`{}`", command_name.clone())),
            HelpBehaviour::Hide => (),
        }
    }
}

fn find_similar_commands<H: BuildHasher>(searched_command_name: &str, msg: &Message, groups: &HashMap<String, Arc<CommandGroup>, H>, help_options: &HelpOptions) -> Vec<String> {
    let mut result = Vec::new();

    for (_, group) in groups {

        for (command_name, command) in &group.commands {

            if edit_distance::edit_distance(command_name, &searched_command_name) > help_options.max_edit_distance { continue };

            match *command {
                CommandOrAlias::Command(ref cmd) => {
                    let command_options = cmd.options();
                    if !command_options.suggested { continue };

                    analyse_command_for_suggestion(&mut result, msg, &command_options, help_options, command_name);
                },
                CommandOrAlias::Alias(ref name) => {
                    match group.commands[name] {
                        CommandOrAlias::Command(ref cmd) => {
                        let command_options = cmd.options();
                        if !command_options.suggested { continue };

                        analyse_command_for_suggestion(&mut result, msg, &command_options, help_options, command_name);
                    },
                    _ => continue,
                    }
                }
            }
        }
    }

    result
}

fn generate_similar_commands_message<H: BuildHasher>(searched_command_name: &str, msg: &Message, help_options: &HelpOptions, groups: &HashMap<String, Arc<CommandGroup>, H>) -> String {
    if help_options.find_similar_commands {
        let similar_commands = &find_similar_commands(searched_command_name, msg, groups, help_options);

        if similar_commands.is_empty() {
                help_options.command_not_found_text.replace("{}", &format!("`{}`", searched_command_name))
        } else {
                help_options.suggestion_text.replace("{}", &similar_commands.join(" "))
        }
    } else {
        help_options.command_not_found_text.replace("{}", &format!("`{}`", searched_command_name))
    }
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
/// # let mut client = Client::new("token", Handler).unwrap();
/// #
/// use serenity::framework::standard::{StandardFramework, help_commands};
///
/// client.with_framework(StandardFramework::new()
///     .help(help_commands::with_embeds));
/// ```
pub fn with_embeds<H: BuildHasher>(
    _: &mut Context,
    msg: &Message,
    help_options: &HelpOptions,
    groups: HashMap<String, Arc<CommandGroup>, H>,
    owners: HashSet<UserId>,
    args: &Args
) -> Result<(), CommandError> {
    if !args.is_empty() {
        let name = args.full();

        for (group_name, group) in &groups {
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
                            if has_all_requirements(&cmd.options(), msg) {
                                found = Some((command_name, cmd));
                            } else {
                                break;
                            }
                        },
                        CommandOrAlias::Alias(ref name) => {
                            let actual_command = &group.commands[name];

                            match *actual_command {
                                CommandOrAlias::Command(ref cmd) => {
                                    if has_all_requirements(&cmd.options(), msg) {
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
                    error_embed(&msg.channel_id, &help_options.no_help_available_text, help_options.embed_error_colour);

                    return Ok(());
                }

                let _ = msg.channel_id.send_message(|mut m| {
                    m.embed(|mut embed| {
                        embed.colour(help_options.embed_success_colour);

                        embed.title(command_name.clone());

                        if let Some(ref desc) = command.desc {
                            embed.description(desc);
                        }

                        if let Some(ref usage) = command.usage {
                            let value = format!("`{} {}`", command_name.clone(), usage);

                            embed.field(&help_options.usage_label, value, true);
                        }

                        if let Some(ref example) = command.example {
                            let value = format!("`{} {}`", command_name.clone(), example);

                            embed.field(&help_options.usage_sample_label, value, true);
                        }

                        if group_name != "Ungrouped" {
                            embed.field(&help_options.grouped_label, group_name, true);
                        }

                        if !command.aliases.is_empty() {
                            let aliases = command.aliases.join(", ");

                            embed.field(&help_options.aliases_label, aliases, true);
                        }

                        let available = if command.dm_only {
                            &help_options.dm_only_text
                        } else if command.guild_only {
                            &help_options.guild_only_text
                        } else {
                            &help_options.dm_and_guild_text
                        };

                        embed.field(&help_options.available_text, available, true);

                        embed
                    });

                    m
                });

                return Ok(());
            }
        }

        let error_msg = generate_similar_commands_message(&name, &msg, &help_options, &groups);

        error_embed(&msg.channel_id, &error_msg, help_options.embed_error_colour);

        return Ok(());
    }

    let _ = msg.channel_id.send_message(|mut m| {
        m.embed(|mut embed| {
            if let Some(striked_command_text) = help_options.striked_commands_tip.clone() {
                embed.colour(help_options.embed_success_colour);
                embed.description(format!(
                    "{}\n{}",
                    &help_options.individual_command_tip,
                    &striked_command_text,
                ));
            } else {
                embed.colour(help_options.embed_success_colour);
                embed.description(&help_options.individual_command_tip);
            }

            let mut group_names = groups.keys().collect::<Vec<_>>();
            group_names.sort();

            for group_name in group_names {
                let group = &groups[group_name];
                let mut desc = String::new();

                if let Some(ref x) = group.prefix {
                    let _ = write!(desc, "{}: `{}`\n", &help_options.group_prefix, x);
                }

                let mut has_commands = false;

                let commands = remove_aliases(&group.commands);
                let mut command_names = commands.keys().collect::<Vec<_>>();
                command_names.sort();

                for name in command_names {
                    let cmd = &commands[name];
                    let cmd = cmd.options();

                    let mut display = HelpBehaviour::Nothing;

                    if !cmd.dm_only && !cmd.guild_only || cmd.dm_only && msg.is_private() || cmd.guild_only && !msg.is_private() {

                        if cmd.help_available && has_correct_permissions(&cmd, msg) {

                            if let Some(guild) = msg.guild() {
                                let guild = guild.read();

                                if let Some(member) = guild.members.get(&msg.author.id) {

                                    if !has_correct_roles(&cmd, &guild, &member) {
                                        display = help_options.lacking_role;
                                    }
                                }
                            }
                        } else {
                            display = help_options.lacking_permissions;
                        }
                    } else {
                        display = help_options.wrong_channel;
                    }

                    if cmd.owners_only && !owners.contains(&msg.author.id) {
                        display += help_options.lacking_ownership;
                    }

                    match display {
                        HelpBehaviour::Strike => {
                            let name = format!("~~`{}`~~", &name);
                            let _ = write!(desc, "{}\n", name);
                            has_commands = true;
                        },
                        HelpBehaviour::Nothing => {
                            let _ = write!(desc, "`{}`\n", name);
                            has_commands = true;
                        },
                        HelpBehaviour::Hide => {}
                    }
                }

                if has_commands {
                    embed.field(&group_name[..], &desc[..], true);
                }
            }

            embed
        });

        m
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
/// # let mut client = Client::new("token", Handler).unwrap();
/// #
/// use serenity::framework::standard::{StandardFramework, help_commands};
///
/// client.with_framework(StandardFramework::new()
///     .help(help_commands::plain));
/// ```
pub fn plain<H: BuildHasher>(
    _: &mut Context,
    msg: &Message,
    help_options: &HelpOptions,
    groups: HashMap<String, Arc<CommandGroup>, H>,
    _owners: HashSet<UserId>,
    args: &Args
) -> Result<(), CommandError> {
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
                            if has_all_requirements(&cmd.options(), msg) {
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
                                    if has_all_requirements(&cmd.options(), msg) {
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
                    let _ = write!(result, "**{}:** `{}`\n", help_options.aliases_label, aliases);
                }

                if let Some(ref desc) = command.desc {
                    let _ = write!(result, "**{}:** {}\n", help_options.description_label, desc);
                }

                if let Some(ref usage) = command.usage {
                    let _ = write!(result, "**{}:** `{} {}`\n", help_options.usage_label, command_name, usage);
                }

                if let Some(ref example) = command.example {
                    let _ = write!(result, "**{}:** `{} {}`\n", help_options.usage_sample_label, command_name, example);
                }

                if group_name != "Ungrouped" {
                    let _ = write!(result, "**{}:** {}\n", help_options.grouped_label, group_name);
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
            .say(&help_options.suggestion_text.replace("{}", name));

        return Ok(());
    }

    let mut result = "**Commands**\n".to_string();

    if let Some(striked_command_text) = help_options.striked_commands_tip.clone() {
        let _ = write!(result, "{}\n{}\n\n", &help_options.individual_command_tip, &striked_command_text);
    } else {
        let _ = write!(result, "{}\n\n", &help_options.individual_command_tip);
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

            if let Some(ref x) = group.prefix {
                let _ = write!(result, "({}: `{}`): ", help_options.group_prefix, x);
            }

            result.push_str(&group_help);
            result.push('\n');
        }
    }

    let _ = msg.channel_id.say(&result);

    Ok(())
}
