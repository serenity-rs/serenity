use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::Write;
use super::{Command, CommandGroup, CommandOrAlias};
use super::command::InternalCommand;
use ::client::Context;
use ::model::Message;
use ::utils::Colour;

fn error_embed(ctx: &Context, message: &Message, input: &str) {
    let _ = ctx.send_message(message.channel_id, |m| m
        .embed(|e| e
            .colour(Colour::dark_red())
            .description(input)));
}

fn remove_aliases(cmds: &HashMap<String, CommandOrAlias>) -> HashMap<String, &InternalCommand> {
    let mut result = HashMap::new();

    for (n, v) in cmds {
        if let CommandOrAlias::Command(ref cmd) = *v {
            result.insert(n.to_owned(), cmd);
        }
    }

    result
}

pub fn with_embeds(ctx: &Context,
                   message: &Message,
                   groups: HashMap<String, Arc<CommandGroup>>,
                   args: Vec<String>) -> Result<(), String> {
    if !args.is_empty() {
        let name = args.join(" ");

        for (group_name, group) in groups {
            let mut found: Option<(&String, &InternalCommand)> = None;

            if let Some(ref prefix) = group.prefix {
                for (command_name, command) in &group.commands {
                    if name == format!("{} {}", prefix, command_name) || name == *command_name {
                        match *command {
                            CommandOrAlias::Command(ref cmd) => {
                                found = Some((command_name, cmd));
                            },
                            CommandOrAlias::Alias(ref name) => {
                                error_embed(ctx, message, &format!("Did you mean {:?}?", name));
                                return Ok(());
                            }
                        }
                    }
                }
            } else {
                for (command_name, command) in &group.commands {
                    if name == command_name[..] {
                        match *command {
                            CommandOrAlias::Command(ref cmd) => {
                                found = Some((command_name, cmd));
                            },
                            CommandOrAlias::Alias(ref name) => {
                                error_embed(ctx, message, &format!("Did you mean {:?}?", name));
                                return Ok(());
                            }
                        }
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
                .description("To get help with an individual command, pass its \
                              name as an argument to this command.");

            for (group_name, group) in groups {
                let mut desc = String::new();

                if let Some(ref x) = group.prefix {
                    let _ = write!(desc, "Prefix: {}\n", x);
                }

                desc.push_str("Commands:\n");

                let mut no_commands = true;

                for (n, cmd) in remove_aliases(&group.commands) {
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
                    if name == format!("{} {}", prefix, command_name) || name == *command_name  {
                        match *command {
                            CommandOrAlias::Command(ref cmd) => {
                                found = Some((command_name, cmd));
                            },
                            CommandOrAlias::Alias(ref name) => {
                                let _ = ctx.say(&format!("Did you mean {:?}?", name));
                                return Ok(());
                            }
                        }
                    }
                }
            } else {
                for (command_name, command) in &group.commands {
                    if name == command_name[..] {
                        match *command {
                            CommandOrAlias::Command(ref cmd) => {
                                found = Some((command_name, cmd));
                            },
                            CommandOrAlias::Alias(ref name) => {
                                let _ = ctx.say(&format!("Did you mean {:?}?", name));
                                return Ok(());
                            }
                        }
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
                    let _ = write!(result, "**Usage:** `{} {}`\n", command_name, usage);
                }

                if let Some(ref example) = command.example {
                    let _ = write!(result, "**Sample usage:** `{} {}`\n", command_name, example);
                }

                if group_name != "Ungrouped" {
                    let _ = write!(result, "**Group:** {}\n", group_name);
                }

                result.push_str("**Available:** ");
                result.push_str(if command.dm_only {
                    "Only in DM"
                } else if command.guild_only {
                    "Only in guilds"
                } else {
                    "In DM and guilds"
                });
                result.push_str("\n");

                let _ = ctx.say(&result);

                return Ok(());
            }
        }

        let _ = ctx.say(&format!("**Error**: Command `{}` not found.", name));

        return Ok(());
    }

    let mut result = "**Commands**\nTo get help with an individual command, pass its \
                      name as an argument to this command.\n\n"
        .to_string();

    for (group_name, group) in groups {
        let _ = write!(result, "**{}:** ", group_name);

        if let Some(ref x) = group.prefix {
            let _ = write!(result, "(prefix: `{}`): ", x);
        }

        let mut no_commands = true;

        for (n, cmd) in remove_aliases(&group.commands) {
            if cmd.help_available {
                let _ = write!(result, "`{}` ", n);

                no_commands = false;
            }
        }

        if no_commands {
            result.push_str("*[No Commands]*");
        }

        result.push('\n');
    }

    let _ = ctx.say(&result);

    Ok(())
}
