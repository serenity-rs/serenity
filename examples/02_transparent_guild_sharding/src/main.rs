use std::env;

use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

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
struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
       if msg.content == "!ping" {
            println!("Shard {}", ctx.shard_id);

            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!") {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}


fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler).expect("Err creating client");

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
