/// Decode behaviour for received RTP packets within the driver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DecodeMode {
	/// Packets received from Discord are handed over to events without any
	/// changes applied.
	///
	/// No CPU work involved.
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
