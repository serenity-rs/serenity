//! Requires the "cache", "methods", and "voice" features be enabled in your
//! Cargo.toml, like so:
//!
//! ```toml
//! [dependencies.serenity]
//! version = "*"
//! features = ["cache", "framework"]
//! ```
//!
//! Note that - due to bot users not being able to search - this example may
//! only be run under a user account.
//!
//! This particular example will automatically ensure that only the current user
//! may search; this acts as a "selfbot".

#[macro_use]
extern crate serenity;

use serenity::client::{CACHE, Client, Context};
use serenity::model::Message;
use serenity::utils::builder::{SortingMode, SortingOrder};
use std::env;

fn main() {
    let mut client = Client::login_user(&env::var("DISCORD_TOKEN").unwrap());

    client.with_framework(|f| f
        .configure(|c| c.prefix("~").on_mention(true))
        .command("search", |c| c
            .exec(search)
            .check(self_check)));

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
    });

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

fn self_check(_context: &mut Context, message: &Message) -> bool {
    message.author.id == CACHE.read().unwrap().user.id
}

command!(search(context, message, args) {
    let query = args.join(" ");

    if query.is_empty() {
        let _ = context.say("You must provide a query");

        return Ok(());
    }

    let guild_id = match message.guild_id() {
        Some(guild_id) => guild_id,
        None => {
            let _ = context.say("Only supports guilds");

            return Ok(());
        },
    };

    let channel_ids = {
        let cache = CACHE.read().unwrap();

        let guild = match cache.get_guild(guild_id) {
            Some(guild) => guild,
            None => {
                let _ = context.say("Guild data not found");

                return Ok(());
            },
        };

        guild.channels
            .values()
            .filter(|c| c.name.starts_with("search-"))
            .map(|c| c.id)
            .collect::<Vec<_>>()
    };

    let search = guild_id.search_channels(&channel_ids, |s| s
        .content(&query)
        .context_size(0)
        .has_attachment(true)
        .has_embed(true)
        .max_id(message.id.0 - 1)
        .sort_by(SortingMode::Timestamp)
        .sort_order(SortingOrder::Descending));

    let messages = match search {
        Ok(messages) => messages,
        Err(why) => {
            println!("Error performing search '{}': {:?}", query, why);

            let _ = context.say("Error occurred while searching");

            return Ok(());
        },
    };

    let _ = context.send_message(|m| m
        .content(&format!("Found {} total results", messages.total))
        .embed(move |mut e| {
            for (i, mut messages) in messages.results.into_iter().enumerate() {
                let mut message = &mut messages[0];
                message.content.truncate(1000);

                e = e.field(|f| f
                    .name(&format!("Result {}", i))
                    .value(&message.content));
            }

            e
        }));
});
