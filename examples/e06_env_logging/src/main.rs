use serenity::async_trait;
use serenity::prelude::*;
use tracing::{debug, error, info, instrument};

struct Handler;

use serenity::gateway::client::FullEvent;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, _: &Context, event: &FullEvent) {
        match event {
            FullEvent::Ready {
                data_about_bot, ..
            } => {
                // Log at the INFO level. This is a macro from the `tracing` crate.
                info!("{} is connected!", data_about_bot.user.name);
            },
            FullEvent::Resume {
                ..
            } => {
                // Log at the DEBUG level.
                //
                // In this example, this will not show up in the logs because DEBUG is
                // below INFO, which is the set debug level.
                debug!("Resumed");
            },
            _ => {},
        }
    }
}

#[tokio::main]
#[instrument]
async fn main() {
    // Call tracing_subscriber's initialize function, which configures `tracing` via environment
    // variables.
    //
    // For example, you can say to log all levels INFO and up via setting the environment variable
    // `RUST_LOG` to `INFO`.
    //
    // This environment variable is already preset if you use cargo-make to run the example.
    tracing_subscriber::fmt::init();

    // Configure the client with your Discord bot token in the environment.
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a valid token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client =
        Client::builder(token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
