//! This example shows the recommended structure for a basic bot, while also demonstrating
//! how to use Serenity's low-level slash command functions.
//!
//! Individual commands introduce different features or concepts:
//! - `attachmentinput`: Accepting an `Attachment` as input
//! - `id`: Accepting a `User` as input
//! - `modal`: Using the `QuickModal` trait to respond with a modal
//! - `numberinput`: Accepting different types of numbers and restricting input
//! - `welcome`: Localizing commands
//!
//! Note that while it is possible to create and use slash commands with Serenity alone, we
//! recommend using Poise, a command framework with advanced features like edit tracking, single
//! function slash and prefix commands, and flexible argument parsing.
#![expect(clippy::collapsible_match)]
mod commands;

use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{Command, Interaction};
use serenity::model::event::FullEvent;
use serenity::model::id::GuildId;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::InteractionCreate {
                interaction, ..
            } => {
                // Check if the interaction is a (slash) command interaction.
                if let Interaction::Command(command) = interaction {
                    println!("Received command interaction: {command:#?}");

                    // Match the command name and execute its `run()` function, if one exists.
                    let content = match command.data.name.as_str() {
                        "ping" => Some(commands::ping::run(&command.data.options())),
                        "id" => Some(commands::id::run(&command.data.options())),
                        "attachmentinput" => {
                            Some(commands::attachmentinput::run(&command.data.options()))
                        },
                        "modal" => {
                            commands::modal::run(ctx, command).await.unwrap();
                            None
                        },
                        _ => Some("not implemented :(".to_string()),
                    };

                    // Respond to the command, if it was a valid command with a `run()` function
                    // that returns a String.
                    //
                    // Note that the `modal` command creates its own interaction response within
                    // its `run()` function, which is why it is not handled here.
                    if let Some(content) = content {
                        let data = CreateInteractionResponseMessage::new().content(content);
                        let builder = CreateInteractionResponse::Message(data);
                        if let Err(why) = command.create_response(&ctx.http, builder).await {
                            println!("Cannot respond to slash command: {why}");
                        }
                    }
                }
            },
            FullEvent::Ready {
                data_about_bot, ..
            } => {
                println!("{} is connected!", data_about_bot.user.name);

                let guild_id = GuildId::new(
                    env::var("GUILD_ID")
                        .expect("Expected GUILD_ID in environment")
                        .parse()
                        .expect("GUILD_ID must be an integer"),
                );

                // For simplicity, we register the commands here in this example, but the
                // recommended method is to use a prefix command. You only need to re-register when
                // commands change, which is not necessarily on startup, and the `Ready` event can
                // also fire multiple times. If you choose to keep this pattern, consider using an
                // AtomicBool, etc. to ensure registration only happens once.
                let commands = guild_id
                    .set_commands(&ctx.http, &[
                        commands::ping::register(),
                        commands::id::register(),
                        commands::welcome::register(),
                        commands::numberinput::register(),
                        commands::attachmentinput::register(),
                        commands::modal::register(),
                    ])
                    .await;

                println!("I now have the following guild slash commands: {commands:#?}");

                let global_command = Command::create_global_command(
                    &ctx.http,
                    commands::wonderful_command::register(),
                )
                .await;

                println!("I created the following global slash command: {global_command:#?}");
            },
            _ => {},
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a valid token in the environment");

    // Build the client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Arc::new(Handler))
        .await
        .expect("Err creating client");

    // Finally, start a single shard and begin listening to events.
    //
    // Shards will automatically attempt to reconnect, performing exponential backoff until
    // successful.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
