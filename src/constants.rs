use ::internal::prelude::*;

/// The gateway version used by the library. The gateway URI is retrieved via
/// the REST API.
pub const GATEWAY_VERSION: u8 = 6;
/// The maximum unicode code points allowed within a message by Discord.
pub const MESSAGE_CODE_LIMIT: u16 = 2000;
/// The [UserAgent] sent along with every request.
///
/// [UserAgent]: ../hyper/header/struct.UserAgent.html
pub const USER_AGENT: &'static str = concat!("DiscordBot (https://github.com/zeyla/serenity.rs, ", env!("CARGO_PKG_VERSION"), ")");

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    Heartbeat,
    Hello,
    KeepAlive,
    SelectProtocol,
    SessionDescription,
    Speaking,
}

map_nums! { VoiceOpCode;
    Identify 0,
    SelectProtocol 1,
    Hello 2,
    KeepAlive 3,
    SessionDescription 4,
    Speaking 5,
    Heartbeat 8,
}
