pub mod error;
mod events;
pub(crate) mod message;
mod mixer;
pub(crate) mod udp_rx;
pub(crate) mod udp_tx;
pub(crate) mod ws;

use super::{
    connection::{error::Error as ConnectionError, Connection},
    Config,
};
use flume::{Receiver, RecvError, Sender};
use message::*;
use tokio::runtime::Handle;
use tracing::{error, info, instrument};

pub(crate) fn start(config: Config, rx: Receiver<CoreMessage>, tx: Sender<CoreMessage>) {
    tokio::spawn(async move {
        info!("Driver started.");
        runner(config, rx, tx).await;
        info!("Driver finished.");
    });
}

fn start_internals(core: Sender<CoreMessage>) -> Interconnect {
    let (evt_tx, evt_rx) = flume::unbounded();
    let (mix_tx, mix_rx) = flume::unbounded();

    let interconnect = Interconnect {
        core,
        events: evt_tx,
        mixer: mix_tx,
    };

    let ic = interconnect.clone();
    tokio::spawn(async move {
        info!("Event processor started.");
        events::runner(ic, evt_rx).await;
        info!("Event processor finished.");
    });

    let ic = interconnect.clone();
    let handle = Handle::current();
    std::thread::spawn(move || {
        info!("Mixer started.");
        mixer::runner(ic, mix_rx, handle);
        info!("Mixer finished.");
    });

    interconnect
}

#[instrument(skip(rx, tx))]
async fn runner(config: Config, rx: Receiver<CoreMessage>, tx: Sender<CoreMessage>) {
    let mut connection = None;
    let mut interconnect = start_internals(tx);

    loop {
        match rx.recv_async().await {
            Ok(CoreMessage::ConnectWithResult(info, tx)) => {
                connection = match Connection::new(info, &interconnect, &config).await {
                    Ok(connection) => {
                        // Other side may not be listening: this is fine.
                        let _ = tx.send(Ok(()));
                        Some(connection)
                    },
                    Err(why) => {
                        // See above.
                        let _ = tx.send(Err(why));

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
            },
            Ok(CoreMessage::Mute(m)) => {
                let _ = interconnect.mixer.send(MixerMessage::SetMute(m));
            },
            Ok(CoreMessage::Reconnect) => {
                if let Some(mut conn) = connection.take() {
                    // try once: if interconnect, try again.
                    // if still issue, full connect.
                    let info = conn.info.clone();

                    let full_connect = match conn.reconnect().await {
                        Ok(()) => {
                            connection = Some(conn);
                            false
                        },
                        Err(ConnectionError::InterconnectFailure(_)) => {
                            interconnect.restart_volatile_internals();

                            match conn.reconnect().await {
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
                        connection = Connection::new(info, &interconnect, &config)
                            .await
                            .map_err(|e| {
                                error!("Catastrophic connection failure. Stopping. {:?}", e);
                                e
                            })
                            .ok();
                    }
                }
            },
            Ok(CoreMessage::FullReconnect) =>
                if let Some(conn) = connection.take() {
                    let info = conn.info.clone();

                    connection = Connection::new(info, &interconnect, &config)
                        .await
                        .map_err(|e| {
                            error!("Catastrophic connection failure. Stopping. {:?}", e);
                            e
                        })
                        .ok();
                },
            Ok(CoreMessage::RebuildInterconnect) => {
                interconnect.restart_volatile_internals();
            },
            Err(RecvError::Disconnected) | Ok(CoreMessage::Poison) => {
                break;
            },
        }
    }

    info!("Main thread exited");
    interconnect.poison_all();
}
