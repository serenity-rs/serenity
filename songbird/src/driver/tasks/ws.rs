use super::{error::Result, message::*};
use crate::{
    events::CoreContext,
    model::{
        payload::{Heartbeat, Speaking},
        Event as GatewayEvent,
        SpeakingState,
    },
    ws::{Error as WsError, ReceiverExt, SenderExt, WsStream},
};
use flume::Receiver;
use rand::random;
use std::time::Duration;
use tokio::time::{self, Instant};
use tracing::{error, info, instrument, trace, warn};

struct AuxNetwork {
    rx: Receiver<WsMessage>,
    ws_client: WsStream,
    dont_send: bool,

    ssrc: u32,
    heartbeat_interval: Duration,

    speaking: SpeakingState,
    last_heartbeat_nonce: Option<u64>,
}

impl AuxNetwork {
    pub(crate) fn new(
        evt_rx: Receiver<WsMessage>,
        ws_client: WsStream,
        ssrc: u32,
        heartbeat_interval: f64,
    ) -> Self {
        Self {
            rx: evt_rx,
            ws_client,
            dont_send: false,

            ssrc,
            heartbeat_interval: Duration::from_secs_f64(heartbeat_interval / 1000.0),

            speaking: SpeakingState::empty(),
            last_heartbeat_nonce: None,
        }
    }

    #[instrument(skip(self))]
    async fn run(&mut self, interconnect: &mut Interconnect) {
        let mut next_heartbeat = Instant::now() + self.heartbeat_interval;

        loop {
            let mut ws_error = false;

            let hb = time::delay_until(next_heartbeat);

            tokio::select! {
                _ = hb => {
                    ws_error = match self.send_heartbeat().await {
                        Err(e) => {
                            error!("Heartbeat send failure {:?}.", e);
                            true
                        },
                        _ => false,
                    };
                    next_heartbeat = self.next_heartbeat();
                }
                ws_msg = self.ws_client.recv_json_no_timeout(), if !self.dont_send => {
                    ws_error = match ws_msg {
                        Err(WsError::Json(e)) => {
                            warn!("Unexpected JSON {:?}.", e);
                            false
                        },
                        Err(e) => {
                            error!("Error processing ws {:?}.", e);
                            true
                        },
                        Ok(Some(msg)) => {
                            self.process_ws(interconnect, msg);
                            false
                        },
                        _ => false,
                    };
                }
                inner_msg = self.rx.recv_async() => {
                    match inner_msg {
                        Ok(WsMessage::Ws(data)) => {
                            self.ws_client = *data;
                            next_heartbeat = self.next_heartbeat();
                            self.dont_send = false;
                        },
                        Ok(WsMessage::ReplaceInterconnect(i)) => {
                            *interconnect = i;
                        },
                        Ok(WsMessage::SetKeepalive(keepalive)) => {
                            self.heartbeat_interval = Duration::from_secs_f64(keepalive / 1000.0);
                            next_heartbeat = self.next_heartbeat();
                        },
                        Ok(WsMessage::Speaking(is_speaking)) => {
                            if self.speaking.contains(SpeakingState::MICROPHONE) != is_speaking && !self.dont_send {
                                self.speaking.set(SpeakingState::MICROPHONE, is_speaking);
                                info!("Changing to {:?}", self.speaking);

                                let ssu_status = self.ws_client
                                    .send_json(&GatewayEvent::from(Speaking {
                                        delay: Some(0),
                                        speaking: self.speaking,
                                        ssrc: self.ssrc,
                                        user_id: None,
                                    }))
                                    .await;

                                ws_error |= match ssu_status {
                                    Err(e) => {
                                        error!("Issue sending speaking update {:?}.", e);
                                        true
                                    },
                                    _ => false,
                                }
                            }
                        },
                        Err(_) | Ok(WsMessage::Poison) => {
                            break;
                        },
                    }
                }
            }

            if ws_error {
                let _ = interconnect.core.send(CoreMessage::Reconnect);
                self.dont_send = true;
            }
        }
    }

    fn next_heartbeat(&self) -> Instant {
        Instant::now() + self.heartbeat_interval
    }

    async fn send_heartbeat(&mut self) -> Result<()> {
        let nonce = random::<u64>();
        self.last_heartbeat_nonce = Some(nonce);

        trace!("Sent heartbeat {:?}", self.speaking);

        if !self.dont_send {
            self.ws_client
                .send_json(&GatewayEvent::from(Heartbeat { nonce }))
                .await?;
        }

        Ok(())
    }

    fn process_ws(&mut self, interconnect: &Interconnect, value: GatewayEvent) {
        match value {
            GatewayEvent::Speaking(ev) => {
                let _ = interconnect.events.send(EventMessage::FireCoreEvent(
                    CoreContext::SpeakingStateUpdate(ev),
                ));
            },
            GatewayEvent::ClientConnect(ev) => {
                let _ = interconnect
                    .events
                    .send(EventMessage::FireCoreEvent(CoreContext::ClientConnect(ev)));
            },
            GatewayEvent::ClientDisconnect(ev) => {
                let _ = interconnect.events.send(EventMessage::FireCoreEvent(
                    CoreContext::ClientDisconnect(ev),
                ));
            },
            GatewayEvent::HeartbeatAck(ev) => {
                if let Some(nonce) = self.last_heartbeat_nonce.take() {
                    if ev.nonce == nonce {
                        trace!("Heartbeat ACK received.");
                    } else {
                        warn!(
                            "Heartbeat nonce mismatch! Expected {}, saw {}.",
                            nonce, ev.nonce
                        );
                    }
                }
            },
            other => {
                trace!("Received other websocket data: {:?}", other);
            },
        }
    }
}

#[instrument(skip(interconnect, ws_client))]
pub(crate) async fn runner(
    mut interconnect: Interconnect,
    evt_rx: Receiver<WsMessage>,
    ws_client: WsStream,
    ssrc: u32,
    heartbeat_interval: f64,
) {
    info!("WS thread started.");
    let mut aux = AuxNetwork::new(evt_rx, ws_client, ssrc, heartbeat_interval);

    aux.run(&mut interconnect).await;
    info!("WS thread finished.");
}
