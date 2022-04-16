use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::Timestamp;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            // The create message builder allows you to easily create embeds and messages
            // using a builder syntax.
            // This example will create a message that says "Hello, World!", with an embed that has
            // a title, description, an image, three fields, and a footer.
            let msg = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.content("Hello, World!")
                        .embed(|e| {
                            e.title("This is a title")
                                .description("This is a description")
                                .image("attachment://ferris_eyes.png")
                                .fields(vec![
                                    ("This is the first field", "This is a field body", true),
                                    ("This is the second field", "Both fields are inline", true),
                                ])
                                .field("This is the third field", "This is not an inline field", false)
                                .footer(|f| f.text("This is a footer"))
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(Timestamp::now())
                        })
                        .add_file("./ferris_eyes.png")
                })
                .await;

            if let Err(why) = msg {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
