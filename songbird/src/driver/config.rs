use super::CryptoMode;

#[derive(Clone, Debug, Default)]
pub struct Config {
	pub crypto_mode: Option<CryptoMode>,
}
