//! All the events this library handles.
use serde::de::IntoDeserializer;
use serde::de::Unexpected;
use serde::de::Visitor;
use serde::de::value::U8Deserializer;
use serde::{
    de::{Deserializer, Error as DeError, MapAccess},
    ser::{SerializeStruct, Serializer},
    Deserialize,
    Serialize,
};
use serde_json::value::RawValue;
use crate::{
    opcode::OpCode,
    payload::*,
};
// FIXME: add new types.

/// A representation of data received for [`voice`] events.
///
/// [`voice`]: ../../voice/index.html
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Event {
    /// TODO
    Identify(Identify),
    /// TODO
    SelectProtocol(SelectProtocol),
    /// Server's response to the client's Identify operation.
    /// Contains session-specific information, e.g.
    /// [`ssrc`] and supported encryption modes.
    ///
    /// [`ssrc`]: struct.Ready.html#structfield.ssrc
    Ready(Ready),
    /// TODO
    Heartbeat(Heartbeat),
    /// A voice event describing the current session.
    SessionDescription(SessionDescription),
    /// A voice event denoting that someone is speaking.
    Speaking(Speaking),
    /// Acknowledgement from the server for a prior voice heartbeat.
    HeartbeatAck(HeartbeatAck),
    /// TODO
    Resume(Resume),
    /// A "hello" was received with initial voice data, such as the
    /// true [`heartbeat_interval`].
    ///
    /// [`heartbeat_interval`]: struct.Hello.html#structfield.heartbeat_interval
    Hello(Hello),
    /// Message received if a Resume request was successful.
    Resumed,
    /// Status update in the current channel, indicating that a user has
    /// connected.
    ClientConnect(ClientConnect),
    /// Status update in the current channel, indicating that a user has
    /// disconnected.
    ClientDisconnect(ClientDisconnect),
}

impl Event {
    pub fn kind(&self) -> OpCode {
        use Event::*;
        match self {
            Identify(_) => OpCode::Identify,
            SelectProtocol(_) => OpCode::SelectProtocol,
            Ready(_) => OpCode::Ready,
            Heartbeat(_) => OpCode::Heartbeat,
            SessionDescription(_) => OpCode::SessionDescription,
            Speaking(_) => OpCode::Speaking,
            HeartbeatAck(_) => OpCode::HeartbeatAck,
            Resume(_) => OpCode::Resume,
            Hello(_) => OpCode::Hello,
            Resumed => OpCode::Resumed,
            ClientConnect(_) => OpCode::ClientConnect,
            ClientDisconnect(_) => OpCode::ClientDisconnect,
        }
    }
}

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut s = serializer.serialize_struct("Event", 2)?;

        s.serialize_field("op", &self.kind())?;

        use Event::*;
        match self {
            Identify(e) => s.serialize_field("d", e)?,
            SelectProtocol(e) => s.serialize_field("d", e)?,
            Ready(e) => s.serialize_field("d", e)?,
            Heartbeat(e) => s.serialize_field("d", e)?,
            SessionDescription(e) => s.serialize_field("d", e)?,
            Speaking(e) => s.serialize_field("d", e)?,
            HeartbeatAck(e) => s.serialize_field("d", e)?,
            Resume(e) => s.serialize_field("d", e)?,
            Hello(e) => s.serialize_field("d", e)?,
            Resumed => s.serialize_field("d", &None::<()>)?,
            ClientConnect(e) => s.serialize_field("d", e)?,
            ClientDisconnect(e) => s.serialize_field("d", e)?,
        }

        s.end()
    }
}

struct EventVisitor;

impl<'de> Visitor<'de> for EventVisitor {
    type Value = Event;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map with at least two keys ('d', 'op')")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut d = None;
        let mut op = None;

        loop {
            match map.next_key::<&str>()? {
                Some("op") => {
                    let raw = map.next_value::<u8>()?;
                    let des: U8Deserializer<A::Error> = raw.into_deserializer();
                    let valid_op = OpCode::deserialize(des)
                        .map_err(|_| DeError::invalid_value(Unexpected::Unsigned(raw.into()), &"opcode in [0--9] + [12--13]"))?;
                    op = Some(valid_op);
                },
                // Idea: Op comes first, but missing it is not failure.
                // So, if order correct then we don't need to pass the RawValue back out.
                Some("d") => match op {
                    Some(OpCode::Identify) => { return Ok(Event::Identify(map.next_value::<Identify>()?))},
                    Some(OpCode::SelectProtocol) => { return Ok(Event::SelectProtocol(map.next_value::<SelectProtocol>()?))},
                    Some(OpCode::Ready) => { return Ok(Event::Ready(map.next_value::<Ready>()?))},
                    Some(OpCode::Heartbeat) => { return Ok(Event::Heartbeat(map.next_value::<Heartbeat>()?))},
                    Some(OpCode::HeartbeatAck) => { return Ok(Event::HeartbeatAck(map.next_value::<HeartbeatAck>()?))},
                    Some(OpCode::SessionDescription) => { return Ok(Event::SessionDescription(map.next_value::<SessionDescription>()?))},
                    Some(OpCode::Speaking) => { return Ok(Event::Speaking(map.next_value::<Speaking>()?))},
                    Some(OpCode::Resume) => { return Ok(Event::Resume(map.next_value::<Resume>()?))},
                    Some(OpCode::Hello) => { return Ok(Event::Hello(map.next_value::<Hello>()?))},
                    Some(OpCode::Resumed) => { let _ = map.next_value::<Option<()>>()?; return Ok(Event::Resumed)},
                    Some(OpCode::ClientConnect) => { return Ok(Event::ClientConnect(map.next_value::<ClientConnect>()?))},
                    Some(OpCode::ClientDisconnect) => { return Ok(Event::ClientDisconnect(map.next_value::<ClientDisconnect>()?))},
                    None => {
                        d = Some(map.next_value::<&RawValue>()?);
                    }
                },
                Some(_) => {},
                None => if d.is_none() {
                        return Err(DeError::missing_field("d"));
                    } else if op.is_none() {
                        return Err(DeError::missing_field("op"));
                    },
            }

            if d.is_some() && op.is_some() {
                break;
            }
        }

        let d = d.expect("Struct body known to exist if loop has been escaped.").get();
        let op = op.expect("Struct variant known to exist if loop has been escaped.");

        (match op {
            OpCode::Identify => serde_json::from_str::<Identify>(d).map(Event::Identify),
            OpCode::SelectProtocol => serde_json::from_str::<SelectProtocol>(d).map(Event::SelectProtocol),
            OpCode::Ready => serde_json::from_str::<Ready>(d).map(Event::Ready),
            OpCode::Heartbeat => serde_json::from_str::<Heartbeat>(d).map(Event::Heartbeat),
            OpCode::HeartbeatAck => serde_json::from_str::<HeartbeatAck>(d).map(Event::HeartbeatAck),
            OpCode::SessionDescription => serde_json::from_str::<SessionDescription>(d).map(Event::SessionDescription),
            OpCode::Speaking => serde_json::from_str::<Speaking>(d).map(Event::Speaking),
            OpCode::Resume => serde_json::from_str::<Resume>(d).map(Event::Resume),
            OpCode::Hello => serde_json::from_str::<Hello>(d).map(Event::Hello),
            OpCode::Resumed => Ok(Event::Resumed),
            OpCode::ClientConnect => serde_json::from_str::<ClientConnect>(d).map(Event::ClientConnect),
            OpCode::ClientDisconnect => serde_json::from_str::<ClientDisconnect>(d).map(Event::ClientDisconnect),
        }).map_err(DeError::custom)
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_map(EventVisitor)
    }
}

#[cfg(test)]
mod tests {
        use std::net::Ipv4Addr;

use super::Event;
    use crate::{
        id::*,
        opcode::OpCode,
        payload::*,
    };
    use serde_test::{Configure, Token};


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

        let event = serde_json::from_str(&json_data);

        let ident = Identify {
            session_id: "my_session_id".into(),
            token: "my_token".into(),
            server_id: GuildId(41771983423143937),
            user_id: UserId(104694319306248192),
        };

        assert!(
            match event {
                Ok(Event::Identify(i)) if i == ident => true,
                _ => false,
            }
        );
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

        let event = serde_json::from_str(&json_data);

        let ready = Ready {
            ssrc: 1,
            ip: Ipv4Addr::new(127,0,0,1).into(),
            port: 1234,
            modes: vec![
                "xsalsa20_poly1305".into(),
                "xsalsa20_poly1305_suffix".into(),
                "xsalsa20_poly1305_lite".into(),
            ],
        };

        assert!(
            match event {
                Ok(Event::Ready(i)) if i == ready => true,
                _ => false,
            }
        );
    }

    #[test]
    fn serialize_identify() {
        let value = Event::Identify(Identify {
            server_id: GuildId(1),
            session_id: "56f88a86dce65c65b9".into(),
            token: "56f88a86dce65c65b8".into(),
            user_id: UserId(2),
        });

        serde_test::assert_ser_tokens(
            &value,
            &[
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
                Token::NewtypeStruct { name: "GuildId" },
                Token::Str("1"),
                Token::Str("session_id"),
                Token::Str("56f88a86dce65c65b9"),
                Token::Str("token"),
                Token::Str("56f88a86dce65c65b8"),
                Token::Str("user_id"),
                Token::NewtypeStruct { name: "UserId" },
                Token::Str("2"),
                Token::StructEnd,
                Token::StructEnd,
            ]
        );
    }

    #[test]
    fn serialize_ready() {
        let value = Event::Ready(Ready {
            modes: vec![
                "xsalsa20_poly1305".into(),
                "xsalsa20_poly1305_suffix".into(),
                "xsalsa20_poly1305_lite".into(),
            ],
            ip: Ipv4Addr::new(127,0,0,1).into(),
            port: 12345,
            ssrc: 0xcafe_d00d,
        });

        println!("{:?}", serde_json::to_string_pretty(&value));

        serde_test::assert_ser_tokens(
            &value.compact(),
            &[
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
                Token::NewtypeVariant { name: "IpAddr", variant: "V4" },
                Token::Tuple{ len: 4 },
                Token::U8(127),
                Token::U8(0),
                Token::U8(0),
                Token::U8(1),
                Token::TupleEnd,
                Token::Str("modes"),
                Token::Seq{ len: Some(3) },
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
            ]
        );
    }
}
