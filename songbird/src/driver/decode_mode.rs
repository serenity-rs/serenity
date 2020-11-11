/// Decode behaviour for received RTP packets within the driver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DecodeMode {
    /// Packets received from Discord are handed over to events without any
    /// changes applied.
    ///
    /// No CPU work involved.
    ///
    /// *BEWARE: this will almost certainly break [user speaking events].
    /// Silent frame detection only works if extensions can be parsed or
    /// are not present, as they are encrypted.
    /// This event requires such functionality.*
    ///
    /// [user speaking events]: ../events/enum.CoreEvent.html#variant.SpeakingUpdate
    Pass,
    /// Decrypts the body of each received packet.
    ///
    /// Small per-packet CPU use.
    Decrypt,
    /// Decrypts and decodes each received packet, correctly accounting for losses.
    ///
    /// Larger per-packet CPU use.
    Decode,
}

impl DecodeMode {
    /// Returns whether this mode will decrypt received packets.
    pub fn should_decrypt(self) -> bool {
        self != DecodeMode::Pass
    }
}
