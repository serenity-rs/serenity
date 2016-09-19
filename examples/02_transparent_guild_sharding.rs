extern crate serenity;

use serenity::Client;
use std::env;

// Serenity implements transparent sharding in a way that you do not need to
// manually handle separate processes or connections manually.
//
// Transparent sharding is useful for a shared state. Instead of having states
// with duplicated data, a shared state means all your data can be easily
// accessible across all shards.
//
// If your bot is on many guilds - or over the maximum of 2500 - then you
// should/must use guild sharding.
//
// This is an example file showing how guild sharding works. For this to
// properly be able to be seen in effect, your bot should be in at least 2
// guilds.
//
// Taking a scenario of 2 guilds, try saying "!ping" in one guild. It should
// print either "0" or "1" in the console. Saying "!ping" in the other guild,
// it should state the other number in the console. This confirms that guild
// sharding works.
fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::login_bot(&token);

    client.on_message(|context, message| {
        if message.content == "!ping" {
            {
                let connection = context.connection.lock().unwrap();

                if let Some(shard_info) = connection.shard_info() {
                    println!("Shard {}", shard_info[0]);
                }
            }

            let _ = context.say("Pong!");
        }
    });

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
    });

    // The total number of shards to use. The "current shard number" of a
    // connection - that is, the shard it is assigned to - is indexed at 0,
    // while the total shard count is indexed at 1.
    //
    // This means if you have 5 shards, your total shard count will be 5, while
    // each shard will be assigned numbers 0 through 4.
    let _ = client.start_shards(2);
}
