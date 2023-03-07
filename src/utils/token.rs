//! Utilities to parse and validate Discord tokens.

use std::{fmt, str};

use crate::model::id::UserId;

/// Validates that a token is likely in a valid format.
///
/// This performs the following checks on a given token:
///
/// - Is not empty;
/// - Contains 3 parts (split by the period char `'.'`);
/// - The second part of the token is at least 6 characters long;
///
/// # Examples
///
/// Validate that a token is valid and that a number of malformed tokens are
/// actually invalid:
///
/// ```
/// use serenity::utils::token::validate;
///
/// // ensure a valid token is in fact a valid format:
/// assert!(validate("Mjg4NzYwMjQxMzYzODc3ODg4.C_ikow.j3VupLBuE1QWZng3TMGH0z_UAwg").is_ok());
///
/// assert!(validate("Mjg4NzYwMjQxMzYzODc3ODg4").is_err());
/// assert!(validate("").is_err());
/// ```
///
/// # Errors
///
/// Returns a [`InvalidToken`] when one of the above checks fail.
/// The type of failure is not specified.
pub fn validate(token: impl AsRef<str>) -> Result<(), InvalidToken> {
    parse(token).map(|_| ()).ok_or(InvalidToken)
}

/// Error that can be return by [`validate`].
#[derive(Debug)]
pub struct InvalidToken;

impl std::error::Error for InvalidToken {}

impl fmt::Display for InvalidToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("The provided token was invalid")
    }
}

/// Verifies that the token adheres to the Discord token format and extracts the bot user ID and
/// the token generation unix timestamp.
pub fn parse(token: impl AsRef<str>) -> Option<(UserId, i64)> {
    // The token consists of three base64-encoded parts
    // Tokens can be preceded by "Bot " (that's how the Discord API expects them)
    let mut parts = token.as_ref().trim_start_matches("Bot ").split('.');

    // First part must be a base64-encoded stringified user ID
    let user_id = base64::decode_config(parts.next()?, base64::URL_SAFE).ok()?;
    let user_id = UserId(str::from_utf8(&user_id).ok()?.parse().ok()?);

    // Second part must be a base64-encoded token generation timestamp
    let timestamp = parts.next()?;
    // The base64-encoded timestamp must be at least 6 characters
    if timestamp.len() < 6 {
        return None;
    }
    let timestamp_bytes = base64::decode_config(timestamp, base64::URL_SAFE).ok()?;
    let mut timestamp = 0;
    for byte in timestamp_bytes {
        timestamp *= 256;
        timestamp += byte as i64;
    }
    // Some timestamps are based on the Discord epoch. Convert to Unix epoch
    if timestamp < 1_293_840_000 {
        timestamp += 1_293_840_000;
    }

    // Third part is a base64-encoded HMAC that's not interesting on its own
    base64::decode_config(parts.next()?, base64::URL_SAFE).ok()?;

    Some((user_id, timestamp))
}
