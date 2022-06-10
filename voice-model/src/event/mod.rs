mod from;
#[cfg(test)]
mod tests;

use serde::de::value::U8Deserializer;
use serde::de::{Deserializer, Error as DeError, IntoDeserializer, MapAccess, Unexpected, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::opcode::OpCode;
use crate::payload::*;

/// A representation of data received for voice gateway events.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Event {
    /// Used to begin a voice websocket connection.
    Identify(Identify),
    /// Used to select the voice protocol and encryption mechanism.
    SelectProtocol(SelectProtocol),
    /// Server's response to the client's Identify operation.
    /// Contains session-specific information, e.g.
    /// SSRC, and supported encryption modes.
    Ready(Ready),
    /// Periodic messages used to keep the websocket connection alive.
    Heartbeat(Heartbeat),
    /// Server's confirmation of a negotiated encryption scheme.
    SessionDescription(SessionDescription),
    /// A voice event denoting that someone is speaking.
    Speaking(Speaking),
    /// Acknowledgement from the server for a prior voice heartbeat.
    HeartbeatAck(HeartbeatAck),
    /// Sent by the client after a disconnect to attempt to resume a session.
    Resume(Resume),
    /// Used to determine how often the client must send a heartbeat.
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
    where
        S: Serializer,
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
        formatter.write_str("a map with at least two keys ('d', 'op')")
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
                    let valid_op = OpCode::deserialize(des).map_err(|_| {
                        DeError::invalid_value(
                            Unexpected::Unsigned(raw.into()),
                            &"opcode in [0--9] + [12--13]",
                        )
                    })?;
                    op = Some(valid_op);
                },
                // Idea: Op comes first, but missing it is not failure.
                // So, if order correct then we don't need to pass the RawValue back out.
                Some("d") => match op {
                    Some(OpCode::Identify) => return Ok(map.next_value::<Identify>()?.into()),
                    Some(OpCode::SelectProtocol) =>
                        return Ok(map.next_value::<SelectProtocol>()?.into()),
                    Some(OpCode::Ready) => return Ok(map.next_value::<Ready>()?.into()),
                    Some(OpCode::Heartbeat) => return Ok(map.next_value::<Heartbeat>()?.into()),
                    Some(OpCode::HeartbeatAck) =>
                        return Ok(map.next_value::<HeartbeatAck>()?.into()),
                    Some(OpCode::SessionDescription) =>
                        return Ok(map.next_value::<SessionDescription>()?.into()),
                    Some(OpCode::Speaking) => return Ok(map.next_value::<Speaking>()?.into()),
                    Some(OpCode::Resume) => return Ok(map.next_value::<Resume>()?.into()),
                    Some(OpCode::Hello) => return Ok(map.next_value::<Hello>()?.into()),
                    Some(OpCode::Resumed) => {
                        let _ = map.next_value::<Option<()>>()?;
                        return Ok(Event::Resumed);
                    },
                    Some(OpCode::ClientConnect) =>
                        return Ok(map.next_value::<ClientConnect>()?.into()),
                    Some(OpCode::ClientDisconnect) =>
                        return Ok(map.next_value::<ClientDisconnect>()?.into()),
                    None => {
                        d = Some(map.next_value::<&RawValue>()?);
                    },
                },
                Some(_) => {},
                None =>
                    if d.is_none() {
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
            OpCode::Identify => serde_json::from_str::<Identify>(d).map(Into::into),
            OpCode::SelectProtocol => serde_json::from_str::<SelectProtocol>(d).map(Into::into),
            OpCode::Ready => serde_json::from_str::<Ready>(d).map(Into::into),
            OpCode::Heartbeat => serde_json::from_str::<Heartbeat>(d).map(Into::into),
            OpCode::HeartbeatAck => serde_json::from_str::<HeartbeatAck>(d).map(Into::into),
            OpCode::SessionDescription =>
                serde_json::from_str::<SessionDescription>(d).map(Into::into),
            OpCode::Speaking => serde_json::from_str::<Speaking>(d).map(Into::into),
            OpCode::Resume => serde_json::from_str::<Resume>(d).map(Into::into),
            OpCode::Hello => serde_json::from_str::<Hello>(d).map(Into::into),
            OpCode::Resumed => Ok(Event::Resumed),
            OpCode::ClientConnect => serde_json::from_str::<ClientConnect>(d).map(Into::into),
            OpCode::ClientDisconnect => serde_json::from_str::<ClientDisconnect>(d).map(Into::into),
        })
        .map_err(DeError::custom)
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(EventVisitor)
    }
}
