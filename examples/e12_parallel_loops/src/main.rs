use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use chrono::offset::Utc;
use serenity::async_trait;
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::gateway::ActivityData;
use serenity::model::id::GenericChannelId;
use serenity::prelude::*;

struct Handler {
    is_loop_running: AtomicBool,
}

use serenity::gateway::client::FullEvent;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message {
                new_message, ..
            } => {
                if new_message.content == "!ping"
                    && let Err(why) = new_message.channel_id.say(&ctx.http, "Pong!").await
                {
                    println!("Error sending message: {why:?}");
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

                // We need to check that the loop is not already running when this event triggers,
                // as this event triggers every time the bot enters or leaves a
                // guild, along every time the ready shard event triggers.
                //
                // An AtomicBool is used because it doesn't require a mutable reference to be
                // changed, as we don't have one due to self being an immutable
                // reference.
                if !self.is_loop_running.load(Ordering::Relaxed) {
                    // We have to clone the ctx, as it gets moved into the new thread.
                    let ctx1 = ctx.clone();
                    // tokio::spawn creates a new green thread that can run in parallel with the
                    // rest of the application.
                    tokio::spawn(async move {
                        loop {
                            log_system_load(&ctx1).await;
                            tokio::time::sleep(Duration::from_secs(120)).await;
                        }
                    });

                    // And of course, we can run more than one thread at different timings.
                    let ctx2 = ctx.clone();
                    tokio::spawn(async move {
                        loop {
                            set_activity_to_current_time(&ctx2);
                            tokio::time::sleep(Duration::from_secs(60)).await;
                        }
                    });

                    // Now that the loop is running, we set the bool to true
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

    // We can use ChannelId directly to send a message to a specific channel; in this case, the
    // message would be sent to the #testing channel on the discord server.
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
    let message = GenericChannelId::new(381926291785383946).send_message(&ctx.http, builder).await;
    if let Err(why) = message {
        eprintln!("Error sending message: {why:?}");
    };
}

fn set_activity_to_current_time(ctx: &Context) {
    let current_time = Utc::now();
    let formatted_time = current_time.to_rfc2822();

    ctx.set_activity(Some(ActivityData::playing(formatted_time)));
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
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}
