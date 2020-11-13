use std::{env, path::Path};

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    http::AttachmentType,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            // The create message builder allows you to easily create embeds and messages
            // using a builder syntax.
            // This example will create a message that says "Hello, World!", with an embed that has
            // a title, description, three fields, and a footer.
            let msg = msg.channel_id.send_message(&ctx.http, |m| {
                m.content("Hello, World!");
                m.embed(|e| {
                    e.title("This is a title");
                    e.description("This is a description");
                    e.image("attachment://ferris_eyes.png");
                    e.fields(vec![
                        ("This is the first field", "This is a field body", true),
                        ("This is the second field", "Both of these fields are inline", true),
                    ]);
                    e.field("This is the third field", "This is not an inline field", false);
                    e.footer(|f| {
                        f.text("This is a footer");

                        f
                    });

                    e
                });
                m.add_file(AttachmentType::Path(Path::new("./ferris_eyes.png")));
                m
            }).await;

            if let Err(why) = msg {
                println!("Error sending message: {:?}", why);
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
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
