#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Mode {
    Normal,
    Suffix,
    Lite,
}

impl Mode {
    pub fn to_request_str(self) -> &'static str {
        use Mode::*;
        match self {
            Normal => "xsalsa20_poly1305",
            Suffix => "xsalsa20_poly1305_suffix",
            Lite => "xsalsa20_poly1305_lite",
            _ => unreachable!(),
        }
    }
}

// FIXME: implement encrypt + decrypt + nonce selection for each.
