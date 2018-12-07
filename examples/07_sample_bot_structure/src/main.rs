//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```
mod commands;

use std::{collections::HashSet, env};

use serenity::{
    framework::StandardFramework,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
    http,
};

use log::{error, info};

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

fn main() {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    kankyo::load().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    env_logger::init();

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    let owners = match http::get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        },
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .owners(owners)
            .prefix("~"))
        .command("ping", |c| c.cmd(commands::meta::ping))
        .command("multiply", |c| c.cmd(commands::math::multiply))
        .command("quit", |c| c
            .cmd(commands::owner::quit)
            .owners_only(true)));

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
