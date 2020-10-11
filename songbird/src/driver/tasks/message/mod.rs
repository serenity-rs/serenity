use crate::model::id::GuildId;
use flume::Sender;

mod aux_network;
mod core;
mod events;
mod mixer;
mod udp;

pub(crate) use self::{aux_network::*, core::*, events::*, mixer::*, udp::*};

use tracing::info;

#[derive(Clone, Debug)]
pub(crate) struct Interconnect {
    pub core: Sender<CoreMessage>,
    pub events: Sender<EventMessage>,
    pub aux_packets: Sender<AuxPacketMessage>,
    pub mixer: Sender<MixerMessage>,
}

impl Interconnect {
    pub fn poison(&self) {
        let _ = self.events.send(EventMessage::Poison);
        let _ = self.aux_packets.send(AuxPacketMessage::Poison);
    }

    pub fn poison_all(&self) {
        self.poison();
        let _ = self.mixer.send(MixerMessage::Poison);
    }

    pub fn restart(self, guild_id: GuildId) -> Self {
        self.poison();
        super::start_internals(guild_id, self.core)
    }

    pub fn restart_volatile_internals(&mut self, guild_id: GuildId) {
        self.poison();

        let (evt_tx, evt_rx) = flume::unbounded();
        let (pkt_aux_tx, pkt_aux_rx) = flume::unbounded();

        self.events = evt_tx;
        self.aux_packets = pkt_aux_tx;

        let ic = self.clone();
        tokio::spawn(async move {
            info!("[Voice] Event processor restarted for guild: {}", guild_id);
            super::events::runner(ic, evt_rx).await;
            info!("[Voice] Event processor finished for guild: {}", guild_id);
        });

        let ic = self.clone();
        tokio::spawn(async move {
            info!(
                "[Voice] Network processor restarted for guild: {}",
                guild_id
            );
            super::aux_network::runner(ic, pkt_aux_rx).await;
            info!("[Voice] Network processor finished for guild: {}", guild_id);
        });

        // Make mixer aware of new targets...
        let _ = self
            .mixer
            .send(MixerMessage::ReplaceInterconnect(self.clone()));
    }
}
