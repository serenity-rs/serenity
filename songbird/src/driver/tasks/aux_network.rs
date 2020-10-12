use super::{
    error::{Error, Result},
    message::*,
};
use crate::{
    constants::*,
    events::CoreContext,
    model::{
        payload::{Heartbeat, Speaking},
        Event as GatewayEvent,
        SpeakingState,
    },
    timer::Timer,
    ws::{ReceiverExt, SenderExt, WsStream},
};
use audiopus::{coder::Decoder as OpusDecoder, Channels};
use discortp::{
    demux::{self, DemuxedMut},
    discord::MutableKeepalivePacket,
    rtp::{RtpExtensionPacket, RtpPacket},
    FromPacket,
    MutablePacket,
    Packet,
    PacketSize,
};
use flume::{Receiver, TryRecvError};
use rand::random;
use std::collections::HashMap;
use tokio::net::udp::RecvHalf;
use tracing::{error, info, trace, warn};
use xsalsa20poly1305::{aead::AeadInPlace, Nonce, Tag, XSalsa20Poly1305 as Cipher, TAG_SIZE};

struct AuxNetwork {
    rx: Receiver<AuxPacketMessage>,
    ws_client: Option<WsStream>,

    ssrc: u32,
    ws_keepalive_time: Timer,

    speaking: SpeakingState,
    last_heartbeat_nonce: Option<u64>,
}

impl AuxNetwork {
    pub(crate) fn new(evt_rx: Receiver<AuxPacketMessage>) -> Self {
        Self {
            rx: evt_rx,
            ws_client: None,

            ssrc: 0,
            ws_keepalive_time: Timer::new(45_000),

            speaking: SpeakingState::empty(),
            last_heartbeat_nonce: None,
        }
    }

    async fn run(&mut self, interconnect: &Interconnect) {
        'aux_runner: loop {
            let mut ws_error = match self.process_ws_messages(interconnect).await {
                Err(e) => {
                    error!("Error processing ws {:?}.", e);
                    true
                },
                _ => false,
            };

            tokio::time::delay_for(TIMESTEP_LENGTH / 2).await;

            loop {
                match self.rx.try_recv() {
                    Ok(AuxPacketMessage::Ws(data)) => {
                        self.ws_client = Some(*data);
                        self.ws_keepalive_time.reset();
                    },
                    Ok(AuxPacketMessage::SetSsrc(new_ssrc)) => {
                        self.ssrc = new_ssrc;
                    },
                    Ok(AuxPacketMessage::SetKeepalive(keepalive)) => {
                        self.ws_keepalive_time = Timer::new(keepalive as u64);
                    },
                    Ok(AuxPacketMessage::Speaking(is_speaking)) => {
                        if self.speaking.contains(SpeakingState::MICROPHONE) != is_speaking {
                            self.speaking.set(SpeakingState::MICROPHONE, is_speaking);
                            if let Some(client) = self.ws_client.as_mut() {
                                info!("Changing to {:?}", self.speaking);

                                let ssu_status = client
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
                        }
                    },
                    Err(TryRecvError::Disconnected) | Ok(AuxPacketMessage::Poison) => {
                        break 'aux_runner;
                    },
                    Err(_) => {
                        // No message.
                        break;
                    },
                }
            }

            if ws_error {
                self.ws_client = None;
                let _ = interconnect.core.send(CoreMessage::Reconnect);
            }
        }

        info!("Auxiliary network thread exited");
    }

    async fn process_ws_messages(&mut self, interconnect: &Interconnect) -> Result<()> {
        if let Some(ws) = self.ws_client.as_mut() {
            if self.ws_keepalive_time.check() {
                let nonce = random::<u64>();
                self.last_heartbeat_nonce = Some(nonce);
                trace!("[Aux] Sent heartbeat {:?}", self.speaking);
                ws.send_json(&GatewayEvent::from(Heartbeat { nonce }))
                    .await?;
                self.ws_keepalive_time.increment();
            }

            // FIXME: need to propagate WS disconnection back to main thread to trigger reconnect.
            // FIXME: makw this one big grand select

            while let Ok(Ok(Some(value))) =
                tokio::time::timeout(TIMESTEP_LENGTH / 2, ws.try_recv_json()).await
            {
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
                                info!("Heartbeat ACK received.");
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
        Ok(())
    }
}

pub(crate) async fn runner(interconnect: Interconnect, evt_rx: Receiver<AuxPacketMessage>) {
    let mut aux = AuxNetwork::new(evt_rx);

    aux.run(&interconnect).await;
}
