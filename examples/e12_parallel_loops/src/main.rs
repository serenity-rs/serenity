//! This example demonstrates how to run tasks in a loop with access to Context.
#![expect(clippy::collapsible_if)]
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use jiff::Timestamp;
use jiff::fmt::rfc2822;
use jiff::tz::TimeZone;
use serenity::async_trait;
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::gateway::ActivityData;
use serenity::model::event::FullEvent;
use serenity::model::id::GenericChannelId;
use serenity::prelude::*;

// Placing data in the Handler struct allows us to access that data from within the event handler.
// In the previous global data example, we saw something similar. However, we only need access from
// the event handler this time, so creating a user data struct is unnecessary.
struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message {
                new_message, ..
            } => {
                if new_message.content == "!ping" {
                    if let Err(why) = new_message.channel_id.say(&ctx.http, "Pong!").await {
                        println!("Error sending message: {why:?}");
                    }
                }
            },
            FullEvent::Ready {
                data_about_bot, ..
            } => {
                println!("{} is connected!", data_about_bot.user.name);
            },
            FullEvent::CacheReady {
                ..
            } => {
                println!("Cache built successfully!");

                // We need to check that the loop is not already running, as this event may be
                // dispatched multiple times.
                //
                // An `AtomicBool` is used because it doesn't require a mutable reference to be
                // changed, and we don't have one since `self` here is an immutable reference.
                if !self.is_loop_running.load(Ordering::Relaxed) {
                    // We have to clone the Context since it gets moved into a new thread.
                    let ctx1 = ctx.clone();
                    // `tokio::spawn` creates a new green thread, or task, that can run in parallel
                    // with the rest of the application.
                    //
                    // This task will log the system load every two minutes.
                    tokio::spawn(async move {
                        loop {
                            log_system_load(&ctx1).await;
                            tokio::time::sleep(Duration::from_secs(120)).await;
                        }
                    });

                    // And of course, we can spawn multiple tasks with different timings.
                    //
                    // This task will set the bot's activity text to the current time every
                    // sixty seconds.
                    let ctx2 = ctx.clone();
                    tokio::spawn(async move {
                        loop {
                            set_activity_to_current_time(&ctx2);
                            tokio::time::sleep(Duration::from_secs(60)).await;
                        }
                    });

                    // Now that the loops are running, we set the bool to `true`.
                    self.is_loop_running.swap(true, Ordering::Relaxed);
                }
            },
            _ => {},
        }
    }
}

async fn log_system_load(ctx: &Context) {
    let cpu_load = sys_info::loadavg().unwrap();
    let mem_use = sys_info::mem_info().unwrap();

    let embed = CreateEmbed::new()
        .title("System Resource Load")
        .field("CPU Load Average", format!("{:.2}%", cpu_load.one * 10.0), false)
        .field(
            "Memory Usage",
            format!(
                "{:.2} MB Free out of {:.2} MB",
                mem_use.free as f32 / 1000.0,
                mem_use.total as f32 / 1000.0
            ),
            false,
        );
    let builder = CreateMessage::new().embed(embed);
    // We can use `GenericChannelId` directly to send a message to a specific channel; in this
    // case, the message would be sent to the #testing channel in the Serenity Discord server.
    if let Err(why) =
        GenericChannelId::new(381926291785383946).send_message(&ctx.http, builder).await
    {
        eprintln!("Error sending message: {why:?}");
    };
}

fn set_activity_to_current_time(ctx: &Context) {
    let current_time = Timestamp::now().to_zoned_datetime(TimeZone::UTC);
    let formatted_time = rfc2822::to_string(&current_time).unwrap();

    ctx.set_activity(Some(ActivityData::custom(formatted_time)));
}

#[tokio::main]
async fn main() {
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a valid token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        // We initialize the AtomicBool to `false` to ensure the loop begins running the first time
        // the `CacheReady` event is dispatched.
        .event_handler(Arc::new(Handler {
            is_loop_running: AtomicBool::new(false),
        }))
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
