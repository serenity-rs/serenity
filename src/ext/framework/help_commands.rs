pub use ext::framework::command::{Command, CommandType, CommandGroup};

use ::utils::Colour;
use std::collections::HashMap;
use std::sync::Arc;
use ::client::Context;
use ::model::Message;

fn error_embed(ctx: &Context, message: &Message, input: &str) {
    let _ = ctx.send_message(message.channel_id, |m| {
        m.embed(|e| {
            e.colour(Colour::dark_red())
                .description(input)
        })
    });
}

#[allow(dead_code)]
pub fn with_embeds(ctx: &Context,
                   message: &Message,
                   groups: HashMap<String, Arc<CommandGroup>>,
                   args: Vec<String>) {
    if args.len() > 0 {
        let name = args.join(" ");
        for (_, group) in groups {
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
                    error_embed(&ctx, &message, "**Error**: No help available.");
                    return;
                }
                let _ = ctx.send_message(message.channel_id, |m| {
                    m.embed(|e| {
                        let mut embed = e.colour(Colour::rosewater())
                            .title(command_name);
                        if let Some(ref desc) = command.desc {
                            embed = embed.field(|f| {
                                f.name("Description")
                                    .value(&desc)
                                    .inline(false)
                            });
                        }
                        if let Some(ref usage) = command.usage {
                            embed = embed.field(|f| {
                                f.name("Usage")
                                    .value(&format!("{} {}", command_name, usage))
                            });
                        }
                        embed = embed.field(|f| {
                            f.name("Available")
                                .value(if command.dm_only {
                                    "Only in DM"
                                } else if command.guild_only {
                                    "Only in guilds"
                                } else {
                                    "Everywhere"
                                })
                        });
                        embed
                    })
                });
                return;
            }
        }
        error_embed(&ctx,
                    &message,
                    &format!("**Error**: Command `{}` not found.", name));
        return;
    }
    let _ = ctx.send_message(message.channel_id, |m| {
        m.embed(|e| {
            let mut embed = e.colour(Colour::blurple())
                .description("To get help about individual command, pass its name as an argument \
                              to this command.");
            for (name, group) in groups {
                let mut desc = String::new();
                if let Some(ref x) = group.prefix {
                    desc.push_str(&format!("Prefix: {}\n", x));
                }
                let mut no_commands = true;
                desc.push_str("Commands:\n");
                for (n, cmd) in &group.commands {
                    if cmd.help_available {
                        desc.push_str(&format!("`{}`\n", n));
                        no_commands = false;
                    }
                }
                if no_commands {
                    desc.push_str("*[No commands]*");
                }
                embed = embed.field(|f| {
                    f.name(&name)
                        .value(&desc)
                });
            }
            embed
        })
    });
}

pub fn plain(ctx: &Context,
             _: &Message,
             groups: HashMap<String, Arc<CommandGroup>>,
             args: Vec<String>) {
    if args.len() > 0 {
        let name = args.join(" ");
        for (_, group) in groups {
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
                    return;
                }
                let mut result = format!("**{}**\n", command_name);
                if let Some(ref desc) = command.desc {
                    result.push_str(&format!("**Description:** {}", desc));
                }
                if let Some(ref usage) = command.usage {
                    result.push_str(&format!("**Usage:** {}", usage));
                }
                result.push_str(&format!("**Available:** {}",
                                         if command.dm_only {
                                             "Only in DM"
                                         } else if command.guild_only {
                                             "Only in guilds"
                                         } else {
                                             "Everywhere"
                                         }));
                let _ = ctx.say(&result);
                return;
            }
        }
        let _ = ctx.say(&format!("**Error**: Command `{}` not found.", name));
        return;
    }
    let mut result = "**Commands**\nTo get help about individual command, pass its name as an \
                  argument to this command.\n\n"
        .to_string();
    for (name, group) in groups {
        let mut desc = String::new();
        if let Some(ref x) = group.prefix {
            desc.push_str(&format!("(prefix: `{}`): ", x));
        }
        let mut no_commands = true;
        for (n, cmd) in &group.commands {
            if cmd.help_available {
                desc.push_str(&format!("`{}` ", n));
                no_commands = false;
            }
        }
        if no_commands {
            desc.push_str("*[No commands]*");
        }
        result.push_str(&format!("**{}:** {}\n", name, desc));
    }
    let _ = ctx.say(&result);
}
