//! The Client contains information about a single bot or user's token, as well
//! as event handlers. Dispatching events to configured handlers and starting
//! the shards' connections are handled directly via the client. In addition,
//! the `http` module and `Cache` are also automatically handled by the
//! Client module for you.
//!
//! A [`Context`] is provided for every handler.
//!
//! The `http` module is the lower-level method of interacting with the Discord
//! REST API. Realistically, there should be little reason to use this yourself,
//! as the Context will do this for you. A possible use case of using the `http`
//! module is if you do not have a Cache, for purposes such as low memory
//! requirements.
//!
//! Click [here][Client examples] for an example on how to use a `Client`.
//!
//! [`Client`]: struct.Client.html#examples
//! [`Context`]: struct.Context.html
//! [Client examples]: struct.Client.html#examples

pub mod shard_manager;

mod error;

pub use self::error::Error as ClientError;

use futures::Future;
use hyper::client::{Client as HyperClient, HttpConnector};
use hyper::Body;
use hyper_tls::HttpsConnector;
use self::shard_manager::{ShardManager, ShardManagerOptions, ShardingStrategy};
use std::sync::Arc;
use super::http::Client as HttpClient;
use Error;

#[cfg(feature = "cache")]
use cache::Cache;

#[derive(Debug)]
pub struct ClientOptions {
    pub http_client: Arc<HyperClient<HttpsConnector<HttpConnector>, Body>>,
    pub sharding: ShardingStrategy,
    pub token: String,
}

#[derive(Debug)]
pub struct Client {
    #[cfg(feature = "cache")]
    pub cache: Arc<RefCell<Cache>>,
    pub http: Arc<HttpClient>,
    pub shard_manager: ShardManager,
    token: Arc<String>,
    ws_uri: Arc<String>,
}

impl Client {
    pub fn new(options: ClientOptions) -> Box<Future<Item = Self, Error = Error>> {
        let token = {
            let trimmed = options.token.trim();

            Arc::new(if trimmed.starts_with("Bot ") {
                trimmed.to_string()
            } else {
                format!("Bot {}", trimmed)
            })
        };

        let strategy = options.sharding;
        let client = Arc::new(ftry!(HttpClient::new(
            options.http_client,
            Arc::clone(&token),
        )));

        let done = client.get_bot_gateway().map(move |gateway| {
            let uri = Arc::new(gateway.url);

            Self {
                #[cfg(feature = "cache")]
                cache: Arc::new(RefCell::new(Cache::default())),
                http: client,
                shard_manager: ShardManager::new(ShardManagerOptions {
                    strategy: strategy,
                    token: Arc::clone(&token),
                    ws_uri: Arc::clone(&uri),
                }),
                token: token,
                ws_uri: Arc::clone(&uri),
            }
        }).from_err();

        Box::new(done)
    }

    // pub fn connect(&self) -> ::futures::Stream<Item = Dispatch, Error = ::Error> {
    //     self.shard_manager.start().map(|(shard_id, msg)| {
    //         Dispatch {
    //             msg,
    //             shard_id,
    //         }
    //     })
    // }
}

pub struct Dispatch {
    pub msg: ::model::event::GatewayEvent,
    pub shard_id: u64,
}

// Validates that a token is likely in a valid format.
//
// This performs the following checks on a given token:
//
// - At least one character long;
// - Contains 3 parts (split by the period char `'.'`);
// - The second part of the token is at least 6 characters long;
// - The token does not contain any whitespace prior to or after the token.
//
// # Examples
//
// Validate that a token is valid and that a number of invalid tokens are
// actually invalid:
//
// ```rust,no_run
// use serenity::client::validate_token;
//
// // ensure a valid token is in fact valid:
// assert!(validate_token"Mjg4NzYwMjQxMzYzODc3ODg4.C_ikow.j3VupLBuE1QWZng3TMGH0z_UAwg").is_ok());
//
// // "cat" isn't a valid token:
// assert!(validate_token("cat").is_err());
//
// // tokens must have three parts, separated by periods (this is still
// // actually an invalid token):
// assert!(validate_token("aaa.abcdefgh.bbb").is_ok());
//
// // the second part must be _at least_ 6 characters long:
// assert!(validate_token("a.abcdef.b").is_ok());
// assert!(validate_token("a.abcde.b").is_err());
// ```
//
// # Errors
//
// Returns a [`ClientError::InvalidToken`] when one of the above checks fail.
// The type of failure is not specified.
//
// [`ClientError::InvalidToken`]: enum.ClientError.html#variant.InvalidToken
// pub fn validate_token(token: &str) -> Result<()> {
//     if token.is_empty() {
//         return Err(Error::Client(ClientError::InvalidToken));
//     }

//     let parts: Vec<&str> = token.split('.').collect();

//     // Check that the token has a total of 3 parts.
//     if parts.len() != 3 {
//         return Err(Error::Client(ClientError::InvalidToken));
//     }

//     // Check that the second part is at least 6 characters long.
//     if parts[1].len() < 6 {
//         return Err(Error::Client(ClientError::InvalidToken));
//     }

//     // Check that there is no whitespace before/after the token.
//     if token.trim() != token {
//         return Err(Error::Client(ClientError::InvalidToken));
//     }

//     Ok(())
// }
