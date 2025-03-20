use serenity::async_trait;
use serenity::builder::CreateMessage;
use serenity::gateway::client::FullEvent;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message {
                new_message, ..
            } => {
                if new_message.content == "!messageme" {
                    // If the `utils`-feature is enabled, then model structs will have a lot of
                    // useful methods implemented, to avoid using an often
                    // otherwise bulky Context, or even much lower-level `rest`
                    // method.
                    //
                    // In this case, you can direct message a User directly by simply calling a
                    // method on its instance, with the content of the message.
                    let builder = CreateMessage::new().content("Hello!");
                    let dm = new_message.author.id.dm(&ctx.http, builder).await;

                    if let Err(why) = dm {
                        println!("Error when direct messaging user: {why:?}");
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
    // Configure the client with your Discord bot token in the environment.
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a valid token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client =
        Client::builder(token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
