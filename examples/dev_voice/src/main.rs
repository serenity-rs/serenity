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
extern crate tokio;
extern crate tungstenite;

use futures::{
    future,
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
    },
    model::{
        event::{
            Event,
            GatewayEvent,
        },
        id::{
            GuildId,
            UserId,
        },
    },
    voice::{self, Handler},
};
use std::{
    collections::HashMap,
    env,
    iter::Iterator,
    sync::Arc,
};
use tungstenite::Message as TungsteniteMessage;

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

    let inner_state = stream::repeat(
        (
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(UserId(0))),
        ));

    let shard_manager = ShardManager::new(opts);

    shard_manager.start()
        .map_err(|e| println!("Error starting shard manager: {:?}", e))
        .and_then(|mut shard_manager| {
            shard_manager.messages()
                .zip(stream::repeat(Arc::new(Mutex::new(shard_manager))))
                .zip(inner_state)
                .for_each(move |(((shard, message), shard_manager),
                        (handlers, user_id))| {
                    let mut shard = shard.lock();
                    
                    let event = shard.parse(message)
                        .expect("Could not parse shard stream message");

                    shard.process(&event);
                    {
                        let mut shard_manager = shard_manager.lock();
                        shard_manager.process(&event);
                    }

                    let out: Box<Future<Item=(),Error=()> + Send> = Box::new(future::ok(()));

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
                                        send_channel_join(id, guild_id, *user_id, &mut shard)
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
                })
        })
}

fn send_channel_join(voice_id: u64, guild_id: GuildId, user_id: UserId, shard: &mut Shard) -> Handler {
    let voice_update = json!({
        "op": OpCode::VoiceStateUpdate.num(),
        "d": {
            "channel_id": voice_id,
            "guild_id": guild_id,
            "self_deaf": false,
            "self_mute": false,
        }
    });

    shard.send(TungsteniteMessage::Text(voice_update.to_string()))
        .expect("Sending voice-join attempt failed...");

    Handler::standalone(guild_id, user_id)
}

fn try_join_and_play_audio(handler: &mut Handler) {
    if handler.connect() {
        // Pokke Village
        handler.play(
            voice::ytdl("https://www.youtube.com/watch?v=__QdAxqBi5Y").expect("Link to video taken down.")
        );
    }
}
