#![feature(
async_await,
await_macro,
futures_api,
generators,
try_blocks,
try_trait,
)]

extern crate serenity;
extern crate futures;
extern crate env_logger;
extern crate tokio;
extern crate tungstenite;

use futures::{
    compat::{Future01CompatExt, Stream01CompatExt},
    prelude::*,
};
use serde_json::Error as JsonError;
use serenity::gateway::{Shard, Action};
use serenity::model::event::{Event, GatewayEvent};
use serenity::Error as SerenityError;
use std::env;
use std::option::NoneError;
use tungstenite::Error as TungsteniteError;

#[derive(Debug)]
enum Error {
    Json(JsonError),
    None,
    Serenity(SerenityError),
    Tungstenite(TungsteniteError),
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Self {
        Error::Json(err)
    }
}

impl From<NoneError> for Error {
    fn from(_: NoneError) -> Self {
        Error::None
    }
}

impl From<SerenityError> for Error {
    fn from(err: SerenityError) -> Self {
        Error::Serenity(err)
    }
}

impl From<TungsteniteError> for Error {
    fn from(err: TungsteniteError) -> Self {
        Error::Tungstenite(err)
    }
}

fn main() {
    tokio::run(main_future().map_err(|reason| {
        eprintln!("Error when running shard {:?}:", reason);
    }).boxed().compat());
}

async fn main_future() -> Result<(), Error> {
    env_logger::init();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("No token was provided for the bot.");
    // Create new shard.
    let mut shard = await!(Shard::new(token, [0, 1]).compat()).unwrap();
    let mut messages = shard.messages().expect("No shard messages found.").compat();
    println!("Shard is connected.");

    loop {
        // Loop over websocket messages.
        let result: Result<_, Error> = try {
            let message = await!(messages.next())??;
            println!("Receiving shard message {:?}", message);
            // Parse websocket event to Serenity Gateway Event.
            let event = shard.parse(&message)?;
            //Process websocket event.
            let process = shard.process(&event);
            // Handle websocket actions, such as identifying and auto-reconnecting.
            if let Ok(Some(action)) = process {
                match action {
                    Action::Identify => {
                        println!("Identify Requested from Shard 0.");
                        if let Err(why) = shard.identify() {
                            println!("There was an error when identifiying: {:?}", why);
                            break;
                        }
                    },
                    Action::Autoreconnect => {
                        println!("Shard 0 told us to autoreconnect.");
                        if let Err(reason) = await!(shard.autoreconnect().compat()) {
                            println!("Failed to autoreconnect shard. {:?}", reason);
                            break;
                        }
                        messages = shard.messages()?.compat();
                    }
                    Action::Reconnect => {
                        println!("Shard 0 told us to reconnect!");
                        break;
                    },
                    Action::Resume => {
                        println!("Shard 0 told us to resume!");
                        if let Err(reason) = await!(shard.resume().compat()) {
                            println!("Error resuming shard. {:?}", reason);
                            break;
                        }
                        messages = shard.messages()?.compat();
                    },

                }
            };

            // Now you may handle the Shard Events.
            match event {
                GatewayEvent::Dispatch(_, Event::MessageCreate(ev)) => {
                    if ev.message.content == "!pingo" {
                        println!("Pong!");
                    }
                }
                GatewayEvent::Dispatch(_, Event::Ready(_)) => {
                    println!("I am is connected to Discord.");
                }
                _ => {}
            }
        };

        if let Err(why) = result {
            println!("Error with loop occurred: {:?}", why);

            match why {
                Error::Tungstenite(TungsteniteError::ConnectionClosed(Some(close))) => {
                    println!(
                        "Close: code: {}; reason: {}",
                        close.code,
                        close.reason,
                    );
                }
                other => {
                    println!("Shard error: {:?}", other);

                    continue;
                }
            }
            println!("Autoreconnecting shard.");

            await!(shard.autoreconnect().compat())?;
            messages = shard.messages()?.compat();
        }
    }

    Ok(())
}
