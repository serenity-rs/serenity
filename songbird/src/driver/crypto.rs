//! Encryption schemes supported by Discord's secure RTP negotiation.
use byteorder::{NetworkEndian, WriteBytesExt};
use discortp::{rtp::RtpPacket, MutablePacket};
use rand::Rng;
use std::num::Wrapping;
use xsalsa20poly1305::{
    aead::{AeadInPlace, Error as CryptoError},
    Nonce,
    Tag,
    XSalsa20Poly1305 as Cipher,
    NONCE_SIZE,
    TAG_SIZE,
};

/// Variants of the XSalsa20Poly1305 encryption scheme.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CryptoMode {
    /// The RTP header is used as the source of nonce bytes for the packet.
    ///
    /// Equivalent to a nonce of at most 48b (6B) at no extra packet overhead:
    /// the RTP sequence number and timestamp are the varying quantities.
    Normal,
    /// An additional random 24B suffix is used as the source of nonce bytes for the packet.
    /// This is regenerated randomly for each packet.
    ///
    /// Full nonce width of 24B (192b), at an extra 24B per packet (~1.2 kB/s).
    Suffix,
    /// An additional random 4B suffix is used as the source of nonce bytes for the packet.
    /// This nonce value increments by `1` with each packet.
    ///
    /// Nonce width of 4B (32b), at an extra 4B per packet (~0.2 kB/s).
    Lite,
}

impl From<CryptoState> for CryptoMode {
    fn from(val: CryptoState) -> Self {
        use CryptoState::*;
        match val {
            Normal => CryptoMode::Normal,
            Suffix => CryptoMode::Suffix,
            Lite(_) => CryptoMode::Lite,
        }
    }
}

impl CryptoMode {
    /// Returns the name of a mode as it will appear during negotiation.
    pub fn to_request_str(self) -> &'static str {
        use CryptoMode::*;
        match self {
            Normal => "xsalsa20_poly1305",
            Suffix => "xsalsa20_poly1305_suffix",
            Lite => "xsalsa20_poly1305_lite",
        }
    }

    /// Returns the number of bytes each nonce is stored as within
    /// a packet.
    pub fn nonce_size(self) -> usize {
        use CryptoMode::*;
        match self {
            Normal => RtpPacket::minimum_packet_size(),
            Suffix => NONCE_SIZE,
            Lite => 4,
        }
    }

    /// Returns the number of bytes occupied by the encryption scheme
    /// which fall before the payload.
    pub fn payload_prefix_len(self) -> usize {
        TAG_SIZE
    }

    /// Returns the number of bytes occupied by the encryption scheme
    /// which fall after the payload.
    pub fn payload_suffix_len(self) -> usize {
        use CryptoMode::*;
        match self {
            Normal => 0,
            Suffix | Lite => self.nonce_size(),
        }
    }

    /// Calculates the number of additional bytes required compared
    /// to an unencrypted payload.
    pub fn payload_overhead(self) -> usize {
        self.payload_prefix_len() + self.payload_suffix_len()
    }

    /// Extracts the byte slice in a packet used as the nonce, and the remaining mutable
    /// portion of the packet.
    fn nonce_slice<'a>(self, header: &'a [u8], body: &'a mut [u8]) -> (&'a [u8], &'a mut [u8]) {
        use CryptoMode::*;
        match self {
            Normal => (header, body),
            Suffix | Lite => {
                let len = body.len();
                let (body_left, nonce_loc) = body.split_at_mut(len - self.payload_suffix_len());
                (&nonce_loc[..self.nonce_size()], body_left)
            },
        }
    }

    /// Decrypts a Discord RT(C)P packet using the given key.
    ///
    /// If successful, this returns the number of bytes to be ignored from the
    /// start and end of the packet payload.
    #[inline]
    pub(crate) fn decrypt_in_place(
        self,
        packet: &mut impl MutablePacket,
        cipher: &Cipher,
    ) -> Result<(usize, usize), CryptoError> {
        let header_len = packet.packet().len() - packet.payload().len();
        let (header, body) = packet.packet_mut().split_at_mut(header_len);
        let (slice_to_use, body_remaining) = self.nonce_slice(header, body);

        let mut nonce = Nonce::default();
        let nonce_slice = if slice_to_use.len() == NONCE_SIZE {
            Nonce::from_slice(&slice_to_use[..NONCE_SIZE])
        } else {
            let max_bytes_avail = slice_to_use.len();
            nonce[..self.nonce_size().min(max_bytes_avail)].copy_from_slice(slice_to_use);
            &nonce
        };

        let body_start = self.payload_prefix_len();
        let body_tail = self.payload_suffix_len();

        let (tag_bytes, data_bytes) = body_remaining.split_at_mut(body_start);
        let tag = Tag::from_slice(tag_bytes);

        Ok(cipher
            .decrypt_in_place_detached(nonce_slice, b"", data_bytes, tag)
            .map(|_| (body_start, body_tail))?)
    }

    /// Encrypts a Discord RT(C)P packet using the given key.
    ///
    /// Use of this requires that the input packet has had a nonce generated in the correct location,
    /// and `payload_len` specifies the number of bytes after the header including this nonce.
    #[inline]
    pub fn encrypt_in_place(
        self,
        packet: &mut impl MutablePacket,
        cipher: &Cipher,
        payload_len: usize,
    ) -> Result<(), CryptoError> {
        let header_len = packet.packet().len() - packet.payload().len();
        let (header, body) = packet.packet_mut().split_at_mut(header_len);
        let (slice_to_use, body_remaining) = self.nonce_slice(header, &mut body[..payload_len]);

        let mut nonce = Nonce::default();
        let nonce_slice = if slice_to_use.len() == NONCE_SIZE {
            Nonce::from_slice(&slice_to_use[..NONCE_SIZE])
        } else {
            nonce[..self.nonce_size()].copy_from_slice(slice_to_use);
            &nonce
        };

        // body_remaining is now correctly truncated by this point.
        // the true_payload to encrypt follows after the first TAG_LEN bytes.
        let tag =
            cipher.encrypt_in_place_detached(nonce_slice, b"", &mut body_remaining[TAG_SIZE..])?;
        body_remaining[..TAG_SIZE].copy_from_slice(&tag[..]);

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum CryptoState {
    Normal,
    Suffix,
    Lite(Wrapping<u32>),
}

impl From<CryptoMode> for CryptoState {
    fn from(val: CryptoMode) -> Self {
        use CryptoMode::*;
        match val {
            Normal => CryptoState::Normal,
            Suffix => CryptoState::Suffix,
            Lite => CryptoState::Lite(Wrapping(rand::random::<u32>())),
        }
    }
}

impl CryptoState {
    /// Writes packet nonce into the body, if required, returning the new length.
    pub fn write_packet_nonce(
        &mut self,
        packet: &mut impl MutablePacket,
        payload_end: usize,
    ) -> usize {
        let mode = self.kind();
        let endpoint = payload_end + mode.payload_suffix_len();

        use CryptoState::*;
        match self {
            Suffix => {
                rand::thread_rng().fill(&mut packet.payload_mut()[payload_end..endpoint]);
            },
            Lite(mut i) => {
                (&mut packet.payload_mut()[payload_end..endpoint])
                    .write_u32::<NetworkEndian>(i.0)
                    .expect(
                        "Nonce size is guaranteed to be sufficient to write u32 for lite tagging.",
                    );
                i += Wrapping(1);
            },
            _ => {},
        }

        endpoint
    }

    pub fn kind(&self) -> CryptoMode {
        CryptoMode::from(*self)
    }
}
