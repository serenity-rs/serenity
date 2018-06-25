extern crate futures;
extern crate serenity;
extern crate tokio;
extern crate env_logger;
extern crate parking_lot;

use serenity::{
    gateway::{
        ShardingStrategy,
        ShardManager,
        ShardManagerOptions,
        SimpleReconnectQueue,
    },
    model::event::{
        Event,
        GatewayEvent,
    },
};
use std::{
    borrow::Cow,
    env, 
    rc::Rc,
    sync::Arc,
};
use futures::{future, stream, Future, Stream};
use parking_lot::Mutex;

fn main() {
    env_logger::init().expect("Error initializing env_logger");

    tokio::run(try_main());
}

fn try_main() -> impl Future<Item = (), Error = ()> {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let opts = ShardManagerOptions {
        strategy: ShardingStrategy::multi(4),
        token: token,
        ws_uri: String::from("nothing"),
        queue: SimpleReconnectQueue::new(4),
    }; 

    let mut shard_manager = ShardManager::new(opts);
    shard_manager.start()
        .map_err(|e| println!("Error starting shard manager: {:?}", e))
        .and_then(|mut shard_manager| {
            shard_manager.messages()
                .zip(stream::repeat(Arc::new(Mutex::new(shard_manager))))
                .for_each(move |((shard, message), shard_manager)| {
                    let mut shard = shard.lock();
                    
                    let event = shard.parse(message)
                        .expect("Could not parse shard stream message");

                    shard.process(&event);
                    {
                        let mut shard_manager = shard_manager.lock();
                        shard_manager.process(&event);
                    }

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

                    future::ok(())
                })
        })

    // tokio::spawn(future);
}
