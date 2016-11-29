use super::utils::{into_map, into_string, remove, warn_field};
use super::UserId;
use ::constants::VoiceOpCode;
use ::internal::prelude::*;
use ::utils::decode_array;

#[derive(Clone, Copy, Debug)]
pub struct VoiceHeartbeat {
    pub heartbeat_interval: u64,
}

#[derive(Clone, Debug)]
pub struct VoiceHello {
    pub heartbeat_interval: u64,
    pub ip: String,
    pub modes: Vec<String>,
    pub port: u16,
    pub ssrc: u32,
}

#[derive(Clone, Debug)]
pub struct VoiceReady {
    pub mode: String,
    pub secret_key: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct VoiceSpeaking {
    pub speaking: bool,
    pub ssrc: u32,
    pub user_id: UserId,
}

#[derive(Clone, Debug)]
pub enum VoiceEvent {
    Heartbeat(VoiceHeartbeat),
    Hello(VoiceHello),
    Ready(VoiceReady),
    Speaking(VoiceSpeaking),
    KeepAlive,
    Unknown(VoiceOpCode, Value)
}

impl VoiceEvent {
    pub fn decode(value: Value) -> Result<VoiceEvent> {
        let mut value = try!(into_map(value));
        let op = req!(try!(remove(&mut value, "op")).as_u64());
        let mut map = try!(remove(&mut value, "d").and_then(into_map));

        let opcode = try!(VoiceOpCode::from_num(op)
            .ok_or(Error::Client(ClientError::InvalidOpCode)));

        match opcode {
            VoiceOpCode::Heartbeat => {
                missing!(map, VoiceEvent::Heartbeat(VoiceHeartbeat {
                    heartbeat_interval: req!(try!(remove(&mut map, "heartbeat_interval")).as_u64()),
                }))
            },
            VoiceOpCode::Hello => {
                missing!(map, VoiceEvent::Hello(VoiceHello {
                    heartbeat_interval: req!(try!(remove(&mut map, "heartbeat_interval"))
                        .as_u64()),
                    ip: try!(remove(&mut map, "ip").and_then(into_string)),
                    modes: try!(decode_array(try!(remove(&mut map, "modes")),
                                             into_string)),
                    port: req!(try!(remove(&mut map, "port"))
                        .as_u64()) as u16,
                    ssrc: req!(try!(remove(&mut map, "ssrc"))
                        .as_u64()) as u32,
                }))
            },
            VoiceOpCode::KeepAlive => Ok(VoiceEvent::KeepAlive),
            VoiceOpCode::SessionDescription => {
                missing!(map, VoiceEvent::Ready(VoiceReady {
                    mode: try!(remove(&mut map, "mode")
                        .and_then(into_string)),
                    secret_key: try!(decode_array(try!(remove(&mut map, "secret_key")),
                        |v| Ok(req!(v.as_u64()) as u8)
                    )),
                }))
            },
            VoiceOpCode::Speaking => {
                missing!(map, VoiceEvent::Speaking(VoiceSpeaking {
                    speaking: req!(try!(remove(&mut map, "speaking")).as_bool()),
                    ssrc: req!(try!(remove(&mut map, "ssrc")).as_u64()) as u32,
                    user_id: try!(remove(&mut map, "user_id").and_then(UserId::decode)),
                }))
            }
            other => Ok(VoiceEvent::Unknown(other, Value::Object(map))),
        }
    }
}
