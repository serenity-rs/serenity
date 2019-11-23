//! A set of constants used by the library.

/// The maximum length of the textual size of an embed.
pub const EMBED_MAX_LENGTH: u16 = 6000;
/// The gateway version used by the library. The gateway URI is retrieved via
/// the REST API.
pub const GATEWAY_VERSION: u8 = 6;
/// The voice gateway version used by the library.
pub const VOICE_GATEWAY_VERSION: u8 = 3;
/// The large threshold to send on identify.
pub const LARGE_THRESHOLD: u8 = 250;
/// The maximum unicode code points allowed within a message by Discord.
pub const MESSAGE_CODE_LIMIT: u16 = 2000;
/// The [UserAgent] sent along with every request.
///
/// [UserAgent]: ../../reqwest/header/constant.USER_AGENT.html
pub const USER_AGENT: &str = concat!(
    "DiscordBot (https://github.com/serenity-rs/serenity, ",
    env!("CARGO_PKG_VERSION"),
    ")"
);

/// List of messages Discord shows on member join.
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
    #[doc(hidden)]
    __Nonexhaustive,
}

enum_number!(
    OpCode {
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
    }
);

impl OpCode {
    pub fn num(self) -> u64 {
        match self {
            OpCode::Event => 0,
            OpCode::Heartbeat => 1,
            OpCode::Identify => 2,
            OpCode::StatusUpdate => 3,
            OpCode::VoiceStateUpdate => 4,
            OpCode::VoiceServerPing => 5,
            OpCode::Resume => 6,
            OpCode::Reconnect => 7,
            OpCode::GetGuildMembers => 8,
            OpCode::InvalidSession => 9,
            OpCode::Hello => 10,
            OpCode::HeartbeatAck => 11,
            OpCode::__Nonexhaustive => unreachable!(),
        }
    }
}

/// Enum to map voice opcodes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum VoiceOpCode {
    /// Used to begin a voice websocket connection.
    Identify = 0,
    /// Used to select the voice protocol.
    SelectProtocol = 1,
    /// Used to complete the websocket handshake.
    Ready = 2,
    /// Used to keep the websocket connection alive.
    Heartbeat = 3,
    /// Used to describe the session.
    SessionDescription = 4,
    /// Used to indicate which users are speaking.
    Speaking = 5,
    /// Heartbeat ACK, received by the client to show the server's receipt of a heartbeat.
    HeartbeatAck = 6,
    /// Sent after a disconnect to attempt to resume a session.
    Resume = 7,
    /// Used to determine how often the client must send a heartbeat.
    Hello = 8,
    /// Sent by the server if a session coulkd successfully be resumed.
    Resumed = 9,
    /// Message indicating that another user has connected to the voice channel.
    ClientConnect = 12,
    /// Message indicating that another user has disconnected from the voice channel.
    ClientDisconnect = 13,
    #[doc(hidden)]
    __Nonexhaustive,
}

enum_number!(
    VoiceOpCode {
        Identify,
        SelectProtocol,
        Ready,
        Heartbeat,
        SessionDescription,
        Speaking,
        HeartbeatAck,
        Resume,
        Hello,
        Resumed,
        ClientConnect,
        ClientDisconnect,
    }
);

impl VoiceOpCode {
    pub fn num(self) -> u64 {
        match self {
            VoiceOpCode::Identify => 0,
            VoiceOpCode::SelectProtocol => 1,
            VoiceOpCode::Ready => 2,
            VoiceOpCode::Heartbeat => 3,
            VoiceOpCode::SessionDescription => 4,
            VoiceOpCode::Speaking => 5,
            VoiceOpCode::HeartbeatAck => 6,
            VoiceOpCode::Resume => 7,
            VoiceOpCode::Hello => 8,
            VoiceOpCode::Resumed => 9,
            VoiceOpCode::ClientConnect => 12,
            VoiceOpCode::ClientDisconnect => 13,
            VoiceOpCode::__Nonexhaustive => unreachable!(),
        }
    }
}

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
}
