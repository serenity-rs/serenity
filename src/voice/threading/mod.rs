mod aux_network;
mod events;

use crate::{
    error::Error,
    gateway::WsStream,
    internal::Timer,
    model::id::GuildId,
    voice::{
        connection::Connection,
        constants::*,
        events::{
            CoreContext,
            EventData,
            EventStore,
        },
        tracks::{
            LoopState,
            PlayMode,
            TrackHandle,
            TrackState,
        },
        Status,
    },
};
use log::{error, info, warn};
use std::{
    net::{
        SocketAddr,
    },
    thread::Builder as ThreadBuilder,
    time::Duration,
};
use tokio::{
    time::{delay_for, timeout},
    net::{
        UdpSocket,
        udp::{RecvHalf, SendHalf},
    },
    sync::mpsc::{
        error::{
            SendError,
            TryRecvError,
        },
        self,
        UnboundedReceiver,
        UnboundedSender,
    },
};
use xsalsa20poly1305::XSalsa20Poly1305 as Cipher;

#[derive(Clone, Debug)]
pub(crate) struct Interconnect {
    pub(crate) core: UnboundedSender<CoreMessage>,
    pub(crate) events: UnboundedSender<EventMessage>,
    pub(crate) aux_packets: UnboundedSender<AuxPacketMessage>,
}

impl Interconnect {
    fn poison(&self) {
        let _ = self.events.send(EventMessage::Poison);
        let _ = self.aux_packets.send(AuxPacketMessage::Poison);
    }

    fn restart(self, guild_id: GuildId) -> Self {
        self.poison();
        start_internals(guild_id, self.core)
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
    UdpDestination(SocketAddr),
    UdpCipher(Cipher),
    Ws(Box<WsStream>),

    SetSsrc(u32),
    SetKeepalive(f64),
    Speaking(bool),

    Poison,
}

pub(crate) enum CoreMessage {
    Reconnect,
}

pub(crate) fn start(guild_id: GuildId, rx: UnboundedReceiver<Status>) {
    tokio::spawn(async move {
        info!("[Voice] Core started for guild: {}", guild_id);
        runner(guild_id, rx).await;
        info!("[Voice] Core finished for guild: {}", guild_id);
    });
}

fn start_internals(guild_id: GuildId, core: UnboundedSender<CoreMessage>) -> Interconnect {
    let (evt_tx, evt_rx) = mpsc::unbounded_channel();
    let (pkt_aux_tx, pkt_aux_rx) = mpsc::unbounded_channel();

    let interconnect = Interconnect {
        core,
        events: evt_tx,
        aux_packets: pkt_aux_tx,
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

    interconnect
}

async fn runner(guild_id: GuildId, mut rx: UnboundedReceiver<Status>) {
    let mut tracks = Vec::new();
    let mut connection = None;
    let mut timer = Timer::new(20);
    let mut bitrate = DEFAULT_BITRATE;
    let mut mute = false;

    let (reconnect_tx, mut reconnect_rx) = mpsc::unbounded_channel();

    let mut interconnect = start_internals(guild_id, reconnect_tx);

    'runner: loop {
        loop {
            // info!("lop");
            match rx.try_recv() {
                Ok(Status::Connect(info)) => {
                    connection = match Connection::new(info, &interconnect, bitrate).await {
                        Ok(connection) => Some(connection),
                        Err(why) => {
                            warn!("[Voice] Error connecting: {:?}", why);

                            None
                        },
                    };
                },
                Ok(Status::Disconnect) => {
                    connection = None;
                },
                Ok(Status::SetTrack(s)) => {
                    tracks.clear();

                    if let Some(aud) = s {
                        tracks.push(aud);
                    }
                },
                Ok(Status::AddTrack(mut s)) => {
                    let evts = s.events.take()
                        .unwrap_or_default();
                    let state = s.state();
                    let handle = s.handle.clone();

                    tracks.push(s);

                    let _ = interconnect.events.send(EventMessage::AddTrack(evts, state, handle));
                },
                Ok(Status::SetBitrate(b)) => {
                    bitrate = b;
                    if let Some(conn) = connection.as_mut() {
                        if let Err(e) = conn.set_bitrate(b) {
                            warn!("[Voice] Bitrate set unsuccessfully: {:?}", e);
                        }
                    }
                },
                Ok(Status::AddEvent(evt)) => {
                    let _ = interconnect.events.send(EventMessage::AddGlobalEvent(evt));
                }
                Ok(Status::Mute(m)) => {
                    mute = m;
                },
                Err(TryRecvError::Empty) => {
                    // If we received nothing, then we can perform an update.
                    // info!("Actually broke");
                    break;
                },
                Err(TryRecvError::Closed) => {
                    break 'runner;
                },
            }
        }

        // info!("Huh");

        // Overall here, check if there's an error.
        //
        // If there is a connection, try to send an update. This should not
        // error. If there is though for some spurious reason, then set `error`
        // to `true`.
        //
        // Otherwise, wait out the timer and do _not_ error and wait to receive
        // another event.
        let error = match connection.as_mut() {
            Some(connection) => {
                let cycle = connection.cycle(&mut tracks, &mut timer, bitrate, &interconnect, mute).await;

                match cycle {
                    Ok(()) => {
                        // Send state changes
                        let mut i = 0;
                        let mut to_remove = Vec::with_capacity(tracks.len());
                        while i < tracks.len() {
                            let aud = tracks.get_mut(i)
                                .expect("[Voice] Tried to remove an illegal track index.");

                            if aud.playing.is_done() {
                                let p_state = aud.playing();
                                tracks.remove(i);
                                to_remove.push(i);
                                let _ = interconnect.events.send(EventMessage::ChangeState(i, TrackStateChange::Mode(p_state)));
                            } else {
                                i += 1;
                            }
                        }

                        // Tick
                        let _ = interconnect.events.send(EventMessage::Tick);

                        // Then do removals.
                        for i in &to_remove[..] {
                            let _ = interconnect.events.send(EventMessage::RemoveTrack(*i));
                        }

                        false
                    },
                    Err(why) => {
                        if matches!(why, Error::VoiceInterconnectFailure) {
                            interconnect = interconnect.restart(guild_id);
                        }

                        error!(
                            "(╯°□°）╯︵ ┻━┻ Error updating connection: {:?}",
                            why
                        );

                        true
                    },
                }
            },
            None => {
                timer.hold().await;

                false
            },
        };

        let remote_error = match reconnect_rx.try_recv() {
            Err(TryRecvError::Empty) => {
                false
            },
            _ => {
                true
            },
        };

        // If there was an error, then just reset the connection and try to get
        // another.
        if error || remote_error {
            if let Some(mut conn) = connection.take() {
                // try once: if interconnect, try again.
                // if still issue, full connect.
                let info = conn.connection_info.clone();

                let full_connect = match conn.reconnect(&interconnect).await {
                    Ok(()) => {
                        connection = Some(conn);
                        false
                    },
                    Err(Error::VoiceInterconnectFailure) => {
                        interconnect = interconnect.restart(guild_id);

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
                    connection = Connection::new(info, &interconnect, bitrate).await
                        .map_err(|e| {error!("[Voice] Catastrophic connection failure. Stopping. {:?}", e); e})
                        .ok();
                }
            }
        }
    }

    info!("[Voice] Main thread exited");
    interconnect.poison();
}
