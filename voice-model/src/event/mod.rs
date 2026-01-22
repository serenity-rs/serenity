mod from;
#[cfg(test)]
mod tests;

use serde::de::value::U8Deserializer;
use serde::de::{Deserializer, Error as DeError, IntoDeserializer, MapAccess, Unexpected, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::opcode::Opcode;
use crate::payload::*;

/// A representation of data received for voice gateway events.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Event {
    /// Used to begin a voice websocket connection.
    Identify(Identify),
    /// Used to select the voice protocol and encryption mechanism.
    SelectProtocol(SelectProtocol),
    /// Server's response to the client's Identify operation. Contains session-specific
    /// information, e.g. SSRC, and supported encryption modes.
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
    /// Status update in the current channel, indicating that a user has connected.
    ClientConnect(ClientConnect),
    /// Status update in the current channel, indicating that a user has disconnected.
    ClientDisconnect(ClientDisconnect),
    /// Video stream information (unused by bots).
    Video(Video),
    /// List of user IDs currently in the voice channel.
    ClientsConnect(ClientsConnect),
    /// Media sink wants update (unused by bots).
    MediaSinkWants(MediaSinkWants),
    /// Voice backend version (unused by bots).
    VoiceBackendVersion(VoiceBackendVersion),
    /// Channel options update (unused by bots).
    ChannelOptionsUpdate(ChannelOptionsUpdate),
    /// User flags update (muted, deafened, etc.).
    Flags(Flags),
    /// Platform information for a user.
    Platform(Platform),
    /// DAVE: Signals the bot is ready for group operations after epoch transition.
    DaveTransitionReady(DaveTransitionReady),
    /// DAVE: Notifies of an upcoming epoch change.
    DavePrepareEpoch(DavePrepareEpoch),
    /// DAVE: Provides the external sender package for MLS group initialization.
    DaveMlsExternalSender(DaveMlsExternalSender),
    /// DAVE: Sends the key package for MLS group participation.
    DaveMlsKeyPackage(DaveMlsKeyPackage),
    /// DAVE: Provides proposals for group member changes.
    DaveMlsProposals(DaveMlsProposals),
    /// DAVE: Provides a commit with optional welcome for group transitions.
    DaveMlsCommitWelcome(DaveMlsCommitWelcome),
    /// DAVE: Provides the welcome message for new members.
    DaveMlsWelcome(DaveMlsWelcome),
    /// DAVE: Prepares for a protocol transition.
    DavePrepareTransition(DavePrepareTransition),
    /// DAVE: Executes a prepared protocol transition.
    DaveExecuteTransition(DaveExecuteTransition),
    /// DAVE: Announces a commit for group transition.
    DaveMlsAnnounceCommitTransition(DaveMlsAnnounceCommitTransition),
    /// DAVE: Reports an invalid commit or welcome message.
    DaveMlsInvalidCommitWelcome(DaveMlsInvalidCommitWelcome),
}

impl Event {
    pub fn kind(&self) -> Opcode {
        use Event::*;
        match self {
            Identify(_) => Opcode::Identify,
            SelectProtocol(_) => Opcode::SelectProtocol,
            Ready(_) => Opcode::Ready,
            Heartbeat(_) => Opcode::Heartbeat,
            SessionDescription(_) => Opcode::SessionDescription,
            Speaking(_) => Opcode::Speaking,
            HeartbeatAck(_) => Opcode::HeartbeatAck,
            Resume(_) => Opcode::Resume,
            Hello(_) => Opcode::Hello,
            Resumed => Opcode::Resumed,
            Video(_) => Opcode::Video,
            ClientsConnect(_) => Opcode::ClientsConnect,
            ClientConnect(_) => Opcode::ClientConnect,
            ClientDisconnect(_) => Opcode::ClientDisconnect,
            MediaSinkWants(_) => Opcode::MediaSinkWants,
            VoiceBackendVersion(_) => Opcode::VoiceBackendVersion,
            ChannelOptionsUpdate(_) => Opcode::ChannelOptionsUpdate,
            Flags(_) => Opcode::Flags,
            Platform(_) => Opcode::Platform,
            DaveTransitionReady(_) => Opcode::DaveTransitionReady,
            DavePrepareEpoch(_) => Opcode::DavePrepareEpoch,
            DaveMlsExternalSender(_) => Opcode::DaveMlsExternalSender,
            DaveMlsKeyPackage(_) => Opcode::DaveMlsKeyPackage,
            DaveMlsProposals(_) => Opcode::DaveMlsProposals,
            DaveMlsCommitWelcome(_) => Opcode::DaveMlsCommitWelcome,
            DaveMlsWelcome(_) => Opcode::DaveMlsWelcome,
            DavePrepareTransition(_) => Opcode::DavePrepareTransition,
            DaveExecuteTransition(_) => Opcode::DaveExecuteTransition,
            DaveMlsAnnounceCommitTransition(_) => Opcode::DaveMlsAnnounceCommitTransition,
            DaveMlsInvalidCommitWelcome(_) => Opcode::DaveMlsInvalidCommitWelcome,
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
            Video(e) => s.serialize_field("d", e)?,
            ClientsConnect(e) => s.serialize_field("d", e)?,
            ClientConnect(e) => s.serialize_field("d", e)?,
            ClientDisconnect(e) => s.serialize_field("d", e)?,
            MediaSinkWants(e) => s.serialize_field("d", e)?,
            VoiceBackendVersion(e) => s.serialize_field("d", e)?,
            ChannelOptionsUpdate(e) => s.serialize_field("d", e)?,
            Flags(e) => s.serialize_field("d", e)?,
            Platform(e) => s.serialize_field("d", e)?,
            DaveTransitionReady(e) => s.serialize_field("d", e)?,
            DavePrepareEpoch(e) => s.serialize_field("d", e)?,
            DaveMlsExternalSender(e) => s.serialize_field("d", e)?,
            DaveMlsKeyPackage(e) => s.serialize_field("d", e)?,
            DaveMlsProposals(e) => s.serialize_field("d", e)?,
            DaveMlsCommitWelcome(e) => s.serialize_field("d", e)?,
            DaveMlsWelcome(e) => s.serialize_field("d", e)?,
            DavePrepareTransition(e) => s.serialize_field("d", e)?,
            DaveExecuteTransition(e) => s.serialize_field("d", e)?,
            DaveMlsAnnounceCommitTransition(e) => s.serialize_field("d", e)?,
            DaveMlsInvalidCommitWelcome(e) => s.serialize_field("d", e)?,
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
                    let valid_op = Opcode::deserialize(des).map_err(|_| {
                        DeError::invalid_value(
                            Unexpected::Unsigned(raw.into()),
                            &"opcode in [0--20] + [21--31]",
                        )
                    })?;
                    op = Some(valid_op);
                },
                // Idea: Op comes first, but missing it is not failure.
                // So, if order correct then we don't need to pass the RawValue back out.
                Some("d") => match op {
                    Some(Opcode::Identify) => return Ok(map.next_value::<Identify>()?.into()),
                    Some(Opcode::SelectProtocol) => {
                        return Ok(map.next_value::<SelectProtocol>()?.into());
                    },
                    Some(Opcode::Ready) => return Ok(map.next_value::<Ready>()?.into()),
                    Some(Opcode::Heartbeat) => return Ok(map.next_value::<Heartbeat>()?.into()),
                    Some(Opcode::HeartbeatAck) => {
                        return Ok(map.next_value::<HeartbeatAck>()?.into());
                    },
                    Some(Opcode::SessionDescription) => {
                        return Ok(map.next_value::<SessionDescription>()?.into());
                    },
                    Some(Opcode::Speaking) => return Ok(map.next_value::<Speaking>()?.into()),
                    Some(Opcode::Resume) => return Ok(map.next_value::<Resume>()?.into()),
                    Some(Opcode::Hello) => return Ok(map.next_value::<Hello>()?.into()),
                    Some(Opcode::Resumed) => {
                        let _ = map.next_value::<Option<()>>()?;
                        return Ok(Event::Resumed);
                    },
                    Some(Opcode::Video) => {
                        return Ok(map.next_value::<Video>()?.into());
                    },
                    Some(Opcode::ClientsConnect) => {
                        return Ok(map.next_value::<ClientsConnect>()?.into());
                    },
                    Some(Opcode::ClientConnect) => {
                        return Ok(map.next_value::<ClientConnect>()?.into());
                    },
                    Some(Opcode::ClientDisconnect) => {
                        return Ok(map.next_value::<ClientDisconnect>()?.into());
                    },
                    Some(Opcode::MediaSinkWants) => {
                        return Ok(map.next_value::<MediaSinkWants>()?.into());
                    },
                    Some(Opcode::VoiceBackendVersion) => {
                        return Ok(map.next_value::<VoiceBackendVersion>()?.into());
                    },
                    Some(Opcode::ChannelOptionsUpdate) => {
                        return Ok(map.next_value::<ChannelOptionsUpdate>()?.into());
                    },
                    Some(Opcode::Flags) => {
                        return Ok(map.next_value::<Flags>()?.into());
                    },
                    Some(Opcode::Platform) => {
                        return Ok(map.next_value::<Platform>()?.into());
                    },
                    Some(Opcode::DaveTransitionReady) => {
                        return Ok(map.next_value::<DaveTransitionReady>()?.into());
                    },
                    Some(Opcode::DavePrepareEpoch) => {
                        return Ok(map.next_value::<DavePrepareEpoch>()?.into());
                    },
                    Some(Opcode::DaveMlsExternalSender) => {
                        return Ok(map.next_value::<DaveMlsExternalSender>()?.into());
                    },
                    Some(Opcode::DaveMlsKeyPackage) => {
                        return Ok(map.next_value::<DaveMlsKeyPackage>()?.into());
                    },
                    Some(Opcode::DaveMlsProposals) => {
                        return Ok(map.next_value::<DaveMlsProposals>()?.into());
                    },
                    Some(Opcode::DaveMlsCommitWelcome) => {
                        return Ok(map.next_value::<DaveMlsCommitWelcome>()?.into());
                    },
                    Some(Opcode::DaveMlsWelcome) => {
                        return Ok(map.next_value::<DaveMlsWelcome>()?.into());
                    },
                    Some(Opcode::DavePrepareTransition) => {
                        return Ok(map.next_value::<DavePrepareTransition>()?.into());
                    },
                    Some(Opcode::DaveExecuteTransition) => {
                        return Ok(map.next_value::<DaveExecuteTransition>()?.into());
                    },
                    Some(Opcode::DaveMlsAnnounceCommitTransition) => {
                        return Ok(map.next_value::<DaveMlsAnnounceCommitTransition>()?.into());
                    },
                    Some(Opcode::DaveMlsInvalidCommitWelcome) => {
                        return Ok(map.next_value::<DaveMlsInvalidCommitWelcome>()?.into());
                    },
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
            Opcode::Identify => serde_json::from_str::<Identify>(d).map(Into::into),
            Opcode::SelectProtocol => serde_json::from_str::<SelectProtocol>(d).map(Into::into),
            Opcode::Ready => serde_json::from_str::<Ready>(d).map(Into::into),
            Opcode::Heartbeat => serde_json::from_str::<Heartbeat>(d).map(Into::into),
            Opcode::HeartbeatAck => serde_json::from_str::<HeartbeatAck>(d).map(Into::into),
            Opcode::SessionDescription =>
                serde_json::from_str::<SessionDescription>(d).map(Into::into),
            Opcode::Speaking => serde_json::from_str::<Speaking>(d).map(Into::into),
            Opcode::Resume => serde_json::from_str::<Resume>(d).map(Into::into),
            Opcode::Hello => serde_json::from_str::<Hello>(d).map(Into::into),
            Opcode::Resumed => Ok(Event::Resumed),
            Opcode::Video => serde_json::from_str::<Video>(d).map(Into::into),
            Opcode::ClientsConnect => serde_json::from_str::<ClientsConnect>(d).map(Into::into),
            Opcode::ClientConnect => serde_json::from_str::<ClientConnect>(d).map(Into::into),
            Opcode::ClientDisconnect => serde_json::from_str::<ClientDisconnect>(d).map(Into::into),
            Opcode::MediaSinkWants => serde_json::from_str::<MediaSinkWants>(d).map(Into::into),
            Opcode::VoiceBackendVersion =>
                serde_json::from_str::<VoiceBackendVersion>(d).map(Into::into),
            Opcode::ChannelOptionsUpdate =>
                serde_json::from_str::<ChannelOptionsUpdate>(d).map(Into::into),
            Opcode::Flags => serde_json::from_str::<Flags>(d).map(Into::into),
            Opcode::Platform => serde_json::from_str::<Platform>(d).map(Into::into),
            Opcode::DaveTransitionReady =>
                serde_json::from_str::<DaveTransitionReady>(d).map(Into::into),
            Opcode::DavePrepareEpoch => serde_json::from_str::<DavePrepareEpoch>(d).map(Into::into),
            Opcode::DaveMlsExternalSender =>
                serde_json::from_str::<DaveMlsExternalSender>(d).map(Into::into),
            Opcode::DaveMlsKeyPackage =>
                serde_json::from_str::<DaveMlsKeyPackage>(d).map(Into::into),
            Opcode::DaveMlsProposals => serde_json::from_str::<DaveMlsProposals>(d).map(Into::into),
            Opcode::DaveMlsCommitWelcome =>
                serde_json::from_str::<DaveMlsCommitWelcome>(d).map(Into::into),
            Opcode::DaveMlsWelcome => serde_json::from_str::<DaveMlsWelcome>(d).map(Into::into),
            Opcode::DavePrepareTransition =>
                serde_json::from_str::<DavePrepareTransition>(d).map(Into::into),
            Opcode::DaveExecuteTransition =>
                serde_json::from_str::<DaveExecuteTransition>(d).map(Into::into),
            Opcode::DaveMlsAnnounceCommitTransition =>
                serde_json::from_str::<DaveMlsAnnounceCommitTransition>(d).map(Into::into),
            Opcode::DaveMlsInvalidCommitWelcome =>
                serde_json::from_str::<DaveMlsInvalidCommitWelcome>(d).map(Into::into),
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
