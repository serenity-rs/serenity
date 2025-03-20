mod commands;

use std::env;

use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::gateway::client::FullEvent;
use serenity::model::application::{Command, Interaction};
use serenity::model::id::GuildId;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        // clippy can't decide between if it wants it collapsed, or if it wants you to use if let
        // because its a single pattern.
        #[expect(clippy::collapsible_match)]
        match event {
            FullEvent::InteractionCreate {
                interaction, ..
            } => {
                if let Interaction::Command(command) = interaction {
                    println!("Received command interaction: {command:#?}");

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

                let global_command =
                    Command::create_global_command(&ctx.http, commands::wonderful_command::register())
                        .await;

                println!("I created the following global slash command: {global_command:#?}");

                println!("{} is connected!", data_about_bot.user.name);
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

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
