//! A collection of default help commands for the framework.
//!
//! # Example
//!
//! Using the [`with_embeds`] function to have the framework's help message use
//! embeds:
//!
//! ```rs,no_run
//! use serenity::ext::framework::help_commands;
//! use serenity::Client;
//! use std::env;
//!
//! let mut client = Client::login(&env::var("DISCORD_TOKEN").unwrap());
//! client.with_framework(|f| f
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
use super::command::CommandGroup;
use super::{Command, Framework};
use ::client::Context;
use ::model::Message;
use ::utils::Colour;

fn error_embed(msg: &Message, input: &str) {
    let _ = msg.channel_id
        .send_message(|m| m
        .embed(|e| e
            .colour(Colour::dark_red())
            .description(input)));
}

/// Posts formatted text displaying each individual command group and its commands.
///
/// # Examples
///
/// Use the command with `exec_help`:
///
/// ```rust
/// # use serenity::Client;
/// # let mut client = Client::login("token");
/// use serenity::ext::framework::help_commands;
///
/// client.with_framework(|f| f
///     .command("help", |c| c.exec_help(help_commands::plain)));
/// ```
pub fn plain<T: Command + Ord>(msg: &Message,
             framework: &Framework<T>,
             args: &[String]) -> Result<(), String> {
    if !args.is_empty() {
        let name = args.join(" ");

        for (ref group_name, ref group) in &framework.groups {
            let mut found = None::<(&String, &Arc<T>)>;

            for (ref command_name, ref command) in &group.commands {
                let with_prefix = if !group.prefix.is_empty() {
                    format!("{} {}", group.prefix, command_name)
                } else {
                    command_name.to_string()
                };

                if name == with_prefix || name == **command_name  {
                    if let Some(ref name) = framework.aliases.get(&name) {
                        let _ = msg.channel_id.say(&format!("Did you mean {:?}?", name));
                        return Ok(());
                    } else {
                        found = Some((command_name, *command));
                    }
                }
            }

            if let Some((command_name, command)) = found {
                if !command.help_available() {
                    let _ = msg.channel_id.say("**Error**: No help available.");
                    return Ok(());
                }

                let mut result = format!("**{}**\n", command_name);

                if let Some(ref desc) = command.desc() {
                    let _ = write!(result, "**Description:** {}\n", desc);
                }

                if let Some(ref usage) = command.usage() {
                    let _ = write!(result, "**Usage:** `{} {}`\n", command_name, usage);
                }

                if let Some(ref example) = command.example() {
                    let _ = write!(result, "**Sample usage:** `{} {}`\n", command_name, example);
                }

                if *group_name != "Ungrouped" {
                    let _ = write!(result, "**Group:** {}\n", group_name);
                }

                result.push_str("**Available:** ");
                result.push_str(if command.dm_only() {
                    "Only in DM"
                } else if command.guild_only() {
                    "Only in guilds"
                } else {
                    "In DM and guilds"
                });
                result.push_str("\n");

                let _ = msg.channel_id.say(&result);

                return Ok(());
            }
        }

        let _ = msg.channel_id.say(&format!("**Error**: Command `{}` not found.", name));

        return Ok(());
    }

    let mut result = "**Commands**\nTo get help with an individual command, pass its \
                      name as an argument to this command.\n\n"
        .to_string();

    let _ = ctx.channel_id.unwrap().send_message(|m| m
        .embed(|mut e| {
            e = e.colour(Colour::rosewater())
                .description("To get help with an individual command, pass its \
                              name as an argument to this command.");

    for (ref group_name, ref group) in &framework.groups {
        let mut group_help = "".to_owned();
 
        for (ref name, ref cmd) in &group.commands {
            if cmd.help_available() {
                let _ = write!(group_help, "`{}` ", name);
            }

            if has_commands {
                e = e.field(|f| f.name(&group_name).value(&desc));
            }
        }

        if group_help.len() > 0 {
            let _ = write!(result, "**{}:**", group_name);

            if !group.prefix.is_empty() {
                let _ = write!(result, "(prefix: `{}`): ", x);
            }
            result.push_str(&group_help);
            result.push('\n');
        }
    }
    
    let _ = msg.channel_id.say(&result);

    Ok(())
}


/// Posts an embed showing each individual command group and its commands.
///
/// # Examples
///
/// Use the command with `exec_help`:
///
/// ```rust
/// # use serenity::Client;
/// # let mut client = Client::login("token");
/// use serenity::ext::framework::help_commands;
///
/// client.with_framework(|f| f
///     .command("help", |c| c.exec_help(help_commands::with_embeds)));
/// ```
pub fn with_embeds<T: Command + Ord>(msg: &Message,
                   framework: &Framework<T>,
                   args: Vec<String>) -> Result<(), String> {
    if !args.is_empty() {
        let name = args.join(" ");

        for (ref group_name, ref group) in &framework.groups {
            let mut found = None::<(&String, &Arc<T>)>;

            for (ref command_name, ref command) in &group.commands {
                let with_prefix = if !group.prefix.is_empty() {
                    format!("{} {}", group.prefix, command_name)
                } else {
                    command_name.to_string()
                };

                if name == with_prefix {
                    if let Some(ref command_name) = framework.aliases.get(&name) {
                        error_embed(msg, &format!("Did you mean \"{}\"?", command_name));
                        return Ok(());
                    } else {
                        found = Some((command_name, *command));
                    }
                }
            }

            if let Some((ref command_name, ref command)) = found {
                if !command.help_available() {
                    error_embed(msg, "**Error**: No help available.");

                    return Ok(());
                }

                let _ = msg.channel_id.send_message(|m| {
                    m.embed(|e| {
                        let mut embed = e.colour(Colour::rosewater())
                            .title(command_name);
                        if let Some(ref desc) = command.desc() {
                            embed = embed.description(desc);
                        }

                        if let Some(ref usage) = command.usage() {
                            embed = embed.field(|f| f
                                .name("Usage")
                                .value(&format!("`{} {}`", command_name, usage)));
                        }

                        if let Some(ref example) = command.example() {
                            embed = embed.field(|f| f
                                .name("Sample usage")
                                .value(&format!("`{} {}`", command_name, example)));
                        }

                        if *group_name != "Ungrouped" {
                            embed = embed.field(|f| f
                                .name("Group")
                                .value(&group_name));
                        }

                        let available = if command.dm_only() {
                            "Only in DM"
                        } else if command.guild_only() {
                            "Only in guilds"
                        } else {
                            "In DM and guilds"
                        };

                        embed = embed.field(|f| f
                            .name("Available")
                            .value(available));

                        embed
                    })
                });

                return Ok(());
            }
        }

        let error_msg = format!("**Error**: Command `{}` not found.", name);
        error_embed(msg, &error_msg);

        return Ok(());
    }

    let _ = msg.channel_id.send_message(|m| m
        .embed(|mut e| {
            e = e.colour(Colour::rosewater())
                .description("To get help with an individual command, pass its \
                              name as an argument to this command.");

            for (ref group_name, ref group) in &framework.groups {
                let mut desc = String::new();

                if !group.prefix.is_empty() {
                    let _ = write!(desc, "Prefix: {}\n", group.prefix);
                }

                let mut has_commands = false;

                for (ref name, ref cmd) in &group.commands {
                    if cmd.help_available() {
                        let _ = write!(desc, "`{}`\n", name);

                        has_commands = true;
                    }

                    if has_commands {
                        e = e.field(|f| f.name(&group_name).value(&desc));
                    }
                }
            }

            e
        }));

    Ok(())
}
