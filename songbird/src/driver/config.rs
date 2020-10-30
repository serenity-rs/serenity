use super::CryptoMode;

/// Configuration for the inner Driver.
///
/// At present, this cannot be changed.
#[derive(Clone, Debug, Default)]
pub struct Config {
    /// Selected tagging mode for voice packet encryption.
    pub crypto_mode: Option<CryptoMode>,
}
