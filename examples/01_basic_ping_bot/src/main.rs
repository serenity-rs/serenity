#![feature(
    async_await,
    await_macro,
    futures_api,
    generators,
    try_blocks,
    try_trait,
)]

extern crate env_logger;
extern crate futures;
extern crate serde_json;
extern crate serenity;
extern crate tokio;
extern crate tungstenite;

use futures::{
    compat::{Future01CompatExt, Stream01CompatExt, TokioDefaultSpawner},
    prelude::*,
};
use serde_json::Error as JsonError;
use serenity::gateway::Shard;
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
    tokio::run(try_main().map_err(|why| {
        println!("Error running shard: {:?}", why);
    }).boxed().compat(TokioDefaultSpawner));
}

async fn try_main() -> Result<(), Error> {
    env_logger::init();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a new shard, specifying the token, the ID of the shard (0 of 1),
    // and a handle to the event loop
    let mut shard = await!(Shard::new(token, [0, 1]).compat())?;
    println!("Connected shard");
    let mut messages = shard.messages().compat();

    loop {
        // Loop over websocket messages.
        let result: Result<_, Error> = try {
            println!("Getting message");
            let message = await!(messages.next())??;

            // Parse the websocket message into a serenity GatewayEvent.
            let event = shard.parse(&message)?;

            // Have the shard process the WebSocket event, in case it needs
            // to mutate its state, send a packet, etc.
            //
            // This can give back a future in the event something needs to
            // be done, such as waiting for a reconnection.
            if let Some(future) = shard.process(&event) {
                await!(future.compat())?;
            }

            // Now you can do whatever you want with the event.
            match event {
                GatewayEvent::Dispatch(_, Event::MessageCreate(ev)) => {
                    if ev.message.content == "!ping" {
                        println!("Pong!");
                    }
                },
                GatewayEvent::Dispatch(_, Event::Ready(_)) => {
                    println!("Connected to Discord!");
                },
                _ => {
                    // Ignore all other messages.
                },
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
                },
                other => {
                    println!("Shard error: {:?}", other);

                    continue;
                },
            }

            println!("Autoreconnecting");

            await!(shard.autoreconnect().compat())?;
            messages = shard.messages().compat();
        }
    }
}
