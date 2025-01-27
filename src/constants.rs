//! A set of constants used by the library.

use nonmax::NonMaxU16;

/// The maximum length of the textual size of an embed.
pub const EMBED_MAX_LENGTH: usize = 6000;

/// The maximum number of embeds in a message.
pub const EMBED_MAX_COUNT: usize = 10;

/// The maximum number of stickers in a message.
pub const STICKER_MAX_COUNT: usize = 3;

/// The gateway version used by the library. The gateway URL is retrieved via the REST API.
pub const GATEWAY_VERSION: u8 = 10;

/// The large threshold to send on identify.
pub const LARGE_THRESHOLD: u8 = 250;

/// The maximum unicode code points allowed within a message by Discord.
pub const MESSAGE_CODE_LIMIT: usize = 2000;

/// The maximum number of members the bot can fetch at once
pub const MEMBER_FETCH_LIMIT: NonMaxU16 = match NonMaxU16::new(1000) {
    Some(m) => m,
    None => unreachable!(),
};

/// The [UserAgent] sent along with every request.
///
/// [UserAgent]: ::reqwest::header::USER_AGENT
pub const USER_AGENT: &str = concat!(
    "DiscordBot (https://github.com/serenity-rs/serenity, ",
    env!("CARGO_PKG_VERSION"),
    ")"
);

enum_number! {
    /// An enum representing the gateway opcodes.
    ///
    /// [Discord docs](https://discord.com/developers/docs/topics/opcodes-and-status-codes#gateway-gateway-opcodes).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[non_exhaustive]
    pub enum Opcode {
        /// Dispatches an event.
        Dispatch = 0,
        /// Used for ping checking.
        Heartbeat = 1,
        /// Used for client handshake.
        Identify = 2,
        /// Used to update the client status.
        PresenceUpdate = 3,
        /// Used to join/move/leave voice channels.
        VoiceStateUpdate = 4,
        /// Used for voice ping checking.
        VoiceServerPing = 5,
        /// Used to resume a closed connection.
        Resume = 6,
        /// Used to tell clients to reconnect to the gateway.
        Reconnect = 7,
        /// Used to request guild members.
        RequestGuildMembers = 8,
        /// Used to notify clients that they have an invalid session Id.
        InvalidSession = 9,
        /// Sent immediately after connection, contains heartbeat + server info.
        Hello = 10,
        /// Sent immediately following a client heartbeat that was received.
        HeartbeatAck = 11,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// An enum representing the gateway close codes.
    ///
    /// [Discord docs](https://discord.com/developers/docs/topics/opcodes-and-status-codes#gateway-gateway-close-event-codes)
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[non_exhaustive]
    pub enum CloseCode {
        /// Unknown error; try reconnecting.
        UnknownError = 4000,
        /// Invalid gateway opcode.
        UnknownOpcode = 4001,
        /// An invalid payload was sent.
        DecodeError = 4002,
        /// A payload was sent prior to identifying, or the session was invalidated.
        NotAuthenticated = 4003,
        /// The account token sent with the identify payload was incorrect.
        AuthenticationFailed = 4004,
        /// More than one identify payload was sent.
        AlreadyAuthenticated = 4005,
        /// The sequence sent when resuming the session was invalid.
        InvalidSequence = 4007,
        /// Payloads were being sent too quickly.
        RateLimited = 4008,
        /// The gateway session timed out, and a new one must be started.
        SessionTimeout = 4009,
        /// An invalid shard was sent when identifying.
        InvalidShard = 4010,
        /// The session would have handled too many guilds; you must use sharding to connect.
        ShardingRequired = 4011,
        /// An invalid gateway API version was sent.
        InvalidApiVersion = 4012,
        /// An invalid gateway intent was sent.
        InvalidGatewayIntents = 4013,
        /// A disallowed gateway intent was sent; you may have it disabled or may not be approved
        /// to use it.
        DisallowedGatewayIntents = 4014,

        _ => Unknown(u16),
    }
}
