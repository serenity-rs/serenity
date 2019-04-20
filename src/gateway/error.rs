use std::{
    error::Error as StdError,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult
    }
};
use tungstenite::protocol::CloseFrame;

/// An error that occurred while attempting to deal with the gateway.
///
/// Note that - from a user standpoint - there should be no situation in which
/// you manually handle these.
#[derive(Clone, Debug)]
pub enum Error {
    /// There was an error building a URL.
    BuildingUrl,
    /// The connection closed, potentially uncleanly.
    Closed(Option<CloseFrame<'static>>),
    /// Expected a Hello during a handshake
    ExpectedHello,
    /// When there was an error sending a heartbeat.
    HeartbeatFailed,
    /// When invalid authentication (a bad token) was sent in the IDENTIFY.
    InvalidAuthentication,
    /// Expected a Ready or an InvalidateSession
    InvalidHandshake,
    /// An indicator that an unknown opcode was received from the gateway.
    InvalidOpCode,
    /// When invalid sharding data was sent in the IDENTIFY.
    ///
    /// # Examples
    ///
    /// Sending a shard ID of 5 when sharding with 3 total is considered
    /// invalid.
    InvalidShardData,
    /// When no authentication was sent in the IDENTIFY.
    NoAuthentication,
    /// When a session Id was expected (for resuming), but was not present.
    NoSessionId,
    /// When a shard would have too many guilds assigned to it.
    ///
    /// # Examples
    ///
    /// When sharding 5500 guilds on 2 shards, at least one of the shards will
    /// have over the maximum number of allowed guilds per shard.
    ///
    /// This limit is currently 2500 guilds per shard.
    OverloadedShard,
    /// Failed to reconnect after a number of attempts.
    ReconnectFailure,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { f.write_str(self.description()) }
}

impl StdError for Error {
    fn description(&self) -> &str {
        use self::Error::*;

        match *self {
            BuildingUrl => "Error building url",
            Closed(_) => "Connection closed",
            ExpectedHello => "Expected a Hello",
            HeartbeatFailed => "Failed sending a heartbeat",
            InvalidAuthentication => "Sent invalid authentication",
            InvalidHandshake => "Expected a valid Handshake",
            InvalidOpCode => "Invalid OpCode",
            InvalidShardData => "Sent invalid shard data",
            NoAuthentication => "Sent no authentication",
            NoSessionId => "No Session Id present when required",
            OverloadedShard => "Shard has too many guilds",
            ReconnectFailure => "Failed to Reconnect",
            __Nonexhaustive => unreachable!(),
        }
    }
}
