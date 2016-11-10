use ::prelude_internal::*;

/// The base URI for the API.
pub const API_BASE: &'static str = "https://discordapp.com/api/v6";
/// The gateway version used by the library. The gateway URI is retrieved via
/// the REST API.
pub const GATEWAY_VERSION: u8 = 6;
/// The [UserAgent] sent along with every request.
///
/// [UserAgent]: ../hyper/header/struct.UserAgent.html
pub const USER_AGENT: &'static str = concat!("DiscordBot (https://github.com/zeyla/serenity, ", env!("CARGO_PKG_VERSION"), ")");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpCode {
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
    SyncGuild,
    SyncCall,
}

map_nums! { OpCode;
    Event 0,
    Heartbeat 1,
    Identify 2,
    StatusUpdate 3,
    VoiceStateUpdate 4,
    VoiceServerPing 5,
    Resume 6,
    Reconnect 7,
    GetGuildMembers 8,
    InvalidSession 9,
    Hello 10,
    HeartbeatAck 11,
    SyncGuild 12,
    SyncCall 13,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoiceOpCode {
    Identify,
    SelectProtocol,
    Hello,
    Heartbeat,
    SessionDescription,
    Speaking,
}

map_nums! { VoiceOpCode;
    Identify 0,
    SelectProtocol 1,
    Hello 2,
    Heartbeat 3,
    SessionDescription 4,
    Speaking 5,
}
