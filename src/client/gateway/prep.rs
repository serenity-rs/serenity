use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use std::net::Shutdown;
use std::sync::mpsc::{
    TryRecvError,
    Receiver as MpscReceiver,
    Sender as MpscSender
};
use std::time::Duration as StdDuration;
use std::{env, thread};
use super::super::ClientError;
use super::{GatewayError, GatewayStatus};
use time::{self, Duration};
use websocket::client::request::Url as RequestUrl;
use websocket::client::{Receiver, Sender};
use websocket::stream::WebSocketStream;
use ::constants::{self, OpCode};
use ::error::{Error, Result};
use ::internal::ws_impl::{ReceiverExt, SenderExt};
use ::model::{Event, GatewayEvent, ReadyEvent};

#[inline]
pub fn parse_ready(event: GatewayEvent,
               tx: &MpscSender<GatewayStatus>,
               receiver: &mut Receiver<WebSocketStream>,
               identification: Value)
               -> Result<(ReadyEvent, u64)> {
    match event {
        GatewayEvent::Dispatch(seq, Event::Ready(event)) => {
            Ok((event, seq))
        },
        GatewayEvent::InvalidateSession => {
            debug!("Session invalidation");

            let _ = tx.send(GatewayStatus::SendMessage(identification));

            match try!(receiver.recv_json(GatewayEvent::decode)) {
                GatewayEvent::Dispatch(seq, Event::Ready(event)) => {
                    Ok((event, seq))
                },
                other => {
                    debug!("Unexpected event: {:?}", other);

                    Err(Error::Gateway(GatewayError::InvalidHandshake))
                },
            }
        },
        other => {
            debug!("Unexpected event: {:?}", other);

            Err(Error::Gateway(GatewayError::InvalidHandshake))
        },
    }
}

pub fn identify(token: &str, shard_info: Option<[u8; 2]>) -> Value {
    ObjectBuilder::new()
        .insert("op", OpCode::Identify.num())
        .insert_object("d", |mut object| {
            object = identify_compression(object)
                .insert("large_threshold", 250) // max value
                .insert_object("properties", |object| object
                    .insert("$browser", "Feature-full and ergonomic discord rust library")
                    .insert("$device", "serenity")
                    .insert("$os", env::consts::OS)
                    .insert("$referrer", "")
                    .insert("$referring_domain", "")
                )
                .insert("token", token)
                .insert("v", constants::GATEWAY_VERSION);

            if let Some(shard_info) = shard_info {
                object = object
                    .insert_array("shard", |a| a
                        .push(shard_info[0])
                        .push(shard_info[1]));
            }

            object
        })
        .build()
}

#[cfg(not(feature = "debug"))]
pub fn identify_compression(object: ObjectBuilder) -> ObjectBuilder {
    object.insert("compression", true)
}

#[cfg(feature = "debug")]
pub fn identify_compression(object: ObjectBuilder) -> ObjectBuilder {
    object.insert("compression", false)
}

pub fn build_gateway_url(base: &str) -> Result<RequestUrl> {
    RequestUrl::parse(&format!("{}?v={}", base, constants::GATEWAY_VERSION))
        .map_err(|_| Error::Client(ClientError::Gateway))
}

pub fn keepalive(interval: u64,
             mut sender: Sender<WebSocketStream>,
             channel: MpscReceiver<GatewayStatus>) {
    let mut base_interval = Duration::milliseconds(interval as i64);
    let mut next_tick = time::get_time() + base_interval;

    let mut last_sequence = 0;

    'outer: loop {
        thread::sleep(StdDuration::from_millis(100));

        loop {
            match channel.try_recv() {
                Ok(GatewayStatus::ChangeInterval(interval)) => {
                    base_interval = Duration::milliseconds(interval as i64);
                },
                Ok(GatewayStatus::ChangeSender(new_sender)) => {
                    sender = new_sender;
                },
                Ok(GatewayStatus::SendMessage(val)) => {
                    if let Err(why) = sender.send_json(&val) {
                        warn!("Err sending message: {:?}", why);
                    }
                },
                Ok(GatewayStatus::Sequence(seq)) => {
                    last_sequence = seq;
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break 'outer,
            }
        }

        if time::get_time() >= next_tick {
            next_tick = next_tick + base_interval;

            let map = ObjectBuilder::new()
                .insert("d", last_sequence)
                .insert("op", OpCode::Heartbeat.num())
                .build();

            if let Err(why) = sender.send_json(&map) {
                warn!("Err sending keepalive: {:?}", why);
            }
        }
    }

    let _ = sender.get_mut().shutdown(Shutdown::Both);
}
