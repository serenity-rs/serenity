mod aux_network;
pub mod error;
mod events;
mod mixer;

use audiopus::Bitrate;
use serenity::{
    gateway::WsStream,
    model::id::GuildId,
};
use crate::{
    connection::{error::Error as ConnectionError, Connection},
    events::{
        CoreContext,
        EventData,
        EventStore,
    },
    tracks::{
        LoopState,
        PlayMode,
        Track,
        TrackHandle,
        TrackState,
    },
    Status,
};
use flume::{
    Receiver,
    Sender,
    RecvError,
};
use tracing::{error, info, warn};
use std::time::Duration;
use tokio::net::udp::RecvHalf;
use xsalsa20poly1305::XSalsa20Poly1305 as Cipher;

#[derive(Clone, Debug)]
pub(crate) struct Interconnect {
    pub(crate) core: Sender<Status>,
    pub(crate) events: Sender<EventMessage>,
    pub(crate) aux_packets: Sender<AuxPacketMessage>,
    pub(crate) mixer: Sender<MixerMessage>,
}

impl Interconnect {
    fn poison(&self) {
        let _ = self.events.send(EventMessage::Poison);
        let _ = self.aux_packets.send(AuxPacketMessage::Poison);
    }

    fn poison_all(&self) {
        self.poison();
        let _ = self.mixer.send(MixerMessage::Poison);
    }

    fn restart(self, guild_id: GuildId) -> Self {
        self.poison();
        start_internals(guild_id, self.core)
    }

    fn restart_volatile_internals(&mut self, guild_id: GuildId) {
        self.poison();
        
        let (evt_tx, evt_rx) = flume::unbounded();
        let (pkt_aux_tx, pkt_aux_rx) = flume::unbounded();

        self.events = evt_tx;
        self.aux_packets = pkt_aux_tx;

        let ic = self.clone();
        tokio::spawn(async move {
            info!("[Voice] Event processor restarted for guild: {}", guild_id);
            events::runner(ic, evt_rx).await;
            info!("[Voice] Event processor finished for guild: {}", guild_id);
        });

        let ic = self.clone();
        tokio::spawn(async move {
            info!("[Voice] Network processor restarted for guild: {}", guild_id);
            aux_network::runner(ic, pkt_aux_rx).await;
            info!("[Voice] Network processor finished for guild: {}", guild_id);
        });

        // Make mixer aware of new targets...
        let _ = self.mixer.send(MixerMessage::ReplaceInterconnect(self.clone()));
    }
}

pub(crate) struct MixerConnection {
    pub(crate) cipher: Cipher,
    pub(crate) udp: Sender<UdpMessage>,
}

impl Drop for MixerConnection {
    fn drop(&mut self) {
        let _ = self.udp.send(UdpMessage::Poison);
    }
}

pub(crate) enum EventMessage {
    // Event related.
    // Track events should fire off the back of state changes.
    AddGlobalEvent(EventData),
    AddTrackEvent(usize, EventData),
    FireCoreEvent(CoreContext),

    AddTrack(EventStore, TrackState, TrackHandle),
    ChangeState(usize, TrackStateChange),
    RemoveTrack(usize),
    RemoveAllTracks,
    Tick,

    Poison,
}

#[derive(Debug)]
pub(crate) enum TrackStateChange {
    Mode(PlayMode),
    Volume(f32),
    Position(Duration),
    // Bool indicates user-set.
    Loops(LoopState, bool),
    Total(TrackState),
}

pub(crate) enum AuxPacketMessage {
    Udp(RecvHalf),
    UdpCipher(Cipher),
    Ws(Box<WsStream>),

    SetSsrc(u32),
    SetKeepalive(f64),
    Speaking(bool),

    Poison,
}

pub(crate) enum UdpMessage {
    Packet(Vec<u8>), // FIXME: do something cheaper.
    Poison,
}

pub(crate) enum MixerMessage {
    AddTrack(Track),
    SetTrack(Option<Track>),
    SetBitrate(Bitrate),
    SetMute(bool),
    SetConn(MixerConnection, u32),
    DropConn,
    ReplaceInterconnect(Interconnect),
    RebuildEncoder,
    Poison,
}

pub(crate) fn start(guild_id: GuildId, rx: Receiver<Status>, tx: Sender<Status>) {
    tokio::spawn(async move {
        info!("[Voice] Core started for guild: {}", guild_id);
        runner(guild_id, rx, tx).await;
        info!("[Voice] Core finished for guild: {}", guild_id);
    });
}

fn start_internals(guild_id: GuildId, core: Sender<Status>) -> Interconnect {
    let (evt_tx, evt_rx) = flume::unbounded();
    let (pkt_aux_tx, pkt_aux_rx) = flume::unbounded();
    let (mix_tx, mix_rx) = flume::unbounded();

    let interconnect = Interconnect {
        core,
        events: evt_tx,
        aux_packets: pkt_aux_tx,
        mixer: mix_tx,
    };

    let ic = interconnect.clone();
    tokio::spawn(async move {
        info!("[Voice] Event processor started for guild: {}", guild_id);
        events::runner(ic, evt_rx).await;
        info!("[Voice] Event processor finished for guild: {}", guild_id);
    });

    let ic = interconnect.clone();
    tokio::spawn(async move {
        info!("[Voice] Network processor started for guild: {}", guild_id);
        aux_network::runner(ic, pkt_aux_rx).await;
        info!("[Voice] Network processor finished for guild: {}", guild_id);
    });

    let ic = interconnect.clone();
    std::thread::spawn(move || {
        info!("[Voice] Mixer started for guild: {}", guild_id);
        mixer::runner(ic, mix_rx);
        info!("[Voice] Mixer finished for guild: {}", guild_id);
    });

    interconnect
}

async fn runner(guild_id: GuildId, rx: Receiver<Status>, tx: Sender<Status>) {
    let mut connection = None;
    let mut interconnect = start_internals(guild_id, tx);

    loop {
        match rx.recv_async().await {
            Ok(Status::Connect(info)) => {
                connection = match Connection::new(info, &interconnect).await {
                    Ok(connection) => Some(connection),
                    Err(why) => {
                        warn!("[Voice] Error connecting: {:?}", why);

                        None
                    },
                };
            },
            Ok(Status::Disconnect) => {
                connection = None;
                let _ = interconnect.mixer.send(MixerMessage::DropConn);
                let _ = interconnect.mixer.send(MixerMessage::RebuildEncoder);
            },
            Ok(Status::SetTrack(s)) => {
                let _ = interconnect.mixer.send(MixerMessage::SetTrack(s));
            },
            Ok(Status::AddTrack(s)) => {
                let _ = interconnect.mixer.send(MixerMessage::AddTrack(s));
            },
            Ok(Status::SetBitrate(b)) => {
                let _ = interconnect.mixer.send(MixerMessage::SetBitrate(b));
            },
            Ok(Status::AddEvent(evt)) => {
                let _ = interconnect.events.send(EventMessage::AddGlobalEvent(evt));
            }
            Ok(Status::Mute(m)) => {
                let _ = interconnect.mixer.send(MixerMessage::SetMute(m));
            },
            Ok(Status::Reconnect) => {
                if let Some(mut conn) = connection.take() {
                    // try once: if interconnect, try again.
                    // if still issue, full connect.
                    let info = conn.connection_info.clone();

                    let full_connect = match conn.reconnect(&interconnect).await {
                        Ok(()) => {
                            connection = Some(conn);
                            false
                        },
                        Err(ConnectionError::InterconnectFailure(_)) => {
                            interconnect.restart_volatile_internals(guild_id);

                            match conn.reconnect(&interconnect).await {
                                Ok(()) => {
                                    connection = Some(conn);
                                    false
                                },
                                _ => true,
                            }
                        },
                        _ => true,
                    };

                    if full_connect {
                        connection = Connection::new(info, &interconnect).await
                            .map_err(|e| {error!("[Voice] Catastrophic connection failure. Stopping. {:?}", e); e})
                            .ok();
                    }
                }
            },
            Ok(Status::RebuildInterconnect) => {
                interconnect.restart_volatile_internals(guild_id);
            },
            Err(RecvError::Disconnected) | Ok(Status::Poison) => {
                break;
            },
        }
    }

    info!("[Voice] Main thread exited");
    interconnect.poison_all();
}
