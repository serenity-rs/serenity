//! A set of constants used by the library.

/// The maximum length of the textual size of an embed.
pub const EMBED_MAX_LENGTH: usize = 6000;

/// The gateway version used by the library. The gateway URI is retrieved via
/// the REST API.
pub const GATEWAY_VERSION: u8 = 8;

/// The large threshold to send on identify.
pub const LARGE_THRESHOLD: u8 = 250;

/// The maximum unicode code points allowed within a message by Discord.
pub const MESSAGE_CODE_LIMIT: usize = 2000;

/// The maximum number of members the bot can fetch at once
pub const MEMBER_FETCH_LIMIT: u64 = 1000;

/// The [UserAgent] sent along with every request.
///
/// [UserAgent]: ::reqwest::header::USER_AGENT
pub const USER_AGENT: &str = concat!(
    "DiscordBot (https://github.com/serenity-rs/serenity, ",
    env!("CARGO_PKG_VERSION"),
    ")"
);

/// List of messages Discord shows on member join.
#[allow(clippy::non_ascii_literal)] // allow for discord join messages
pub static JOIN_MESSAGES: &[&str] = &[
    "$user just joined the server - glhf!",
    "$user just joined. Everyone, look busy!",
    "$user just joined. Can I get a heal?",
    "$user joined your party.",
    "$user joined. You must construct additional pylons.",
    "Ermagherd. $user is here.",
    "Welcome, $user. Stay awhile and listen.",
    "Welcome, $user. We were expecting you ( ͡° ͜ʖ ͡°)",
    "Welcome, $user. We hope you brought pizza.",
    "Welcome $user. Leave your weapons by the door.",
    "A wild $user appeared.",
    "Swoooosh. $user just landed.",
    "Brace yourselves. $user just joined the server.",
    "$user just joined... or did they?",
    "$user just arrived. Seems OP - please nerf.",
    "$user just slid into the server.",
    "A $user has spawned in the server.",
    "Big $user showed up!",
    "Where’s $user? In the server!",
    "$user hopped into the server. Kangaroo!!",
    "$user just showed up. Hold my beer.",
    "Challenger approaching - $user has appeared!",
    "It's a bird! It's a plane! Nevermind, it's just $user.",
    r"It's $user! Praise the sun! \[T]/",
    "Never gonna give $user up. Never gonna let $user down.",
    "$user has joined the battle bus.",
    "Cheers, love! $user's here!",
    "Hey! Listen! $user has joined!",
    "We've been expecting you $user",
    "It's dangerous to go alone, take $user!",
    "$user has joined the server! It's super effective!",
    "Cheers, love! $user is here!",
    "$user is here, as the prophecy foretold.",
    "$user has arrived. Party's over.",
    "Ready player $user",
    "$user is here to kick butt and chew bubblegum. And $user is all out of gum.",
    "Hello. Is it $user you're looking for?",
    "$user has joined. Stay a while and listen!",
    "Roses are red, violets are blue, $user joined this server with you",
];

/// Enum to map gateway opcodes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum OpCode {
    /// Dispatches an event.
    Event = 0,
    /// Used for ping checking.
    Heartbeat = 1,
    /// Used for client handshake.
    Identify = 2,
    /// Used to update the client status.
    StatusUpdate = 3,
    /// Used to join/move/leave voice channels.
    VoiceStateUpdate = 4,
    /// Used for voice ping checking.
    VoiceServerPing = 5,
    /// Used to resume a closed connection.
    Resume = 6,
    /// Used to tell clients to reconnect to the gateway.
    Reconnect = 7,
    /// Used to request guild members.
    GetGuildMembers = 8,
    /// Used to notify clients that they have an invalid session Id.
    InvalidSession = 9,
    /// Sent immediately after connection, contains heartbeat + server info.
    Hello = 10,
    /// Sent immediately following a client heartbeat that was received.
    HeartbeatAck = 11,
    /// Unknown opcode.
    Unknown = !0,
}

enum_number!(OpCode {
    Event,
    Heartbeat,
    Identify,
    StatusUpdate,
    VoiceStateUpdate,
    VoiceServerPing,
    Resume,
    Reconnect,
    GetGuildMembers,
    InvalidSession,
    Hello,
    HeartbeatAck,
});

pub mod close_codes {
    /// Unknown error; try reconnecting?
    ///
    /// Can reconnect.
    pub const UNKNOWN_ERROR: u16 = 4000;
    /// Invalid Gateway OP Code.
    ///
    /// Can resume.
    pub const UNKNOWN_OPCODE: u16 = 4001;
    /// An invalid payload was sent.
    ///
    /// Can resume.
    pub const DECODE_ERROR: u16 = 4002;
    /// A payload was sent prior to identifying.
    ///
    /// Cannot reconnect.
    pub const NOT_AUTHENTICATED: u16 = 4003;
    /// The account token sent with the identify payload was incorrect.
    ///
    /// Cannot reconnect.
    pub const AUTHENTICATION_FAILED: u16 = 4004;
    /// More than one identify payload was sent.
    ///
    /// Can reconnect.
    pub const ALREADY_AUTHENTICATED: u16 = 4005;
    /// The sequence sent when resuming the session was invalid.
    ///
    /// Can reconnect.
    pub const INVALID_SEQUENCE: u16 = 4007;
    /// Payloads were being sent too quickly.
    ///
    /// Can resume.
    pub const RATE_LIMITED: u16 = 4008;
    /// A session timed out.
    ///
    /// Can reconnect.
    pub const SESSION_TIMEOUT: u16 = 4009;
    /// An invalid shard when identifying was sent.
    ///
    /// Cannot reconnect.
    pub const INVALID_SHARD: u16 = 4010;
    /// The session would have handled too many guilds.
    ///
    /// Cannot reconnect.
    pub const SHARDING_REQUIRED: u16 = 4011;
    /// Undocumented gateway intents have been provided.
    pub const INVALID_GATEWAY_INTENTS: u16 = 4013;
    /// Disallowed gateway intents have been provided.
    pub const DISALLOWED_GATEWAY_INTENTS: u16 = 4014;
}
