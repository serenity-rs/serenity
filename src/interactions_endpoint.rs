//! Contains tools related to Discord's Interactions Endpoint URL feature.
//!
//! "You can optionally configure an interactions endpoint to receive interactions via HTTP POSTs
//! rather than over Gateway with a bot user."
//!
//! <https://discord.com/developers/docs/tutorials/upgrading-to-application-commands#adding-an-interactions-endpoint-url>
//!
//! See [`Verifier`] for example usage.

/// Parses a hex string into an array of `[u8]`
fn parse_hex<const N: usize>(s: &str) -> Option<[u8; N]> {
    if s.len() != N * 2 {
        return None;
    }

    let mut res = [0; N];
    for (i, byte) in res.iter_mut().enumerate() {
        *byte = u8::from_str_radix(s.get(2 * i..2 * (i + 1))?, 16).ok()?;
    }
    Some(res)
}

/// The byte array couldn't be parsed into a valid cryptographic public key.
#[derive(Debug)]
pub struct InvalidKey(ed25519_dalek::SignatureError);
impl std::fmt::Display for InvalidKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid bot public key: {}", self.0)
    }
}
impl std::error::Error for InvalidKey {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Used to cryptographically verify incoming interactions HTTP request for authenticity.
///
/// If incoming requests are not verified, Discord will reject the URL for security reasons.
///
/// ```rust
/// use serenity::interactions_endpoint::Verifier;
///
/// let verifier =
///     Verifier::new("67c6bd767ca099e79efac9fcce4d2022a63bf7dea780e7f3d813f694c1597089");
///
/// // When receiving an HTTP request:
/// # let http_headers = std::collections::HashMap::from([("X-Signature-Ed25519", ""), ("X-Signature-Timestamp", "")]);
/// # let request_body = &[];
/// let signature = http_headers["X-Signature-Ed25519"];
/// let timestamp = http_headers["X-Signature-Timestamp"];
/// if verifier.verify(signature, timestamp, request_body).is_err() {
///     // Send HTTP 401 Unauthorized response
/// }
/// ```
#[derive(Clone)]
pub struct Verifier {
    public_key: ed25519_dalek::VerifyingKey,
}

impl Verifier {
    /// Creates a new [`Verifier`] from the given public key hex string.
    ///
    /// Panics if the given key is invalid. For a low-level, non-panicking variant, see
    /// [`Self::try_new()`].
    #[must_use]
    pub fn new(public_key: &str) -> Self {
        Self::try_new(parse_hex(public_key).expect("public key must be a 64 digit hex string"))
            .expect("invalid public key")
    }

    /// Creates a new [`Verifier`] from the public key bytes.
    ///
    /// # Errors
    ///
    /// [`InvalidKey`] if the key isn't cryptographically valid.
    pub fn try_new(public_key: [u8; 32]) -> Result<Self, InvalidKey> {
        Ok(Self {
            public_key: ed25519_dalek::VerifyingKey::from_bytes(&public_key).map_err(InvalidKey)?,
        })
    }

    /// Verifies a Discord request for authenticity, given the `X-Signature-Ed25519` HTTP header,
    /// `X-Signature-Timestamp` HTTP headers and request body.
    // We just need to differentiate "pass" and "failure". There's deliberately no data besides ().
    #[expect(clippy::result_unit_err, clippy::missing_errors_doc)]
    pub fn verify(&self, signature: &str, timestamp: &str, body: &[u8]) -> Result<(), ()> {
        use ed25519_dalek::Verifier as _;

        // Extract and parse signature
        let signature_bytes = parse_hex(signature).ok_or(())?;
        let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes);

        // Verify
        let message_to_verify = [timestamp.as_bytes(), body].concat();
        self.public_key.verify(&message_to_verify, &signature).map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex::<4>("bf7dea78"), Some([0xBF, 0x7D, 0xEA, 0x78]));
        assert_eq!(parse_hex::<4>("bf7dea7"), None);
        assert_eq!(parse_hex::<4>("bf7dea789"), None);
        assert_eq!(parse_hex::<4>("bf7dea7x"), None);
        assert_eq!(parse_hex(""), Some([]));
        assert_eq!(
            parse_hex("67c6bd767ca099e79efac9fcce4d2022a63bf7dea780e7f3d813f694c1597089"),
            Some([
                0x67, 0xC6, 0xBD, 0x76, 0x7C, 0xA0, 0x99, 0xE7, 0x9E, 0xFA, 0xC9, 0xFC, 0xCE, 0x4D,
                0x20, 0x22, 0xA6, 0x3B, 0xF7, 0xDE, 0xA7, 0x80, 0xE7, 0xF3, 0xD8, 0x13, 0xF6, 0x94,
                0xC1, 0x59, 0x70, 0x89
            ])
        );
    }
}
