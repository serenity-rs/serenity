//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/zeyla/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```

#[macro_use] extern crate log;
#[macro_use] extern crate serenity;

extern crate env_logger;
extern crate kankyo;

mod commands;

use serenity::framework::StandardFramework;
use serenity::model::event::ResumedEvent;
use serenity::model::Ready;
use serenity::prelude::*;
use std::env;

struct Handler;

impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    fn on_resume(&self, _: Context, _: ResumedEvent) {
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
    env_logger::init().expect("Failed to initialize env_logger");

    let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap(), Handler);

    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .command("ping", |c| c.exec(commands::meta::ping))
        .command("latency", |c| c.exec(commands::meta::latency))
        .command("multiply", |c| c.exec(commands::math::multiply)));

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
