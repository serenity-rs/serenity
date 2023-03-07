use std::net::Ipv4Addr;

use serde_test::{Configure, Token};

use super::Event;
use crate::id::*;
use crate::opcode::OpCode;
use crate::payload::*;
use crate::protocol_data::ProtocolData;
use crate::speaking_state::SpeakingState;

#[test]
fn deserialize_identify_json() {
    let json_data = r#"{
      "d": {
        "server_id": "41771983423143937",
        "user_id": "104694319306248192",
        "session_id": "my_session_id",
        "token": "my_token"
      },
      "op": 0
    }"#;

    let event = serde_json::from_str(json_data);

    let ident = Identify {
        session_id: "my_session_id".into(),
        token: "my_token".into(),
        server_id: GuildId(41771983423143937),
        user_id: UserId(104694319306248192),
    };

    assert!(matches!(event, Ok(Event::Identify(i)) if i == ident));
}

#[test]
fn deserialize_select_protocol_json() {
    let json_data = r#"{
        "op": 1,
        "d": {
            "protocol": "udp",
            "data": {
                "address": "127.0.0.1",
                "port": 1337,
                "mode": "xsalsa20_poly1305_lite"
            }
        }
    }"#;

    let event = serde_json::from_str(json_data);

    let proto = SelectProtocol {
        protocol: "udp".into(),
        data: ProtocolData {
            address: Ipv4Addr::new(127, 0, 0, 1).into(),
            port: 1337,
            mode: "xsalsa20_poly1305_lite".into(),
        },
    };

    assert!(matches!(event, Ok(Event::SelectProtocol(i)) if i == proto));
}

#[test]
fn deserialize_ready_json() {
    let json_data = r#"{
        "op": 2,
        "d": {
            "ssrc": 1,
            "ip": "127.0.0.1",
            "port": 1234,
            "modes": ["xsalsa20_poly1305", "xsalsa20_poly1305_suffix", "xsalsa20_poly1305_lite"],
            "heartbeat_interval": 1
        }
    }"#;

    // NOTE: we *need* to discard the interval here, as using it is an API footgun.

    let event = serde_json::from_str(json_data);

    let ready = Ready {
        ssrc: 1,
        ip: Ipv4Addr::new(127, 0, 0, 1).into(),
        port: 1234,
        modes: vec![
            "xsalsa20_poly1305".into(),
            "xsalsa20_poly1305_suffix".into(),
            "xsalsa20_poly1305_lite".into(),
        ],
    };

    assert!(matches!(event, Ok(Event::Ready(i)) if i == ready));
}

#[test]
fn deserialize_heartbeat_json() {
    let json_data = r#"{
      "op": 3,
      "d": 1501184119561
    }"#;

    let event = serde_json::from_str(json_data);

    let hb = Heartbeat {
        nonce: 1501184119561,
    };

    assert!(matches!(event, Ok(Event::Heartbeat(i)) if i == hb));
}

#[test]
fn deserialize_session_description_json() {
    let json_data = r#"{
        "op": 4,
        "d": {
            "mode": "xsalsa20_poly1305_lite",
            "secret_key": [251, 100, 11]
        }
    }"#;
    let event = serde_json::from_str(json_data);

    let sd = SessionDescription {
        mode: "xsalsa20_poly1305_lite".into(),
        secret_key: vec![251, 100, 11],
    };

    assert!(matches!(event, Ok(Event::SessionDescription(i)) if i == sd));
}

#[test]
fn deserialize_speaking_json() {
    let json_data = r#"{
        "op": 5,
        "d": {
            "speaking": 5,
            "delay": 0,
            "ssrc": 1
        }
    }"#;
    let event = serde_json::from_str(json_data);

    let speak = Speaking {
        speaking: SpeakingState::PRIORITY | SpeakingState::MICROPHONE,
        ssrc: 1,
        delay: Some(0),
        user_id: None,
    };

    assert!(matches!(event, Ok(Event::Speaking(i)) if i == speak));
}

#[test]
fn deserialize_heartbeat_ack_json() {
    let json_data = r#"{
      "op": 6,
      "d": 1501184119561
    }"#;

    let event = serde_json::from_str(json_data);

    let hb = HeartbeatAck {
        nonce: 1501184119561,
    };

    assert!(matches!(event, Ok(Event::HeartbeatAck(i)) if i == hb));
}

#[test]
fn deserialize_resume_json() {
    let json_data = r#"{
      "op": 7,
      "d": {
        "server_id": "41771983423143937",
        "session_id": "my_session_id",
        "token": "my_token"
      }
    }"#;

    let event = serde_json::from_str(json_data);

    let resume = Resume {
        server_id: GuildId(41771983423143937),
        session_id: "my_session_id".into(),
        token: "my_token".into(),
    };

    assert!(matches!(event, Ok(Event::Resume(i)) if i == resume));
}

#[test]
fn deserialize_hello_json() {
    let json_data = r#"{
      "op": 8,
      "d": {
        "heartbeat_interval": 41250
      }
    }"#;

    let event = serde_json::from_str(json_data);

    let hello = Hello {
        heartbeat_interval: 41250.0,
    };

    assert!(match event {
        Ok(Event::Hello(i)) =>
            (i.heartbeat_interval - hello.heartbeat_interval).abs() < f64::EPSILON,
        _ => false,
    });
}

#[test]
fn deserialize_resumed_json() {
    let json_data = r#"{
      "op": 9,
      "d": null
    }"#;

    let event = serde_json::from_str(json_data);

    assert!(matches!(event, Ok(Event::Resumed)));
}

#[test]
fn deserialize_client_connect_json() {
    let json_data = r#"{
      "op": 12,
      "d": {
        "audio_ssrc": 5678,
        "user_id": "1234",
        "video_ssrc": 9012
      }
    }"#;

    let event = serde_json::from_str(json_data);

    let conn = ClientConnect {
        audio_ssrc: 5678,
        user_id: UserId(1234),
        video_ssrc: 9012,
    };

    assert!(matches!(event, Ok(Event::ClientConnect(i)) if i == conn));
}

#[test]
fn deserialize_client_disconnect_json() {
    let json_data = r#"{
      "op": 13,
      "d": {
        "user_id": "1234"
      }
    }"#;

    let event = serde_json::from_str(json_data);

    let conn = ClientDisconnect {
        user_id: UserId(1234),
    };

    assert!(matches!(event, Ok(Event::ClientDisconnect(i)) if i == conn));
}

#[test]
fn serialize_identify() {
    let value: Event = Identify {
        server_id: GuildId(1),
        session_id: "56f88a86dce65c65b9".into(),
        token: "56f88a86dce65c65b8".into(),
        user_id: UserId(2),
    }
    .into();

    serde_test::assert_ser_tokens(&value, &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::Identify as u8),
        Token::Str("d"),
        Token::Struct {
            name: "Identify",
            len: 4,
        },
        Token::Str("server_id"),
        Token::NewtypeStruct {
            name: "GuildId",
        },
        Token::Str("1"),
        Token::Str("session_id"),
        Token::Str("56f88a86dce65c65b9"),
        Token::Str("token"),
        Token::Str("56f88a86dce65c65b8"),
        Token::Str("user_id"),
        Token::NewtypeStruct {
            name: "UserId",
        },
        Token::Str("2"),
        Token::StructEnd,
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_select_protocol() {
    let value: Event = SelectProtocol {
        protocol: "udp".into(),
        data: ProtocolData {
            address: Ipv4Addr::new(192, 168, 0, 141).into(),
            port: 40404,
            mode: "xsalsa20_poly1305_suffix".into(),
        },
    }
    .into();

    serde_test::assert_ser_tokens(&value.readable(), &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::SelectProtocol as u8),
        Token::Str("d"),
        Token::Struct {
            name: "SelectProtocol",
            len: 2,
        },
        Token::Str("data"),
        Token::Struct {
            name: "ProtocolData",
            len: 3,
        },
        Token::Str("address"),
        Token::Str("192.168.0.141"),
        Token::Str("mode"),
        Token::Str("xsalsa20_poly1305_suffix"),
        Token::Str("port"),
        Token::U16(40404),
        Token::StructEnd,
        Token::Str("protocol"),
        Token::Str("udp"),
        Token::StructEnd,
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_ready() {
    let value: Event = Ready {
        modes: vec![
            "xsalsa20_poly1305".into(),
            "xsalsa20_poly1305_suffix".into(),
            "xsalsa20_poly1305_lite".into(),
        ],
        ip: Ipv4Addr::new(127, 0, 0, 1).into(),
        port: 12345,
        ssrc: 0xcafe_d00d,
    }
    .into();

    serde_test::assert_ser_tokens(&value.readable(), &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::Ready as u8),
        Token::Str("d"),
        Token::Struct {
            name: "Ready",
            len: 4,
        },
        Token::Str("ip"),
        Token::Str("127.0.0.1"),
        Token::Str("modes"),
        Token::Seq {
            len: Some(3),
        },
        Token::Str("xsalsa20_poly1305"),
        Token::Str("xsalsa20_poly1305_suffix"),
        Token::Str("xsalsa20_poly1305_lite"),
        Token::SeqEnd,
        Token::Str("port"),
        Token::U16(12345),
        Token::Str("ssrc"),
        Token::U32(0xcafe_d00d),
        Token::StructEnd,
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_heartbeat() {
    let value: Event = Heartbeat {
        nonce: 1234567890,
    }
    .into();

    serde_test::assert_ser_tokens(&value, &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::Heartbeat as u8),
        Token::Str("d"),
        Token::Str("1234567890"),
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_session_description() {
    let value: Event = SessionDescription {
        mode: "xsalsa20_poly1305_suffix".into(),
        secret_key: vec![1, 2, 3, 4, 5],
    }
    .into();

    serde_test::assert_ser_tokens(&value, &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::SessionDescription as u8),
        Token::Str("d"),
        Token::Struct {
            name: "SessionDescription",
            len: 2,
        },
        Token::Str("mode"),
        Token::Str("xsalsa20_poly1305_suffix"),
        Token::Str("secret_key"),
        Token::Seq {
            len: Some(5),
        },
        Token::U8(1),
        Token::U8(2),
        Token::U8(3),
        Token::U8(4),
        Token::U8(5),
        Token::SeqEnd,
        Token::StructEnd,
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_speaking() {
    let value: Event = Speaking {
        delay: Some(0),
        speaking: SpeakingState::MICROPHONE,
        ssrc: 12345678,
        user_id: None,
    }
    .into();

    serde_test::assert_ser_tokens(&value, &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::Speaking as u8),
        Token::Str("d"),
        Token::Struct {
            name: "Speaking",
            len: 4,
        },
        Token::Str("delay"),
        Token::Some,
        Token::U32(0),
        Token::Str("speaking"),
        Token::U8(1),
        Token::Str("ssrc"),
        Token::U32(12345678),
        Token::Str("user_id"),
        Token::None,
        Token::StructEnd,
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_heartbeat_ack() {
    let value: Event = HeartbeatAck {
        nonce: 1234567890,
    }
    .into();

    serde_test::assert_ser_tokens(&value, &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::HeartbeatAck as u8),
        Token::Str("d"),
        Token::Str("1234567890"),
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_resume() {
    let value: Event = Resume {
        server_id: GuildId(1),
        session_id: "sess_sess_sess_sess".into(),
        token: "my_token".into(),
    }
    .into();

    serde_test::assert_ser_tokens(&value, &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::Resume as u8),
        Token::Str("d"),
        Token::Struct {
            name: "Resume",
            len: 3,
        },
        Token::Str("server_id"),
        Token::NewtypeStruct {
            name: "GuildId",
        },
        Token::Str("1"),
        Token::Str("session_id"),
        Token::Str("sess_sess_sess_sess"),
        Token::Str("token"),
        Token::Str("my_token"),
        Token::StructEnd,
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_hello() {
    let value: Event = Hello {
        heartbeat_interval: 41250.0,
    }
    .into();

    serde_test::assert_ser_tokens(&value, &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::Hello as u8),
        Token::Str("d"),
        Token::Struct {
            name: "Hello",
            len: 1,
        },
        Token::Str("heartbeat_interval"),
        Token::F64(41250.0),
        Token::StructEnd,
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_resumed() {
    let value = Event::Resumed;

    serde_test::assert_ser_tokens(&value, &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::Resumed as u8),
        Token::Str("d"),
        Token::None,
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_client_connect() {
    let value: Event = ClientConnect {
        audio_ssrc: 12345,
        user_id: UserId(56),
        video_ssrc: 67890,
    }
    .into();

    serde_test::assert_ser_tokens(&value, &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::ClientConnect as u8),
        Token::Str("d"),
        Token::Struct {
            name: "ClientConnect",
            len: 3,
        },
        Token::Str("audio_ssrc"),
        Token::U32(12345),
        Token::Str("user_id"),
        Token::NewtypeStruct {
            name: "UserId",
        },
        Token::Str("56"),
        Token::Str("video_ssrc"),
        Token::U32(67890),
        Token::StructEnd,
        Token::StructEnd,
    ]);
}

#[test]
fn serialize_client_disconnect() {
    let value: Event = ClientDisconnect {
        user_id: UserId(56),
    }
    .into();

    serde_test::assert_ser_tokens(&value, &[
        Token::Struct {
            name: "Event",
            len: 2,
        },
        Token::Str("op"),
        Token::U8(OpCode::ClientDisconnect as u8),
        Token::Str("d"),
        Token::Struct {
            name: "ClientDisconnect",
            len: 1,
        },
        Token::Str("user_id"),
        Token::NewtypeStruct {
            name: "UserId",
        },
        Token::Str("56"),
        Token::StructEnd,
        Token::StructEnd,
    ]);
}
