//! This is an example showing how to interact with the client's `ShardManager`,
//! which is a struct that can be used to interact with shards. This allows an
//! easy method of retrieving shards' current status, restarting them, or
//! shutting them down.
//!
//! In this example, we run two shards; this means that there will be two
//! WebSocket connections to Discord, and each will receive events for
//! _approximately_ 1/2 of all guilds that the bot is on.
//!
//! This isn't particularly useful for small bots, but is useful for large bots
//! that may need to split load on separate VPSs or dedicated servers.
//! Additionally, Discord requires that there be at least one shard for every
//! 2500 guilds that a bot is on.
//!
//! For the purposes of this example, we'll print the current statuses of the
//! two shards to the terminal every 30 seconds. This includes the ID of the
//! shard, the current connection stage, (e.g. "Connecting" or "Connected"), and
//! the approximate WebSocket latency (time between when a heartbeat is sent to
//! Discord and when a heartbeat acknowledgement is received).
//!
//! # Notes
//!
//! Note that it may take a minute or more for a latency to be recorded or to
//! update, depending on how often Discord tells the client to send a heartbeat.
use std::{env, thread, time::Duration};

use serenity::{
    model::gateway::Ready,
    prelude::*,
};

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            // Note that array index 0 is 0-indexed, while index 1 is 1-indexed.
            //
            // This may seem unintuitive, but it models Discord's behaviour.
            println!(
                "{} is connected on shard {}/{}!",
                ready.user.name,
                shard[0],
                shard[1],
            );
        }
    }
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = Client::new(&token, Handler).expect("Err creating client");

    // Here we clone a lock to the Shard Manager, and then move it into a new
    // thread. The thread will unlock the manager and print shards' status on a
    // loop.
    let manager = client.shard_manager.clone();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(30));

            let lock = manager.lock();
            let shard_runners = lock.runners.lock();

            for (id, runner) in shard_runners.iter() {
                println!(
                    "Shard ID {} is {} with a latency of {:?}",
                    id,
                    runner.stage,
                    runner.latency,
                );
            }
        }
    });

    // Start two shards. Note that there is an ~5 second ratelimit period
    // between when one shard can start after another.
    if let Err(why) = client.start_shards(2) {
        println!("Client error: {:?}", why);
    }
}
