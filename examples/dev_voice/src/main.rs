// This is *not* intended to be an ergonomic or final way to
// use voice in v0.6.x -- its main purpose is for testing and
// to show a manual use case for further refining the API.
//
// Read: this is awfully hacky and is barely "code".
// I had to get around the manager being broken somehow, given
// that it likes to be well-integrated with the shard runner.

extern crate env_logger;
extern crate futures;
extern crate parking_lot;
#[macro_use]
extern crate serde_json;
extern crate serenity;
extern crate tokio_core;
extern crate tungstenite;

use futures::{
    future::{self, Executor,},
    stream,
    Future,
    Stream,
};
use parking_lot::Mutex;
use serenity::{
    constants::OpCode,
    gateway::{
        Shard,
        ShardingStrategy,
        ShardManager,
        ShardManagerOptions,
        SimpleReconnectQueue,
        WrappedShard,
    },
    model::{
        event::{
            Event,
            GatewayEvent,
            VoiceServerUpdateEvent,
            VoiceStateUpdateEvent,
        },
        id::{
            ChannelId,
            GuildId,
            UserId,
        },
        voice::VoiceState,
    },
    voice::{self, Handler},
};
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    env,
    fs::File,
    io::Write,
    iter::Iterator,
    rc::Rc,
    sync::Arc,
};
use tokio_core::reactor::{Core, Handle};
use tungstenite::{Error as TungsteniteError, Message as TungsteniteMessage};

fn main() {
    env_logger::init().expect("Error initializing env_logger");

    let mut core = Core::new().expect("Error creating event loop");
    let future = try_main2(core.handle());

    core.run(future).expect("Error running event loop");
}

fn try_main2(handle: Handle) -> impl Future<Item = (), Error = ()> {
    let remote = handle.remote().clone();

    handle.execute(future::ok("spawn-test")
        .map(|val| {
            println!("Zeroth future: {}", val);
        }));

    future::ok("test")
        .map(move |val| {
            println!("First future: {}", val);
            remote.spawn(move |handle| {
                println!("Building second future...");
                handle.spawn(future::ok("test2")
                    .map(|val| {
                        println!("Second future: {}", val);
                    }));
                Ok(())
            });
        })
}

fn try_main(handle: Handle) -> impl Future<Item = (), Error = ()> {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let opts = ShardManagerOptions {
        strategy: ShardingStrategy::multi(4),
        token: Rc::new(token),
        ws_uri: Rc::new(String::from("nothing")),
        queue: SimpleReconnectQueue::new(4),
    }; 

    let mut shard_manager = ShardManager::new(opts, handle.clone());
    let future = shard_manager.start()
        .map_err(|e| println!("Error starting shard manager: {:?}", e));

    handle.spawn(future);

    let inner_state = stream::repeat(
        (
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(UserId(0)))
        ));

    let future = shard_manager.messages().zip(inner_state).for_each(move |((shard, message), (handlers, user_id))| {
        let mut shard = shard.borrow_mut();
        let event = shard.parse(message);
        
        let event = event.expect("Could not parse shard stream message");

        shard.process(&event);
        shard_manager.process(&event);

        let mut out: Box<Future<Item=(),Error=()>> = Box::new(future::ok(()));

        // {println!("{:?}", &event);}

        match event {
            GatewayEvent::Dispatch(_, Event::MessageCreate(ev)) => {
                let parts = ev.message.content.split(' ');
                let mut do_voice = false;
                let mut id = 0u64;

                for (i, part) in parts.enumerate() {
                    match i {
                        0 => {
                            if part != "!join" {
                                break;
                            } else {
                                do_voice = true;
                            }
                        },
                        _ => {
                            if let Ok(new_id) = part.parse::<u64>() {
                                id = new_id;
                            }
                        },
                    }
                }

                if do_voice && id != 0 {
                    if let Some(guild_id) = ev.message.guild_id {
                        let handler = {
                            let user_id = user_id.lock();
                            send_channel_join(id, guild_id, *user_id, shard)
                        };
                        let mut map = handlers.lock();
                        
                        map.insert(guild_id, handler);
                    }
                    
                }
            },
            GatewayEvent::Dispatch(_, Event::VoiceStateUpdate(ev)) => {
                println!("{:#?}", ev);
                let mut map = handlers.lock();

                if let Some(guild_id) = ev.guild_id {
                    if let Some(mut handler) = map.get_mut(&guild_id) {
                        handler.update_state(&ev.voice_state);

                        try_join_and_play_audio(&mut handler);
                    }
                }
                
            },
            GatewayEvent::Dispatch(_, Event::VoiceServerUpdate(ev)) => {
                println!("{:#?}", ev);
                let mut map = handlers.lock();

                if let Some(guild_id) = ev.guild_id {
                    if let Some(mut handler) = map.get_mut(&guild_id) {
                        handler.update_server(&ev.endpoint, &ev.token);

                        try_join_and_play_audio(&mut handler);
                    }
                }
            },
            GatewayEvent::Dispatch(_, Event::Ready(ev)) => {
                println!("Connected to Discord!");
                let mut stored_id = user_id.lock();

                *stored_id = ev.ready.user.id;
            },
            _ => {
                // Ignore all other messages.
            },
        }

        out
    });

    future
}

fn send_channel_join(voice_id: u64, guild_id: GuildId, user_id: UserId, mut shard: RefMut<Shard>) -> Handler {
    let voice_update = json!({
        "op": OpCode::VoiceStateUpdate.num(),
        "d": {
            "channel_id": voice_id,
            "guild_id": guild_id,
            "self_deaf": false,
            "self_mute": false,
        }
    });

    shard.send(TungsteniteMessage::Text(voice_update.to_string()));

    Handler::standalone(guild_id, user_id)
}

fn try_join_and_play_audio(handler: &mut Handler) {
    if handler.connect() {
        // Eine Kleine Nachtmusik
        handler.play(
            voice::ytdl("https://www.youtube.com/watch?v=o1FSN8_pp_o").expect("Link to video taken down.")
        );
    }
}