/// The gateway version used by the library. The gateway URI is retrieved via
/// the REST API.
pub const GATEWAY_VERSION: u8 = 6;
/// The maximum unicode code points allowed within a message by Discord.
pub const MESSAGE_CODE_LIMIT: u16 = 2000;
/// The [UserAgent] sent along with every request.
///
/// [UserAgent]: ../hyper/header/struct.UserAgent.html
pub const USER_AGENT: &'static str = concat!("DiscordBot (https://github.com/zeyla/serenity, ", env!("CARGO_PKG_VERSION"), ")");

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorCode {
    BotsCannotUse,
    CannotSendEmptyMessage,
    CannotSendMessagesInVoice,
    CannotSendMessagesToUser,
    ChannelVerificationTooHigh,
    EditByOtherAuthor,
    EmbedDisabled,
    InvalidAccountType,
    InvalidAuthToken,
    InvalidBulkDeleteCount,
    InvalidDMChannelAction,
    InvalidOauthState,
    InvalidPinChannel,
    MaxFriendsReached,
    MaxGuildsReached,
    MaxPinsReached,
    MaxRolesReached,
    MissingAccess,
    MissingPermissions,
    NoteTooLong,
    Oauth2ApplicationLacksBot,
    Oauth2ApplicationLimitReached,
    OnlyBotsCanUse,
    ReactionBlocked,
    SearchIndexUnavailable,
    TooManyReactions,
    Unauthorized,
    UnknownAccount,
    UnknownApplication,
    UnknownChannel,
    UnknownGuild,
    UnknownEmoji,
    UnknownIntegration,
    UnknownInvite,
    UnknownMember,
    UnknownMessage,
    UnknownOverwrite,
    UnknownProvider,
    UnknownRole,
    UnknownToken,
    UnknownUser,
}

/*
map_nums! { ErrorCode;
    BotsCannotUse 20001,
    CannotSendEmptyMessage 50006,
    CannotSendMessagesInVoice 50008,
    CannotSendMessagesToUser 50007,
    ChannelVerificationTooHigh 50009,
    EditByOtherAuthor 50005,
    EmbedDisabled 50004,
    InvalidAccountType 50002,
    InvalidAuthToken 50014,
    InvalidBulkDeleteCount 50016,
    InvalidDMChannelAction 50003,
    InvalidOauthState 50012,
    InvalidPinChannel 50019,
    MaxFriendsReached 30002,
    MaxGuildsReached 30001,
    MaxPinsReached 30003,
    MaxRolesReached 30005,
    MissingAccess 50001,
    MissingPermissions 500013,
    NoteTooLong 50015,
    Oauth2ApplicationLacksBot 50010,
    Oauth2ApplicationLimitReached 50011,
    OnlyBotsCanUse 20002,
    ReactionBlocked 90001,
    SearchIndexUnavailable 110000,
    TooManyReactions 30010,
    Unauthorized 40001,
    UnknownAccount 10001,
    UnknownApplication 10002,
    UnknownChannel 10003,
    UnknownEmoji 10014,
    UnknownGuild 10004,
    UnknownIntegration 10005,
    UnknownInvite 10006,
    UnknownMember 10007,
    UnknownMessage 10008,
    UnknownOverwrite 10009,
    UnknownProvider 10010,
    UnknownRole 10011,
    UnknownToken 10012,
    UnknownUser 10013,
}
*/

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
    pub fn from_num(num: u64) -> Option<Self> {
        match num {
            0 => Some(OpCode::Event),
            1 => Some(OpCode::Heartbeat),
            2 => Some(OpCode::Identify),
            3 => Some(OpCode::StatusUpdate),
            4 => Some(OpCode::VoiceStateUpdate),
            5 => Some(OpCode::VoiceServerPing),
            6 => Some(OpCode::Resume),
            7 => Some(OpCode::Reconnect),
            8 => Some(OpCode::GetGuildMembers),
            9 => Some(OpCode::InvalidSession),
            10 => Some(OpCode::Hello),
            11 => Some(OpCode::HeartbeatAck),
            12 => Some(OpCode::SyncGuild),
            13 => Some(OpCode::SyncCall),
            _ => None,
        }
    }

    pub fn num(&self) -> u64 {
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
    Heartbeat,
    Hello,
    KeepAlive,
    SelectProtocol,
    SessionDescription,
    Speaking,
}

impl VoiceOpCode {
    pub fn from_num(num: u64) -> Option<Self> {
        match num {
            0 => Some(VoiceOpCode::Identify),
            1 => Some(VoiceOpCode::SelectProtocol),
            2 => Some(VoiceOpCode::Hello),
            3 => Some(VoiceOpCode::KeepAlive),
            4 => Some(VoiceOpCode::SessionDescription),
            5 => Some(VoiceOpCode::Speaking),
            8 => Some(VoiceOpCode::Heartbeat),
            _ => None,
        }
    }

    pub fn num(&self) -> u64 {
        match *self {
            VoiceOpCode::Identify => 0,
            VoiceOpCode::SelectProtocol => 1,
            VoiceOpCode::Hello => 2,
            VoiceOpCode::KeepAlive => 3,
            VoiceOpCode::SessionDescription => 4,
            VoiceOpCode::Speaking => 5,
            VoiceOpCode::Heartbeat => 8,
        }
    }
}
