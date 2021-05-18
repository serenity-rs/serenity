use bitflags::__impl_bitflags;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

/// [Gateway Intents] will limit the events your bot will receive via the gateway.
/// By default, all intents except [Privileged Intents] are selected.
///
/// # What are Intents
///
/// A [gateway intent] sets the types of gateway events
/// (e.g. member joins, guild integrations, guild emoji updates, ...) the
/// bot shall receive. Carefully picking the needed intents greatly helps
/// the bot to scale, as less intents will result in less events to be
/// received via the network from Discord and less processing needed for
/// handling the data.
///
/// # Privileged Intents
///
/// The intents [`GatewayIntents::GUILD_PRESENCES`] and [`GatewayIntents::GUILD_MEMBERS`]
/// are [Privileged Intents]. They need to be enabled in the
/// *developer portal*.
///
/// **Note**:
/// Once the bot is in 100 guilds or more, [the bot must be verified] in
/// order to use privileged intents.
///
/// [gateway intent]: https://discord.com/developers/docs/topics/gateway#privileged-intents
/// [Privileged Intents]: https://discord.com/developers/docs/topics/gateway#privileged-intents
/// [the bot must be verified]: https://support.discord.com/hc/en-us/articles/360040720412-Bot-Verification-and-Data-Whitelisting
/// [`GatewayIntents::GuildPresences`]: serenity::client::bridge::gateway::GatewayIntents::GUILD_PRESENCES
/// [`GatewayIntents::GuildMembers`]: serenity::client::bridge::gateway::GatewayIntents::GUILD_MEMBERS
#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
pub struct GatewayIntents {
    /// The flags composing gateway intents.
    ///
    /// # Note
    /// Do not modify this yourself; use the provided methods.
    /// Do the same when creating, unless you're absolutely certain that you're giving valid intents flags.
    pub bits: u64,
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
        ///
        /// This intent is also necessary to even receive the events in contains.
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
        ///
        /// This intent is also necessary to even receive the events in contains.
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

impl Default for GatewayIntents {
    fn default() -> Self {
        Self::empty()
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

#[cfg(feature = "model")]
impl GatewayIntents {
    /// Gets all of the intents that don't are considered privileged by Discord.
    pub const fn non_privileged() -> GatewayIntents {
        // bitflags don't support const evaluation. Workaround.
        // See: https://github.com/bitflags/bitflags/issues/180
        Self::from_bits_truncate(Self::all().bits() & !Self::privileged().bits())
    }

    /// Gets all of the intents that are considered privileged by Discord.
    /// Use of these intents will require explicitly whitelisting the bot.
    pub const fn privileged() -> GatewayIntents {
        // bitflags don't support const evaluation. Workaround.
        // See: https://github.com/bitflags/bitflags/issues/180
        Self::from_bits_truncate(Self::GUILD_MEMBERS.bits() | Self::GUILD_PRESENCES.bits())
    }

    /// Checks if any of the included intents are privileged
    ///
    /// [GUILD_MEMBERS]: #associatedconstant.GUILD_MEMBERS
    /// [GUILD_PRESENCES]: #associatedconstant.GUILD_PRESENCES
    pub fn is_privileged(self) -> bool {
        self.guild_members() || self.guild_presences()
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILDS] intent.
    ///
    /// [GUILDS]: Self::GUILDS
    pub fn guilds(self) -> bool {
        self.contains(Self::GUILDS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_MEMBERS] intent.
    ///
    /// [GUILD_MEMBERS]: Self::GUILD_MEMBERS
    pub fn guild_members(self) -> bool {
        self.contains(Self::GUILD_MEMBERS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_BANS] intent.
    ///
    /// [GUILD_BANS]: Self::GUILD_BANS
    pub fn guild_bans(self) -> bool {
        self.contains(Self::GUILD_BANS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_EMOJIS] intent.
    ///
    /// [GUILD_EMOJIS]: Self::GUILD_EMOJIS
    pub fn guild_emojis(self) -> bool {
        self.contains(Self::GUILD_EMOJIS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_INTEGRATIONS] intent.
    ///
    /// [GUILD_INTEGRATIONS]: Self::GUILD_INTEGRATIONS
    pub fn guild_integrations(self) -> bool {
        self.contains(Self::GUILD_INTEGRATIONS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_WEBHOOKS] intent.
    ///
    /// [GUILD_WEBHOOKS]: Self::GUILD_WEBHOOKS
    pub fn guild_webhooks(self) -> bool {
        self.contains(Self::GUILD_WEBHOOKS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_INVITES] intent.
    ///
    /// [GUILD_INVITES]: Self::GUILD_INVITES
    pub fn guild_invites(self) -> bool {
        self.contains(Self::GUILD_INVITES)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_VOICE_STATES] intent.
    ///
    /// [GUILD_VOICE_STATES]: Self::GUILD_VOICE_STATES
    pub fn guild_voice_states(self) -> bool {
        self.contains(Self::GUILD_VOICE_STATES)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_PRESENCES] intent.
    ///
    /// [GUILD_PRESENCES]: Self::GUILD_PRESENCES
    pub fn guild_presences(self) -> bool {
        self.contains(Self::GUILD_PRESENCES)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_MESSAGE_REACTIONS] intent.
    ///
    /// [GUILD_MESSAGE_REACTIONS]: Self::GUILD_MESSAGE_REACTIONS
    pub fn guild_message_reactions(self) -> bool {
        self.contains(Self::GUILD_MESSAGE_REACTIONS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [GUILD_MESSAGE_TYPING] intent.
    ///
    /// [GUILD_MESSAGE_TYPING]: Self::GUILD_MESSAGE_TYPING
    pub fn guild_message_typing(self) -> bool {
        self.contains(Self::GUILD_MESSAGE_TYPING)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [DIRECT_MESSAGES] intent.
    ///
    /// [DIRECT_MESSAGES]: Self::DIRECT_MESSAGES
    pub fn direct_messages(self) -> bool {
        self.contains(Self::DIRECT_MESSAGES)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [DIRECT_MESSAGE_REACTIONS] intent.
    ///
    /// [DIRECT_MESSAGE_REACTIONS]: Self::DIRECT_MESSAGE_REACTIONS
    pub fn direct_message_reactions(self) -> bool {
        self.contains(Self::DIRECT_MESSAGE_REACTIONS)
    }

    /// Shorthand for checking that the set of intents contains the
    /// [DIRECT_MESSAGE_TYPING] intent.
    ///
    /// [DIRECT_MESSAGE_TYPING]: Self::DIRECT_MESSAGE_TYPING
    pub fn direct_message_typing(self) -> bool {
        self.contains(Self::DIRECT_MESSAGE_TYPING)
    }
}
