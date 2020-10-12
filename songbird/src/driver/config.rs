use super::CryptoMode;

/// Configuration for the inner Driver.
///
/// At present, this cannot be changed.
#[derive(Clone, Debug, Default)]
pub struct Config {
    pub crypto_mode: Option<CryptoMode>,
}
