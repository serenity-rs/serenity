//! This example demonstrates how to use the `tracing` crate with Serenity.
//!
//! `tracing` is a framework for instrumenting Rust programs to collect structured, event-based
//! diagnostic information. In this example, we show how to use `tracing` crate macros to easily
//! construct events at different logging levels.
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::event::FullEvent;
use serenity::prelude::*;
use tracing::{debug, error, info, instrument};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, _: &Context, event: &FullEvent) {
        match event {
            FullEvent::Ready {
                data_about_bot, ..
            } => {
                // Log at the INFO level, using a macro from the `tracing` crate.
                info!("{} is connected!", data_about_bot.user.name);
            },
            FullEvent::Resume {
                ..
            } => {
                // Log at the DEBUG level.
                //
                // In this example, this will not show up in the logs because DEBUG is below INFO,
                // which is the set RUST_LOG level.
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
        // Log at the ERROR level.
        error!("Client error: {:?}", why);
    }
}
