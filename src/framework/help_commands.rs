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

pub fn plain<T: Command>(msg: &Message,
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

    for (ref group_name, ref group) in &framework.groups {
        let _ = write!(result, "**{}:** ", group_name);

        if !group.prefix.is_empty() {
            let _ = write!(result, "(prefix: `{}`): ", group.prefix);
        }

        let mut no_commands = true;
        
        for (ref name, ref cmd) in &group.commands {
            if cmd.help_available() {
                let _ = write!(result, "`{}` ", name);

                no_commands = false;
            }

            if no_commands {
                result.push_str("*[No Commands]*");
            }

            result.push('\n');
        }
    }
    
    let _ = msg.channel_id.say(&result);

    Ok(())
}

pub fn with_embeds<T: Command>(msg: &Message,
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

                let mut no_commands = true;

                for (ref name, ref cmd) in &group.commands {
                    if cmd.help_available() {
                        let _ = write!(desc, "`{}`\n", name);

                        no_commands = false;
                    }

                    if no_commands {
                        let _ = write!(desc, "*[No commands]*");
                    }

                    e = e.field(|f| f.name(&group_name).value(&desc));
                }
            }

            e
        }));

    Ok(())
}

