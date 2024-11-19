use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

// Serenity implements transparent sharding in a way that you do not need to handle separate
// processes or connections manually.
//
// Transparent sharding is useful for a shared cache. Instead of having caches with duplicated
// data, a shared cache means all your data can be easily accessible across all shards.
//
// If your bot is on many guilds - or over the maximum of 2500 - then you should/must use guild
// sharding.
//
// This is an example file showing how guild sharding works. For this to properly be able to be
// seen in effect, your bot should be in at least 2 guilds.
//
// Taking a scenario of 2 guilds, try saying "!ping" in one guild. It should print either "0" or
// "1" in the console. Saying "!ping" in the other guild, it should cache the other number in the
// console. This confirms that guild sharding works.
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            println!("Shard {}", ctx.shard_id);

            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a valid token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client =
        Client::builder(token, intents).event_handler(Handler).await.expect("Err creating client");

    // The total number of shards to use. The "current shard number" of a shard - that is, the
    // shard it is assigned to - is indexed at 0, while the total shard count is indexed at 1.
    //
    // This means if you have 5 shards, your total shard count will be 5, while each shard will be
    // assigned numbers 0 through 4.
    if let Err(why) = client.start_shards(2).await {
        println!("Client error: {why:?}");
    }
}
