use super::Event;
use crate::payload::*;

impl From<Identify> for Event {
    fn from(i: Identify) -> Self {
        Event::Identify(i)
    }
}

impl From<SelectProtocol> for Event {
    fn from(i: SelectProtocol) -> Self {
        Event::SelectProtocol(i)
    }
}

impl From<Ready> for Event {
    fn from(i: Ready) -> Self {
        Event::Ready(i)
    }
}

impl From<Heartbeat> for Event {
    fn from(i: Heartbeat) -> Self {
        Event::Heartbeat(i)
    }
}

impl From<SessionDescription> for Event {
    fn from(i: SessionDescription) -> Self {
        Event::SessionDescription(i)
    }
}

impl From<Speaking> for Event {
    fn from(i: Speaking) -> Self {
        Event::Speaking(i)
    }
}

impl From<HeartbeatAck> for Event {
    fn from(i: HeartbeatAck) -> Self {
        Event::HeartbeatAck(i)
    }
}

impl From<Resume> for Event {
    fn from(i: Resume) -> Self {
        Event::Resume(i)
    }
}

impl From<Hello> for Event {
    fn from(i: Hello) -> Self {
        Event::Hello(i)
    }
}

impl From<ClientConnect> for Event {
    fn from(i: ClientConnect) -> Self {
        Event::ClientConnect(i)
    }
}

impl From<ClientDisconnect> for Event {
    fn from(i: ClientDisconnect) -> Self {
        Event::ClientDisconnect(i)
    }
}

impl From<Video> for Event {
    fn from(i: Video) -> Self {
        Event::Video(i)
    }
}

impl From<ClientsConnect> for Event {
    fn from(i: ClientsConnect) -> Self {
        Event::ClientsConnect(i)
    }
}

impl From<MediaSinkWants> for Event {
    fn from(i: MediaSinkWants) -> Self {
        Event::MediaSinkWants(i)
    }
}

impl From<VoiceBackendVersion> for Event {
    fn from(i: VoiceBackendVersion) -> Self {
        Event::VoiceBackendVersion(i)
    }
}

impl From<ChannelOptionsUpdate> for Event {
    fn from(i: ChannelOptionsUpdate) -> Self {
        Event::ChannelOptionsUpdate(i)
    }
}

impl From<Flags> for Event {
    fn from(i: Flags) -> Self {
        Event::Flags(i)
    }
}

impl From<Platform> for Event {
    fn from(i: Platform) -> Self {
        Event::Platform(i)
    }
}

impl From<DaveTransitionReady> for Event {
    fn from(i: DaveTransitionReady) -> Self {
        Event::DaveTransitionReady(i)
    }
}

impl From<DavePrepareEpoch> for Event {
    fn from(i: DavePrepareEpoch) -> Self {
        Event::DavePrepareEpoch(i)
    }
}

impl From<DaveMlsExternalSender> for Event {
    fn from(i: DaveMlsExternalSender) -> Self {
        Event::DaveMlsExternalSender(i)
    }
}

impl From<DaveMlsKeyPackage> for Event {
    fn from(i: DaveMlsKeyPackage) -> Self {
        Event::DaveMlsKeyPackage(i)
    }
}

impl From<DaveMlsProposals> for Event {
    fn from(i: DaveMlsProposals) -> Self {
        Event::DaveMlsProposals(i)
    }
}

impl From<DaveMlsCommitWelcome> for Event {
    fn from(i: DaveMlsCommitWelcome) -> Self {
        Event::DaveMlsCommitWelcome(i)
    }
}

impl From<DaveMlsWelcome> for Event {
    fn from(i: DaveMlsWelcome) -> Self {
        Event::DaveMlsWelcome(i)
    }
}

impl From<DavePrepareTransition> for Event {
    fn from(i: DavePrepareTransition) -> Self {
        Event::DavePrepareTransition(i)
    }
}

impl From<DaveExecuteTransition> for Event {
    fn from(i: DaveExecuteTransition) -> Self {
        Event::DaveExecuteTransition(i)
    }
}

impl From<DaveMlsAnnounceCommitTransition> for Event {
    fn from(i: DaveMlsAnnounceCommitTransition) -> Self {
        Event::DaveMlsAnnounceCommitTransition(i)
    }
}

impl From<DaveMlsInvalidCommitWelcome> for Event {
    fn from(i: DaveMlsInvalidCommitWelcome) -> Self {
        Event::DaveMlsInvalidCommitWelcome(i)
    }
}
