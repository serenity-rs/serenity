//! This example will showcase the beauty of collectors.
//! The allow to await messages or reactions from a user in the middle
//! of a control flow, one being a command.
use std::{
    collections::HashSet, env,
    time::Duration,
};
use serenity::{
    async_trait,
    collector::MessageCollectorBuilder,
    framework::standard::{
        Args, CommandResult, CommandGroup,
        HelpOptions, help_commands, StandardFramework,
        macros::{command, group, help},
    },
    prelude::*,
    http::Http,
    model::prelude::*,
};

#[group("collector")]
#[commands(challenge)]
struct Collector;

#[help]
async fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, &help_options, groups, owners).await
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
    let token = env::var("DISCORD_TOKEN").expect(
        "Expected a token in the environment",
    );

    let http = Http::new_with_token(&token);

    // We will fetch your bot's id.
    let bot_id = match http.get_current_application_info().await {
        Ok(info) => {
            info.id
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("~")
            .delimiters(vec![", ", ","]))
        .help(&MY_HELP)
        .group(&COLLECTOR_GROUP);

    let mut client = Client::new_with_framework(&token, Handler, framework).await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn challenge(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let mut score = 0u32;
    let _ =  msg.reply(&ctx, "How was that crusty crab called again? 10 seconds time!").await;

    // There are methods implemented for some models to conveniently collect replies.
    if let Some(answer) = &msg.author.await_reply(&ctx).timeout(Duration::from_secs(10)).await {

        if answer.content.to_lowercase() == "ferris" {
            let _ = answer.reply(&ctx, "That's correct!").await;
            score += 1;
        } else {
            let _ = answer.reply(&ctx, "Wrong, it's Ferris!").await;
        }
    } else {
        let _ =  msg.reply(&ctx, "No answer within 10 seconds.").await;
    };

    let react_msg = msg.reply(&ctx, "React with the reaction representing 1. 10 seconds time!").await.unwrap();

    // The message model has a way to collect reactions on it.
    // Other methods are `await_n_reactions` and `await_all_reactions`.
    // Same goes for messages!
    if let Some(reaction) = &react_msg.await_reaction(&ctx).timeout(Duration::from_secs(10)).author_id(msg.author.id).await {

        // By default, the collector will collect only added reactions.
        // We could also pattern-match the reaction in case we want
        // to handle added or removed reactions.
        // In this case we will just get the inner reaction.
        let emoji = &reaction.as_inner_ref().emoji;

        let _ = match emoji.as_data().as_str() {
            "1️⃣" => { score += 1; msg.reply(&ctx, "That's correct!").await },
            _ => msg.reply(&ctx, "Wrong!").await,
        };
    } else {
        let _ = msg.reply(&ctx, "No reaction within 10 seconds.").await;
    };

    let _ = msg.reply(&ctx, "Write five messages!").await;

    // We can create a collector from scratch too using this builder future.
    let mut collector = MessageCollectorBuilder::new(&ctx)
        // Only collect messages by this user.
        .author_id(msg.author.id)
        // At maximum collect 5 messages.
        .collect_limit(5u32)
        // Very important, collectors don't timeout by default.
        // You should always set a timeout.
        .timeout(Duration::from_secs(5))
        // Build the collector.
        .await;

    let mut counter = 0;

    // A collector can be used step by step.
    // However, while not receiving, events will still be evaluated.
    // If you want to expect only one message and then stop accepting
    // new events, you will need to create a new collector.
    loop {
        // Receive a single message.
        if let Some(message) = collector.receive_one().await {
            counter += 1;
            let _ = message.reply(&ctx, &format!("I repeat: {}", message.content)).await;
        // When five messages have been received or one reply took longer than five seconds,
        // we won't receive a message.
        } else {
            break;
        }
    }

    if counter == 5 {
        score += 1;
    }

    collector.stop();

    let _ = msg.reply(&ctx, &format!("You completed {} out of 3 tasks correctly!", score)).await;

    Ok(())
}
