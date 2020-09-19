use std::{
    error::Error as StdError,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult
    }
};
use async_tungstenite::tungstenite::protocol::CloseFrame;

/// An error that occurred while attempting to deal with the gateway.
///
/// Note that - from a user standpoint - there should be no situation in which
/// you manually handle these.
#[derive(Clone, Debug)]
#[non_exhaustive]
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
    /// When undocumented gateway intents are provided.
    InvalidGatewayIntents,
    /// When disallowed gatewax intents are provided.
    ///
    /// If an connection has been established but priviliged gateway intents
    /// were provided without enabling them prior.
    DisallowedGatewayIntents,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::BuildingUrl => f.write_str("Error building url"),
            Error::Closed(_) => f.write_str("Connection closed"),
            Error::ExpectedHello => f.write_str("Expected a Hello"),
            Error::HeartbeatFailed => f.write_str("Failed sending a heartbeat"),
            Error::InvalidAuthentication => f.write_str("Sent invalid authentication"),
            Error::InvalidHandshake => f.write_str("Expected a valid Handshake"),
            Error::InvalidOpCode => f.write_str("Invalid OpCode"),
            Error::InvalidShardData => f.write_str("Sent invalid shard data"),
            Error::NoAuthentication => f.write_str("Sent no authentication"),
            Error::NoSessionId => f.write_str("No Session Id present when required"),
            Error::OverloadedShard => f.write_str("Shard has too many guilds"),
            Error::ReconnectFailure => f.write_str("Failed to Reconnect"),
            Error::InvalidGatewayIntents => f.write_str("Invalid gateway intents were provided"),
            Error::DisallowedGatewayIntents => f.write_str("Disallowed gateway intents were provided"),
        }
    }
}

impl StdError for Error {}
