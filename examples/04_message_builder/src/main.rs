extern crate serenity;

use serenity::Client;
use serenity::utils::MessageBuilder;
use std::env;

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::login_bot(&token);

    client.on_message(|context, message| {
        if message.content == "!ping" {
            let channel = match context.get_channel(message.channel_id) {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);

                    return;
                },
            };

            // The message builder allows for creating a message by mentioning
            // users dynamically, pushing "safe" versions of content (such as
            // bolding normalized content), displaying emojis, and more.
            let response = MessageBuilder::new()
                .push("User ")
                .push_bold_safe(&message.author.name)
                .push(" used the 'ping' command in the ")
                .mention(channel)
                .push(" channel")
                .build();

            if let Err(why) = context.say(&response) {
                println!("Error sending message: {:?}", why);
            }
        }
    });

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
    });

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
