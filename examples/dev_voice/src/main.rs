// This is *not* intended to be an ergonomic or final way to
// use voice in v0.6.x -- its main purpose is for testing and
// to show a manual use case for further refining the API.
//
// Read: this is awfully hacky and is barely "code".
// I had to get around the manager being broken somehow, given
// that it likes to be well-integrated with the shard runner.

extern crate futures;
#[macro_use]
extern crate serde_json;
extern crate serenity;
extern crate tokio_core;
extern crate env_logger;
extern crate tungstenite;

use futures::{
    future,
    Future,
    Stream,
};
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
        },
        voice::VoiceState,
    },
};
use std::{
    cell::{
        RefCell,
        RefMut,
    },
    env,
    fs::File,
    io::Write,
    iter::Iterator,
    rc::Rc,
};
use tokio_core::reactor::{Core, Handle};
use tungstenite::{Error as TungsteniteError, Message as TungsteniteMessage};

fn main() {
    env_logger::init().expect("Error initializing env_logger");

    let mut core = Core::new().expect("Error creating event loop");
    let future = try_main(core.handle());

    core.run(future).expect("Error running event loop");
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

    let future = shard_manager.messages().for_each(move |(shard, message)| {
        let mut shard = shard.borrow_mut();
        
        let bak_msg = message.clone();
        let event = shard.parse(message);
        
        let event = event.expect("Could not parse shard stream message");

        shard.process(&event);
        shard_manager.process(&event);

        let mut out: Box<Future<Item=(),Error=()>> = Box::new(future::ok(()));

        {println!("{:?}", &event);}

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

                if do_voice && id!=0 {
                    out = Box::new(try_audio_connect(id, ev.message.guild_id, shard));
                }
            },
            GatewayEvent::Dispatch(_, Event::Ready(_)) => {
                println!("Connected to Discord!");
            },
            _ => {
                // Ignore all other messages.
            },
        }

        out
    });

    future
}

fn try_audio_connect(voice_id: u64, guild_id: Option<GuildId>, shard: RefMut<Shard>) -> impl Future<Item = (), Error = ()>{
    // TODO
    println!("{}, {:?}", voice_id, guild_id);

    let guild_id = match guild_id {
        Some(GuildId(guild_id)) => guild_id,
        _ => 0,
    };

    let map = json!({
        "op": OpCode::VoiceStateUpdate.num(),
        "d": {
            "channel_id": voice_id,
            "guild_id": guild_id,
            "self_deaf": false,
            "self_mute": false,
        }
    });

    println!("{}", map);

    // TODo: send me.

    future::ok(())
}
