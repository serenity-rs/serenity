mod aux_network;
pub mod error;
mod events;
pub(crate) mod message;
mod mixer;

use crate::{
    driver::connection::{error::Error as ConnectionError, Connection},
    model::id::GuildId,
};
use flume::{
    Receiver,
    Sender,
    RecvError,
};
use message::*;
use tracing::{error, info, warn};

pub(crate) fn start(guild_id: GuildId, rx: Receiver<CoreMessage>, tx: Sender<CoreMessage>) {
    tokio::spawn(async move {
        info!("[Voice] Core started for guild: {}", guild_id);
        runner(guild_id, rx, tx).await;
        info!("[Voice] Core finished for guild: {}", guild_id);
    });
}

fn start_internals(guild_id: GuildId, core: Sender<CoreMessage>) -> Interconnect {
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

async fn runner(guild_id: GuildId, rx: Receiver<CoreMessage>, tx: Sender<CoreMessage>) {
    let mut connection = None;
    let mut interconnect = start_internals(guild_id, tx);

    loop {
        match rx.recv_async().await {
            Ok(CoreMessage::Connect(info)) => {
                connection = match Connection::new(info, &interconnect).await {
                    Ok(connection) => Some(connection),
                    Err(why) => {
                        warn!("[Voice] Error connecting: {:?}", why);

                        None
                    },
                };
            },
            Ok(CoreMessage::Disconnect) => {
                connection = None;
                let _ = interconnect.mixer.send(MixerMessage::DropConn);
                let _ = interconnect.mixer.send(MixerMessage::RebuildEncoder);
            },
            Ok(CoreMessage::SetTrack(s)) => {
                let _ = interconnect.mixer.send(MixerMessage::SetTrack(s));
            },
            Ok(CoreMessage::AddTrack(s)) => {
                let _ = interconnect.mixer.send(MixerMessage::AddTrack(s));
            },
            Ok(CoreMessage::SetBitrate(b)) => {
                let _ = interconnect.mixer.send(MixerMessage::SetBitrate(b));
            },
            Ok(CoreMessage::AddEvent(evt)) => {
                let _ = interconnect.events.send(EventMessage::AddGlobalEvent(evt));
            }
            Ok(CoreMessage::Mute(m)) => {
                let _ = interconnect.mixer.send(MixerMessage::SetMute(m));
            },
            Ok(CoreMessage::Reconnect) => {
                if let Some(mut conn) = connection.take() {
                    // try once: if interconnect, try again.
                    // if still issue, full connect.
                    let info = conn.info.clone();

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
            Ok(CoreMessage::RebuildInterconnect) => {
                interconnect.restart_volatile_internals(guild_id);
            },
            Err(RecvError::Disconnected) | Ok(CoreMessage::Poison) => {
                break;
            },
        }
    }

    info!("[Voice] Main thread exited");
    interconnect.poison_all();
}
