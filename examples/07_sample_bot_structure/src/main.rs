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

#[macro_use]
extern crate serenity;

mod commands;

use serenity::prelude::*;
use serenity::framework::StandardFramework;
use std::env;

struct Handler; impl EventHandler for Handler {}

fn main() {
    let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap(), Handler);

    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .command("ping", |c| c.exec(commands::meta::ping))
        .command("latency", |c| c.exec(commands::meta::latency))
        .command("multiply", |c| c.exec(commands::math::multiply)));

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
