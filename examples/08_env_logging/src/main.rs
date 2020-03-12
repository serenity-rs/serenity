use std::env;

use serenity::{
    async_trait,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};

use log::{debug, error, info};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        // Log at the INFO level. This is a macro from the `log` crate.
        info!("{} is connected!", ready.user.name);
    }

    async fn resume(&self, _: Context, resume: ResumedEvent) {
        // Log at the DEBUG level.
        //
        // In this example, this will not show up in the logs because DEBUG is
        // below INFO, which is the set debug level.
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}

#[tokio::main]
async fn main() {
    // Call env_logger's initialize function, which configures `log` via
    // environment variables.
    //
    // For example, you can say to log all levels INFO and up via setting the
    // environment variable `RUST_LOG` to `INFO`.
    env_logger::init();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = Client::new(&token, Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
