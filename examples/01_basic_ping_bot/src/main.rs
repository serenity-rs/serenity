#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate futures_await as futures;
extern crate serenity;
extern crate tokio_core;

use futures::prelude::*;
use serenity::gateway::Shard;
use serenity::model::event::{Event, GatewayEvent};
use serenity::model::gateway::Game;
use serenity::model::user::OnlineStatus;
use std::error::Error;
use std::env;
use tokio_core::reactor::{Core, Handle};

fn main() {
    let mut core = Core::new().expect("Error creating event loop");
    let future = try_main(core.handle());

    core.run(future).expect("Error running event loop");
}

#[async]
fn try_main(handle: Handle) -> Result<(), Box<Error + 'static>> {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    // Create a new shard, specifying the token, the ID of the shard (0 of 1),
    // and a handle to the event loop
    let mut shard = await!(Shard::new(token, [0, 1], handle))?;

    // Loop over websocket messages.
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

    Ok(())
}
