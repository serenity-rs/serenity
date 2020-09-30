use async_trait::async_trait;
use crate::gateway::InterMessage;
use std::collections::HashMap;
use futures::channel::mpsc::UnboundedSender as Sender;
use crate::model::{
    id::{ChannelId, GuildId, UserId},
    voice::VoiceState,
};

/// FIXME: must be documented.
/// All these take read refs to allow struct to use its own concurrency access mechanisms.
#[async_trait]
pub trait VoiceGatewayManager: Send + Sync {
    async fn initialise(&self, shard_count: u64, user_id: UserId);

    async fn register_shard(&self, shard_id: u64, sender: Sender<InterMessage>);

    async fn deregister_shard(&self, shard_id: u64);

    async fn server_update(&self, guild_id: GuildId, endpoint: &Option<String>, token: &String);

    async fn state_update(&self, guild_id: GuildId, voice_state: &VoiceState);
}
