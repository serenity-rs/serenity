pub use super::{Command, CommandGroup};

use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::Write;
use ::client::Context;
use ::model::Message;
use ::utils::Colour;

fn error_embed(ctx: &Context, message: &Message, input: &str) {
    let _ = ctx.send_message(message.channel_id, |m| m
        .embed(|e| e
            .colour(Colour::dark_red())
            .description(input)));
}

pub fn with_embeds(ctx: &Context,
                   message: &Message,
                   groups: HashMap<String, Arc<CommandGroup>>,
                   args: Vec<String>) -> Result<(), String> {
    if !args.is_empty() {
        let name = args.join(" ");

        for (group_name, group) in groups {
            let mut found: Option<(&String, &Command)> = None;

            if let Some(ref prefix) = group.prefix {
                for (command_name, command) in &group.commands {
                    if name == format!("{} {}", prefix, command_name) {
                        found = Some((command_name, command));
                    }
                }
            } else {
                for (command_name, command) in &group.commands {
                    if name == command_name[..] {
                        found = Some((command_name, command));
                    }
                }
            };

            if let Some((command_name, command)) = found {
                if !command.help_available {
                    error_embed(ctx, message, "**Error**: No help available.");
                    return Ok(());
                }

                let _ = ctx.send_message(message.channel_id, |m| {
                    m.embed(|e| {
                        let mut embed = e.colour(Colour::rosewater())
                            .title(command_name);
                        if let Some(ref desc) = command.desc {
                            embed = embed.field(|f| {
                                f.name("Description")
                                    .value(desc)
                                    .inline(false)
                            });
                        }

                        if let Some(ref usage) = command.usage {
                            embed = embed.field(|f| {
                                f.name("Usage")
                                    .value(&format!("{} {}", command_name, usage))
                            });
                        }

                        if group_name != "Ungrouped" {
                            embed = embed.field(|f| {
                                f.name("Group")
                                    .value(&group_name)
                            });
                        }

                        let available = if command.dm_only {
                            "Only in DM"
                        } else if command.guild_only {
                            "Only in guilds"
                        } else {
                            "In DM and guilds"
                        };

                        embed = embed.field(|f| {
                            f.name("Available")
                                .value(available)
                        });

                        embed
                    })
                });

                return Ok(());
            }
        }

        let error_msg = format!("**Error**: Command `{}` not found.", name);
        error_embed(ctx, message, &error_msg);

        return Ok(());
    }
    let _ = ctx.send_message(message.channel_id, |m| {
        m.embed(|mut e| {
            e = e.colour(Colour::rosewater())
                .description("To get help about individual command, pass its \
                              name as an argument to this command.");

            for (group_name, group) in groups {
                let mut desc = String::new();

                if let Some(ref x) = group.prefix {
                    let _ = write!(desc, "Prefix: {}\n", x);
                }

                let mut no_commands = true;
                let _ = write!(desc, "Commands:\n");

                for (n, cmd) in &group.commands {
                    if cmd.help_available {
                        let _ = write!(desc, "`{}`\n", n);

                        no_commands = false;
                    }
                }

                if no_commands {
                    let _ = write!(desc, "*[No commands]*");
                }

                e = e.field(|f| f.name(&group_name).value(&desc));
            }

            e
        })
    });

    Ok(())
}

pub fn plain(ctx: &Context,
             _: &Message,
             groups: HashMap<String, Arc<CommandGroup>>,
             args: Vec<String>) -> Result<(), String> {
    if !args.is_empty() {
        let name = args.join(" ");

        for (group_name, group) in groups {
            let mut found: Option<(&String, &Command)> = None;
            if let Some(ref prefix) = group.prefix {
                for (command_name, command) in &group.commands {
                    if name == format!("{} {}", prefix, command_name) {
                        found = Some((command_name, command));
                    }
                }
            } else {
                for (command_name, command) in &group.commands {
                    if name == command_name[..] {
                        found = Some((command_name, command));
                    }
                }
            };

            if let Some((command_name, command)) = found {
                if !command.help_available {
                    let _ = ctx.say("**Error**: No help available.");
                    return Ok(());
                }

                let mut result = format!("**{}**\n", command_name);

                if let Some(ref desc) = command.desc {
                    let _ = write!(result, "**Description:** {}\n", desc);
                }

                if let Some(ref usage) = command.usage {
                    let _ = write!(result, "**Usage:** {}\n", usage);
                }

                if group_name != "Ungrouped" {
                    let _ = write!(result, "**Group:** {}\n", group_name);
                }

                let available = if command.dm_only {
                    "Only in DM"
                } else if command.guild_only {
                    "Only in guilds"
                } else {
                    "In DM and guilds"
                };

                let _ = write!(result, "**Available:** {}\n", available);
                let _ = ctx.say(&result);

                return Ok(());
            }
        }

        let _ = ctx.say(&format!("**Error**: Command `{}` not found.", name));

        return Ok(());
    }
    let mut result = "**Commands**\nTo get help about individual command, pass \
                      its name as an argument to this command.\n\n"
        .to_string();

    for (group_name, group) in groups {
        let mut desc = String::new();

        if let Some(ref x) = group.prefix {
            let _ = write!(desc, "(prefix: `{}`): ", x);
        }

        let mut no_commands = true;

        for (n, cmd) in &group.commands {
            if cmd.help_available {
                let _ = write!(desc, "`{}` ", n);
                no_commands = false;
            }
        }

        if no_commands {
            let _ = write!(desc, "*[No commands]*");
        }

        let _ = write!(result, "**{}:** {}\n", group_name, desc);
    }

    let _ = ctx.say(&result);

    Ok(())
}
