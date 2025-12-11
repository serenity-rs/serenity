//! This example will showcase the beauty of collectors. They allow to await messages or reactions
//! from a user in the middle of a control flow, one being a command.
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use serenity::async_trait;
use serenity::collector::{CollectMessages, CollectReactions, MessageCollector};
// Collectors are streams, that means we can use `StreamExt` and `TryStreamExt`.
use serenity::futures::stream::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;

struct Handler;

use serenity::gateway::client::FullEvent;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message {
                new_message, ..
            } => {
                let mut score = 0u32;
                let _ = new_message
                    .reply(&ctx.http, "How was that crusty crab called again? 10 seconds time!")
                    .await;

                // There is a method implemented for some models to conveniently collect replies.
                // They return a builder that can be turned into a Stream, or here,
                // where we can await a single reply
                let collector =
                    new_message.author.id.collect_messages(ctx).timeout(Duration::from_secs(10));
                if let Some(answer) = collector.await {
                    if answer.content.to_lowercase() == "ferris" {
                        let _ = answer.reply(&ctx.http, "That's correct!").await;
                        score += 1;
                    } else {
                        let _ = answer.reply(&ctx.http, "Wrong, it's Ferris!").await;
                    }
                } else {
                    let _ = new_message.reply(&ctx.http, "No answer within 10 seconds.").await;
                };

                let react_msg = new_message
                    .reply(&ctx.http, "React with the reaction representing 1, you got 10 seconds!")
                    .await
                    .unwrap();

                // The message model can also be turned into a Collector to collect reactions on it.
                let collector = react_msg
                    .id
                    .collect_reactions(ctx)
                    .timeout(Duration::from_secs(10))
                    .author_id(new_message.author.id);

                if let Some(reaction) = collector.await {
                    let _ = if reaction.emoji.as_data() == "1️⃣" {
                        score += 1;
                        new_message.reply(&ctx.http, "That's correct!").await
                    } else {
                        new_message.reply(&ctx.http, "Wrong!").await
                    };
                } else {
                    let _ = new_message.reply(&ctx.http, "No reaction within 10 seconds.").await;
                };

                let _ = new_message.reply(&ctx.http, "Write 5 messages in 10 seconds").await;

                // We can create a collector from scratch too using this builder future.
                let collector = MessageCollector::new(ctx)
                // Only collect messages by this user.
                    .author_id(new_message.author.id)
                    .channel_id(new_message.channel_id)
                    .timeout(Duration::from_secs(10))
                    // Build the collector.
                    .stream()
                    .take(5);

                // Let's acquire borrow HTTP to send a message inside the `async move`.
                let http = &ctx.http;

                // We want to process each message and get the length. There are a couple of ways to
                // do this. Folding the stream with `fold` is one way.
                //
                // Using `then` to first reply and then create a new stream with all messages is
                // another way to do it, which can be nice if you want to further
                // process the messages.
                //
                // If you don't want to collect the stream, `for_each` may be sufficient.
                let collected: Vec<_> = collector
                    .then(|msg| async move {
                        let _ = msg.reply(http, format!("I repeat: {}", msg.content)).await;

                        msg
                    })
                    .collect()
                    .await;

                if collected.len() >= 5 {
                    score += 1;
                }

                // We can also collect arbitrary events using the collect() function. For example,
                // here we collect updates to the messages that the user sent above
                // and check for them updating all 5 of them.
                let mut collector = serenity::collector::collect(ctx, move |event| match event {
                    // Only collect MessageUpdate events for the 5 MessageIds we're interested in.
                    Event::MessageUpdate(event)
                        if collected.iter().any(|msg| event.message.id == msg.id) =>
                    {
                        Some(event.message.id)
                    },
                    _ => None,
                })
                .take_until(Box::pin(tokio::time::sleep(Duration::from_secs(20))));

                let _ = new_message
                    .reply(&ctx.http, "Edit each of those 5 messages in 20 seconds")
                    .await;
                let mut edited = HashSet::new();
                while let Some(edited_message_id) = collector.next().await {
                    edited.insert(edited_message_id);
                    if edited.len() >= 5 {
                        break;
                    }
                }

                if edited.len() >= 5 {
                    score += 1;
                    let _ = new_message.reply(&ctx.http, "Great! You edited 5 out of 5").await;
                } else {
                    let _ = new_message
                        .reply(&ctx.http, format!("You only edited {} out of 5", edited.len()))
                        .await;
                }

                let _ = new_message
                    .reply(
                        &ctx.http,
                        format!("TIME'S UP! You completed {score} out of 4 tasks correctly!"),
                    )
                    .await;
            },
            FullEvent::Ready {
                data_about_bot, ..
            } => {
                println!("{} is connected!", data_about_bot.user.name);
            },
            _ => {},
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a valid token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(token, intents)
        .event_handler(Arc::new(Handler))
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
