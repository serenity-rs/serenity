use std::error::Error as StdError;
use std::fmt;

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
    /// When disallowed gateway intents are provided.
    ///
    /// If an connection has been established but privileged gateway intents
    /// were provided without enabling them prior.
    DisallowedGatewayIntents,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildingUrl => f.write_str("Error building url"),
            Self::Closed(_) => f.write_str("Connection closed"),
            Self::ExpectedHello => f.write_str("Expected a Hello"),
            Self::HeartbeatFailed => f.write_str("Failed sending a heartbeat"),
            Self::InvalidAuthentication => f.write_str("Sent invalid authentication"),
            Self::InvalidHandshake => f.write_str("Expected a valid Handshake"),
            Self::InvalidOpCode => f.write_str("Invalid OpCode"),
            Self::InvalidShardData => f.write_str("Sent invalid shard data"),
            Self::NoAuthentication => f.write_str("Sent no authentication"),
            Self::NoSessionId => f.write_str("No Session Id present when required"),
            Self::OverloadedShard => f.write_str("Shard has too many guilds"),
            Self::ReconnectFailure => f.write_str("Failed to Reconnect"),
            Self::InvalidGatewayIntents => f.write_str("Invalid gateway intents were provided"),
            Self::DisallowedGatewayIntents => {
                f.write_str("Disallowed gateway intents were provided")
            },
        }
    }
}

impl StdError for Error {}
