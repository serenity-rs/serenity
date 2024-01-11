//! Utilities to parse and validate Discord tokens.

use std::{fmt, str};

/// Validates that a token is likely in a valid format.
///
/// This performs the following checks on a given token:
/// - Is not empty;
/// - Contains 3 parts (split by the period char `'.'`);
/// - The second part of the token is at least 6 characters long;
///
/// # Examples
///
/// Validate that a token is valid and that a number of malformed tokens are actually invalid:
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
/// Returns a [`InvalidToken`] when one of the above checks fail. The type of failure is not
/// specified.
pub fn validate(token: &str) -> Result<(), InvalidToken> {
    // Tokens can be preceded by "Bot " (that's how the Discord API expects them)
    let mut parts = token.trim_start_matches("Bot ").split('.');

    let is_valid = parts.next().is_some_and(|p| !p.is_empty())
        && parts.next().is_some_and(|p| !p.is_empty())
        && parts.next().is_some_and(|p| !p.is_empty())
        && parts.next().is_none();

    if is_valid {
        Ok(())
    } else {
        Err(InvalidToken)
    }
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
