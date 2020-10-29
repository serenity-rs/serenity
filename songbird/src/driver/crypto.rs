//! Encryption schemes supported by Discord's secure RTP negotiation.

/// Variants of the XSalsa20Poly1305 encryption scheme.
///
/// At present, only `Normal` is supported or selectable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Mode {
    /// The RTP header is used as the source of nonce bytes for the packet.
    ///
    /// Equivalent to a nonce of at most 48b (6B) at no extra packet overhead:
    /// the RTP sequence number and timestamp are the varying quantities.
    Normal,
    /// An additional random 24B suffix is used as the source of nonce bytes for the packet.
    ///
    /// Full nonce width of 24B (192b), at an extra 24B per packet (~1.2 kB/s).
    Suffix,
    /// An additional random 24B suffix is used as the source of nonce bytes for the packet.
    ///
    /// Nonce width of 4B (32b), at an extra 4B per packet (~0.2 kB/s).
    Lite,
}

impl Mode {
    /// Returns the name of a mode as it will appear during negotiation.
    pub fn to_request_str(self) -> &'static str {
        use Mode::*;
        match self {
            Normal => "xsalsa20_poly1305",
            Suffix => "xsalsa20_poly1305_suffix",
            Lite => "xsalsa20_poly1305_lite",
        }
    }
}

// TODO: implement encrypt + decrypt + nonce selection for each.
// This will probably need some research into correct handling of
// padding, reported length, SRTP profiles, and so on.
