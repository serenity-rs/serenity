use std::env::{self, VarError};
use std::ffi::OsStr;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use aformat::{aformat, CapStr};

/// A cheaply clonable, zeroed on drop, String.
///
/// This is a simple newtype of `Arc<str>` that uses [`zeroize::Zeroize`] on last drop to avoid
/// keeping it around in memory.
#[derive(Clone, Deserialize, Serialize)]
pub struct SecretString(Arc<str>);

impl SecretString {
    #[must_use]
    pub fn new(inner: Arc<str>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_tuple(std::any::type_name::<Self>()).field(&"<secret>").finish()
    }
}

impl zeroize::Zeroize for SecretString {
    fn zeroize(&mut self) {
        if let Some(string) = Arc::get_mut(&mut self.0) {
            string.zeroize();
        }
    }
}

#[cfg(feature = "typesize")]
impl typesize::TypeSize for SecretString {
    fn extra_size(&self) -> usize {
        self.0.len() + (size_of::<usize>() * 2)
    }
}

/// A type for securely storing and passing around a Discord token.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "&str")]
pub struct Token(SecretString);

impl Token {
    /// Fetch and parses the token out of the given environment variable.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::Env`] if fetching the variable fails (see [`std::env::var`] for
    /// details). May also return [`TokenError::InvalidToken`] if the token is in an invalid
    /// format (see [`Token::from_str`]).
    pub fn from_env<K: AsRef<OsStr>>(key: K) -> Result<Self, TokenError> {
        env::var(key).map_err(TokenError::Env).and_then(|token| token.parse())
    }

    #[must_use]
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

/// Parses a token and validates that is is likely in a valid format.
///
/// This performs the following checks on a given token:
/// - Is not empty;
/// - Is optionally prefixed with `"Bot "` or `"Bearer "`;
/// - Contains 3 parts (split by the period char `'.'`);
///
/// Note that a token prefixed with `"Bearer "` will have its prefix changed to `"Bot "` when
/// parsed.
///
/// # Examples
///
/// Validate that a token is valid and that a number of malformed tokens are actually invalid:
///
/// ```
/// use serenity::secrets::Token;
///
/// // ensure a valid token is in fact a valid format:
/// assert!("Mjg4NzYwMjQxMzYzODc3ODg4.C_ikow.j3VupLBuE1QWZng3TMGH0z_UAwg".parse::<Token>().is_ok());
///
/// assert!("Mjg4NzYwMjQxMzYzODc3ODg4".parse::<Token>().is_err());
/// assert!("".parse::<Token>().is_err());
/// ```
///
/// # Errors
///
/// Returns a [`TokenError::InvalidToken`] when one of the above checks fail. The type of failure is
/// not specified.
impl FromStr for Token {
    type Err = TokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token = s.trim().trim_start_matches("Bot ").trim_start_matches("Bearer ");

        let mut parts = token.split('.');
        let is_valid = parts.next().is_some_and(|p| !p.is_empty())
            && parts.next().is_some_and(|p| !p.is_empty())
            && parts.next().is_some_and(|p| !p.is_empty())
            && parts.next().is_none();

        if is_valid {
            Ok(Self(SecretString::new(Arc::from(
                aformat!("Bot {}", CapStr::<128>(token)).as_str(),
            ))))
        } else {
            Err(TokenError::InvalidToken)
        }
    }
}

/// Parses a token and validates that is is likely in a valid format.
///
/// Refer to the [`Token::from_str`] implementation for details.
impl TryFrom<&str> for Token {
    type Error = TokenError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// Parses a token and validates that is is likely in a valid format.
///
/// Refer to the [`Token::from_str`] implementation for details.
impl TryFrom<String> for Token {
    type Error = TokenError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// Error that can be returned by [`Token::from_str`], [`Token::try_from`], or [`Token::from_env`].
#[derive(Debug)]
pub enum TokenError {
    Env(VarError),
    InvalidToken,
}

impl std::error::Error for TokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Env(inner) => Some(inner),
            Self::InvalidToken => None,
        }
    }
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Env(inner) => fmt::Display::fmt(&inner, f),
            Self::InvalidToken => f.write_str("The provided token was invalid"),
        }
    }
}
