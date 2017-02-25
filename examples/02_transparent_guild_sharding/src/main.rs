extern crate serenity;

use serenity::Client;
use std::env;

// Serenity implements transparent sharding in a way that you do not need to
// manually handle separate processes or connections manually.
//
// Transparent sharding is useful for a shared cache. Instead of having caches
// with duplicated data, a shared cache means all your data can be easily
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
// it should cache the other number in the console. This confirms that guild
// sharding works.
fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::login_bot(&token);

    client.on_message(|ctx, msg| {
        if msg.content == "!ping" {
            // The current shard needs to be unlocked so it can be read from, as
            // multiple threads may otherwise attempt to read from or mutate it
            // concurrently.
            {
                let shard = ctx.shard.lock().unwrap();

                if let Some(shard_info) = shard.shard_info() {
                    println!("Shard {}", shard_info[0]);
                }
            }

            if let Err(why) = msg.channel_id.say("Pong!") {
                println!("Error sending message: {:?}", why);
            }
        }
    });

    client.on_ready(|_ctx, ready| {
        println!("{} is connected!", ready.user.name);
    });

    // The total number of shards to use. The "current shard number" of a
    // shard - that is, the shard it is assigned to - is indexed at 0,
    // while the total shard count is indexed at 1.
    //
    // This means if you have 5 shards, your total shard count will be 5, while
    // each shard will be assigned numbers 0 through 4.
    if let Err(why) = client.start_shards(2) {
        println!("Client error: {:?}", why);
    }
}
