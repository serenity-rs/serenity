extern crate futures;
extern crate serenity;
extern crate tokio_core;
extern crate env_logger;

use futures::Future;
use serenity::gateway::shard;
use std::env;
use tokio_core::reactor::{Core, Handle};

fn main() {
    env_logger::init().expect("Error initializing env_logger");

    let mut core = Core::new().expect("Error creating event loop");
    let future = try_main(core.handle());

    core.run(future).expect("Error running event loop");
}

fn try_main(handle: Handle) -> impl Future<Item = (), Error = ()> {
	let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    shard::new(token, [0,0], handle)
    	.and_then(|shard| {
    		Ok(())
    	})
    	.map_err(|_| ())
}