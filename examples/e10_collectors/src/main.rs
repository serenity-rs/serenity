//! This example will showcase the beauty of collectors.
//! They allow to await messages or reactions from a user in the middle
//! of a control flow, one being a command.
use std::collections::HashSet;
use std::env;
use std::time::Duration;

use serenity::async_trait;
use serenity::collector::MessageCollector;
use serenity::framework::standard::macros::{command, group, help};
use serenity::framework::standard::{
    help_commands,
    Args,
    CommandGroup,
    CommandResult,
    HelpOptions,
    StandardFramework,
};
// Collectors are streams, that means we can use `StreamExt` and `TryStreamExt`.
use serenity::futures::stream::StreamExt;
use serenity::http::Http;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[group("collector")]
#[commands(challenge)]
struct Collector;

#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    // We will fetch your bot's id.
    let bot_id = match http.get_current_user().await {
        Ok(info) => info.id,
        Err(why) => panic!("Could not access user info: {:?}", why),
    };

    let framework = StandardFramework::new().help(&MY_HELP).group(&COLLECTOR_GROUP);

    framework.configure(|c| {
        c.with_whitespace(true).on_mention(Some(bot_id)).prefix("~").delimiters(vec![", ", ","])
    });

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn challenge(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let mut score = 0u32;
    let _ = msg.reply(ctx, "How was that crusty crab called again? 10 seconds time!").await;

    // There is a method implemented for some models to conveniently collect replies.
    // They return a builder that can be turned into a Stream, or here, where we can
    // await a single reply
    let collector = msg.author.reply_collector(&ctx.shard).timeout(Duration::from_secs(10));
    if let Some(answer) = collector.collect_single().await {
        if answer.content.to_lowercase() == "ferris" {
            let _ = answer.reply(ctx, "That's correct!").await;
            score += 1;
        } else {
            let _ = answer.reply(ctx, "Wrong, it's Ferris!").await;
        }
    } else {
        let _ = msg.reply(ctx, "No answer within 10 seconds.").await;
    };

    let react_msg = msg
        .reply(ctx, "React with the reaction representing 1, you got 10 seconds!")
        .await
        .unwrap();

    // The message model can also be turned into a Collector to collect reactions on it.
    let collector = react_msg
        .reaction_collector(&ctx.shard)
        .timeout(Duration::from_secs(10))
        .author_id(msg.author.id);

    if let Some(reaction) = collector.collect_single().await {
        let _ = if reaction.emoji.as_data() == "1️⃣" {
            score += 1;
            msg.reply(ctx, "That's correct!").await
        } else {
            msg.reply(ctx, "Wrong!").await
        };
    } else {
        let _ = msg.reply(ctx, "No reaction within 10 seconds.").await;
    };

    let _ = msg.reply(ctx, "Write 5 messages in 10 seconds").await;

    // We can create a collector from scratch too using this builder future.
    let collector = MessageCollector::new(&ctx.shard)
    // Only collect messages by this user.
        .author_id(msg.author.id)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(10))
        // Build the collector.
        .collect_stream()
        .take(5);

    // Let's acquire borrow HTTP to send a message inside the `async move`.
    let http = &ctx.http;

    // We want to process each message and get the length.
    // There are a couple of ways to do this. Folding the stream with `fold`
    // is one way.
    // Using `then` to first reply and then create a new stream with all
    // messages is another way to do it, which can be nice if you want
    // to further process the messages.
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

    // We can also collect arbitrary events using the collect() function. For example, here we
    // collect updates to the messages that the user sent above and check for them updating all 5 of
    // them.
    let mut collector = serenity::collector::collect(&ctx.shard, move |event| match event {
        // Only collect MessageUpdate events for the 5 MessageIds we're interested in.
        Event::MessageUpdate(event) if collected.iter().any(|msg| event.id == msg.id) => {
            Some(event.id)
        },
        _ => None,
    })
    .take_until(Box::pin(tokio::time::sleep(Duration::from_secs(20))));

    let _ = msg.reply(ctx, "Edit each of those 5 messages in 20 seconds").await;
    let mut edited = HashSet::new();
    while let Some(edited_message_id) = collector.next().await {
        edited.insert(edited_message_id);
        if edited.len() >= 5 {
            break;
        }
    }

    if edited.len() >= 5 {
        score += 1;
        let _ = msg.reply(ctx, "Great! You edited 5 out of 5").await;
    } else {
        let _ = msg.reply(ctx, &format!("You only edited {} out of 5", edited.len())).await;
    }

    let _ = msg
        .reply(ctx, &format!("TIME'S UP! You completed {} out of 4 tasks correctly!", score))
        .await;

    Ok(())
}
