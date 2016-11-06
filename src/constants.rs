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

impl OpCode {
    pub fn from_num(num: u8) -> Result<OpCode> {
        match num {
            0 => Ok(OpCode::Event),
            1 => Ok(OpCode::Heartbeat),
            2 => Ok(OpCode::Identify),
            3 => Ok(OpCode::StatusUpdate),
            4 => Ok(OpCode::VoiceStateUpdate),
            5 => Ok(OpCode::VoiceServerPing),
            6 => Ok(OpCode::Resume),
            7 => Ok(OpCode::Reconnect),
            8 => Ok(OpCode::GetGuildMembers),
            9 => Ok(OpCode::InvalidSession),
            10 => Ok(OpCode::Hello),
            11 => Ok(OpCode::HeartbeatAck),
            12 => Ok(OpCode::SyncGuild),
            13 => Ok(OpCode::SyncCall),
            other => Err(Error::Decode("Unknown op", Value::U64(other as u64))),
        }
    }

    pub fn num(&self) -> u8 {
        match *self {
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
            OpCode::SyncGuild => 12,
            OpCode::SyncCall => 13,
        }
    }
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

impl VoiceOpCode {
    pub fn from_num(num: u8) -> Result<VoiceOpCode> {
        match num {
            0 => Ok(VoiceOpCode::Identify),
            1 => Ok(VoiceOpCode::SelectProtocol),
            2 => Ok(VoiceOpCode::Hello),
            3 => Ok(VoiceOpCode::Heartbeat),
            4 => Ok(VoiceOpCode::SessionDescription),
            5 => Ok(VoiceOpCode::Speaking),
            other => Err(Error::Decode("Unknown voice op", Value::U64(other as u64))),
        }
    }

    pub fn num(&self) -> u8 {
        use self::*;

        match *self {
            VoiceOpCode::Identify => 0,
            VoiceOpCode::SelectProtocol => 1,
            VoiceOpCode::Hello => 2,
            VoiceOpCode::Heartbeat => 3,
            VoiceOpCode::SessionDescription => 4,
            VoiceOpCode::Speaking => 5,
        }
    }
}
