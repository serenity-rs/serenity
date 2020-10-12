use super::*;
use crate::{
    model::payload::{ClientConnect, ClientDisconnect, Speaking},
    tracks::{TrackHandle, TrackState},
};
use discortp::{rtcp::Rtcp, rtp::Rtp};

/// Information about which tracks or data fired an event.
///
/// [`Track`] events may be local or global, and have no tracks
/// if fired on the global context via [`Handler::add_global_event`].
///
/// [`Track`]: ../tracks/struct.Track.html
/// [`Handler::add_global_event`]: ../struct.Handler.html#method.add_global_event
#[derive(Clone, Debug)]
pub enum EventContext<'a> {
    /// Track event context, passed to events created via [`TrackHandle::add_event`],
    /// [`EventStore::add_event`], or relevant global events.
    ///
    /// [`EventStore::add_event`]: struct.EventStore.html#method.add_event
    /// [`TrackHandle::add_event`]: ../tracks/struct.TrackHandle.html#method.add_event
    Track(&'a [(&'a TrackState, &'a TrackHandle)]),
    /// Speaking state update, typically describing how another voice
    /// user is transmitting audio data. Clients must send at least one such
    /// packet to allow SSRC/UserID matching.
    SpeakingStateUpdate(Speaking),
    /// Speaking state transition, describing whether a given source has started/stopped
    /// transmitting. This fires in response to a silent burst, or the first packet
    /// breaking such a burst.
    SpeakingUpdate { ssrc: u32, speaking: bool },
    /// Opus audio packet, received from another stream (detailed in `packet`).
    /// `payload_offset` contains the true payload location within the raw packet's `payload()`,
    /// if extensions or raw packet data are required.
    /// if `audio.len() == 0`, then this packet arrived out-of-order.
    VoicePacket {
        audio: &'a Vec<i16>,
        packet: &'a Rtp,
        payload_offset: usize,
    },
    /// Telemetry/statistics packet, received from another stream (detailed in `packet`).
    /// `payload_offset` contains the true payload location within the raw packet's `payload()`,
    /// to allow manual decoding of `Rtcp` packet bodies.
    RtcpPacket {
        packet: &'a Rtcp,
        payload_offset: usize,
    },
    /// Fired whenever a client connects to a call for the first time, allowing SSRC/UserID
    /// matching.
    ClientConnect(ClientConnect),
    /// Fired whenever a client disconnects.
    ClientDisconnect(ClientDisconnect),
}

#[derive(Clone, Debug)]
pub(crate) enum CoreContext {
    SpeakingStateUpdate(Speaking),
    SpeakingUpdate {
        ssrc: u32,
        speaking: bool,
    },
    VoicePacket {
        audio: Vec<i16>,
        packet: Rtp,
        payload_offset: usize,
    },
    RtcpPacket {
        packet: Rtcp,
        payload_offset: usize,
    },
    ClientConnect(ClientConnect),
    ClientDisconnect(ClientDisconnect),
}

impl<'a> CoreContext {
    pub(crate) fn to_user_context(&'a self) -> EventContext<'a> {
        use CoreContext::*;

        match self {
            SpeakingStateUpdate(evt) => EventContext::SpeakingStateUpdate(*evt),
            SpeakingUpdate { ssrc, speaking } => EventContext::SpeakingUpdate {
                ssrc: *ssrc,
                speaking: *speaking,
            },
            VoicePacket {
                audio,
                packet,
                payload_offset,
            } => EventContext::VoicePacket {
                audio,
                packet,
                payload_offset: *payload_offset,
            },
            RtcpPacket {
                packet,
                payload_offset,
            } => EventContext::RtcpPacket {
                packet,
                payload_offset: *payload_offset,
            },
            ClientConnect(evt) => EventContext::ClientConnect(*evt),
            ClientDisconnect(evt) => EventContext::ClientDisconnect(*evt),
        }
    }
}

impl EventContext<'_> {
    pub fn to_core_event(&self) -> Option<CoreEvent> {
        use EventContext::*;

        match self {
            SpeakingStateUpdate { .. } => Some(CoreEvent::SpeakingStateUpdate),
            SpeakingUpdate { .. } => Some(CoreEvent::SpeakingUpdate),
            VoicePacket { .. } => Some(CoreEvent::VoicePacket),
            RtcpPacket { .. } => Some(CoreEvent::RtcpPacket),
            ClientConnect { .. } => Some(CoreEvent::ClientConnect),
            ClientDisconnect { .. } => Some(CoreEvent::ClientDisconnect),
            _ => None,
        }
    }
}
