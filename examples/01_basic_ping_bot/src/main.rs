#![feature(catch_expr, proc_macro, generators)]

extern crate env_logger;
extern crate futures_await as futures;
extern crate serenity;
extern crate tokio;

use futures::prelude::{async, await};
use serenity::gateway::Shard;
use serenity::model::event::{Event, GatewayEvent};
use std::error::Error;
use std::env;
use std::rc::Rc;
use tokio::executor::current_thread;

fn main() {
    current_thread::block_on_all(try_main()).expect("Error running event loop");
}

#[async]
fn try_main() -> Result<(), Box<Error + 'static>> {
    env_logger::init();

    // Configure the client with your Discord bot token in the environment.
    let token = Rc::new(env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment"));

    loop {
        // Create a new shard, specifying the token, the ID of the shard (0 of 1),
        // and a handle to the event loop
        let mut shard = await!(Shard::new(Rc::clone(&token), [0, 1]))?;

        // Loop over websocket messages.
        let result: Result<_, Box<Error>> = do catch {
            #[async]
            for message in shard.messages() {
                // Parse the websocket message into a serenity GatewayEvent.
                let event = shard.parse(message)?;

                // Have the shard process the WebSocket event, in case it needs to
                // mutate its state, send a packet, etc.
                shard.process(&event);

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
            }

            ()
        };

        if let Err(why) = result {
            println!("Error with loop occurred: {:?}", why);
            println!("Creating new shard");

            continue;
        }
    }
}
