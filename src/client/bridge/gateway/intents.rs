use bitflags::__impl_bitflags;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

/// [Gateway Intents] will limit the events your bot will receive via the gateway.
/// By default, no intents are specified by Serenity.
///
/// [Gateway Intents]: https://discordapp.com/developers/docs/topics/gateway#gateway-intents
#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
pub struct GatewayIntents {
    /// The flags composing gateway intents.
    ///
    /// # Note
    /// Do not modify this yourself; use the provided methods.
    /// Do the same when creating, unless you're absolutely certain that you're giving valid intents flags.
    pub bits: u64
}

__impl_bitflags! {
    GatewayIntents: u64 {
        /// Enables following gateway events:
        ///
        /// - GUILD_CREATE
        /// - GUILD_DELETE
        /// - GUILD_ROLE_CREATE
        /// - GUILD_ROLE_UPDATE
        /// - GUILD_ROLE_DELETE
        /// - CHANNEL_CREATE
        /// - CHANNEL_UPDATE
        /// - CHANNEL_DELETE
        /// - CHANNEL_PINS_UPDATE
        GUILDS = 1;
        /// Enables following gateway events:
        ///
        /// - GUILD_MEMBER_ADD
        /// - GUILD_MEMBER_UPDATE
        /// - GUILD_MEMBER_REMOVE
        ///
        /// **Info**:
        /// This intent is *privileged*.
        /// In order to use it, you must head to your application in the
        /// Developer Portal and enable the toggle for *Privileged Intents*.
        GUILD_MEMBERS = 1 << 1;
        /// Enables following gateway events:
        ///
        /// - GUILD_BAN_ADD
        /// - GUILD_BAN_REMOVE
        GUILD_BANS = 1 << 2;
        /// Enables following gateway event:
        ///
        /// - GUILD_EMOJIS_UPDATE
        GUILD_EMOJIS = 1 << 3;
        /// Enables following gateway event:
        ///
        /// - GUILD_INTEGRATIONS_UPDATE
        GUILD_INTEGRATIONS = 1 << 4;
        /// Enables following gateway event:
        ///
        /// - WEBHOOKS_UPDATE
        GUILD_WEBHOOKS = 1 << 5;
        /// Enables following gateway events:
        ///
        /// - INVITE_CREATE
        /// - INVITE_DELETE
        GUILD_INVITES = 1 << 6;
        /// Enables following gateway event:
        ///
        /// - VOICE_STATE_UPDATE
        GUILD_VOICE_STATES = 1 << 7;
        /// Enables following gateway event:
        ///
        /// - PRESENCE_UPDATE
        ///
        /// **Info**:
        /// This intent is *privileged*.
        /// In order to use it, you must head to your application in the
        /// Developer Portal and enable the toggle for *Privileged Intents*.
        GUILD_PRESENCES = 1 << 8;
        /// Enables following gateway events:
        ///
        /// - MESSAGE_CREATE
        /// - MESSAGE_UPDATE
        /// - MESSAGE_DELETE
        GUILD_MESSAGES = 1 << 9;
        /// Enables following gateway events:
        ///
        /// - MESSAGE_REACTION_ADD
        /// - MESSAGE_REACTION_REMOVE
        /// - MESSAGE_REACTION_REMOVE_ALL
        /// - MESSAGE_REACTION_REMOVE_EMOJI
        GUILD_MESSAGE_REACTIONS = 1 << 10;
        /// Enable following gateway event:
        ///
        /// - TYPING_START
        GUILD_MESSAGE_TYPING = 1 << 11;
        /// Enable following gateway events:
        ///
        /// - CHANNEL_CREATE
        /// - MESSAGE_CREATE
        /// - MESSAGE_UPDATE
        /// - MESSAGE_DELETE
        /// - CHANNEL_PINS_UPDATE
        DIRECT_MESSAGES = 1 << 12;
        /// Enable following gateway events:
        ///
        /// - MESSAGE_REACTION_ADD
        /// - MESSAGE_REACTION_REMOVE
        /// - MESSAGE_REACTION_REMOVE_ALL
        /// - MESSAGE_REACTION_REMOVE_EMOJI
        DIRECT_MESSAGE_REACTIONS = 1 << 13;
        /// Enable following gateway event:
        ///
        /// - TYPING_START
        DIRECT_MESSAGE_TYPING = 1 << 14;
    }
}

impl<'de> Deserialize<'de> for GatewayIntents {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_bits_truncate(u64::deserialize(deserializer)?))
    }
}

impl Serialize for GatewayIntents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.bits())
    }
}
