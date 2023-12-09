use std::num::NonZeroU16;

use async_trait::async_trait;
use futures::channel::mpsc::UnboundedSender as Sender;

use crate::gateway::ShardRunnerMessage;
use crate::model::id::{GuildId, UserId};
use crate::model::voice::VoiceState;

/// Interface for any compatible voice plugin.
///
/// This interface covers several serenity-specific hooks, as well as packet handlers for
/// voice-specific gateway messages.
#[async_trait]
pub trait VoiceGatewayManager: Send + Sync {
    /// Performs initial setup at the start of a connection to Discord.
    ///
    /// This will only occur once, and provides the bot's ID and shard count.
    async fn initialise(&self, shard_count: NonZeroU16, user_id: UserId);

    /// Handler fired in response to a [`Ready`] event.
    ///
    /// This provides the voice plugin with a channel to send gateway messages to Discord, once per
    /// active shard.
    ///
    /// [`Ready`]: crate::model::event::Event
    async fn register_shard(&self, shard_id: u16, sender: Sender<ShardRunnerMessage>);

    /// Handler fired in response to a disconnect, reconnection, or rebalance.
    ///
    /// This event invalidates the last sender associated with `shard_id`. Unless the bot is fully
    /// disconnecting, this is often followed by a call to [`Self::register_shard`]. Users may wish
    /// to buffer manually any gateway messages sent between these calls.
    async fn deregister_shard(&self, shard_id: u16);

    /// Handler for VOICE_SERVER_UPDATE messages.
    ///
    /// These contain the endpoint and token needed to form a voice connection session.
    async fn server_update(&self, guild_id: GuildId, endpoint: Option<&str>, token: &str);

    /// Handler for VOICE_STATE_UPDATE messages.
    ///
    /// These contain the session ID needed to form a voice connection session.
    async fn state_update(&self, guild_id: GuildId, voice_state: &VoiceState);
}
