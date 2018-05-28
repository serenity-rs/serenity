//! A set of constants denoting the URIs that the lib uses and a constant
//! representing the version in use.

/// The base URI to the REST API.
pub const API_URI_BASE: &str = "https://discordapp.com/api";
/// The versioned URI to the REST API.
pub const API_URI_VERSIONED: &str = "https://discordapp.com/api/v6";
/// The status page base URI.
pub const STATUS_URI_BASE: &str = "https://status.discordapp.com/api";
/// The versioned URI to the status page.
pub const STATUS_URI_VERSIONED: &str = "https://status.discordapp.com/api/v2";
/// The API version that the library supports and uses.
pub const VERSION: u8 = 6;
