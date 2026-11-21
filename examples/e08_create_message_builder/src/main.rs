//! Serenity has many builders that make configuring Discord HTTP requests easy.
//!
//! This example demonstrates the use of several builders to create and send a message that
//! includes an embed with a title, description, an image, three fields, and a footer.
use std::sync::Arc;

use serenity::async_trait;
use serenity::builder::{CreateAttachment, CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::model::Timestamp;
use serenity::model::event::FullEvent;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message {
                new_message, ..
            } => {
                if new_message.content == "!hello" {
                    // First, use the `CreateEmbedFooter` builder to create a footer for the embed.
                    // This can also be done inside of the `footer()` method below, but doing it
                    // separately can make code easier to read, especially with nested builders.
                    let footer = CreateEmbedFooter::new("This is a footer");
                    // Now we use the `CreateEmbed` builder to construct the embed.
                    let embed = CreateEmbed::new()
                        .title("This is a title")
                        .description("This is a description")
                        .image("attachment://ferris_eyes.png", Some("Ferris with big eyes".into()))
                        .fields([
                            ("This is the first field", "This is a field body", true),
                            ("This is the second field", "Both fields are inline", true),
                        ])
                        .field("This is the third field", "This is not an inline field", false)
                        .footer(footer)
                        // Add a timestamp for the current time. The timestamp can also be parsed
                        // from an RFC 3339 date and time string.
                        .timestamp(Timestamp::now());
                    // Finally, we construct the message itself with the `CreateMessage` builder.
                    let builder = CreateMessage::new()
                        .content("Hello, World!")
                        .embed(embed)
                        // If we forget to use the `CreateAttachment` builder and attach the file,
                        // the `attachment://ferris_eyes.png` link above will be invalid.
                        .add_file(CreateAttachment::path("./ferris_eyes.png".as_ref()).unwrap());

                    if let Err(why) = new_message.channel_id.send_message(&ctx.http, builder).await
                    {
                        println!("Error sending message: {why:?}");
                    }
                }
            },
            FullEvent::Ready {
                data_about_bot, ..
            } => {
                println!("{} is connected!", data_about_bot.user.name);
            },
            _ => {},
        }
    }
}

#[tokio::main]
async fn main() {
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a valid token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Arc::new(Handler))
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
