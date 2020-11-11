use super::{CryptoMode, DecodeMode};

/// Configuration for the inner Driver.
///
#[derive(Clone, Debug)]
pub struct Config {
    /// Selected tagging mode for voice packet encryption.
    ///
    /// Defaults to [`CryptoMode::Normal`].
    ///
    /// Changes to this field will not immediately apply if the
    /// driver is actively connected, but will apply to subsequent
    /// sessions.
    ///
    /// [`CryptoMode::Normal`]: enum.CryptoMode.html#variant.Normal
    pub crypto_mode: CryptoMode,
    /// Configures whether decoding and decryption occur for all received packets.
    ///
    /// If voice receiving voice packets, generally you should choose [`DecodeMode::Decode`].
    /// [`DecodeMode::Decrypt`] is intended for users running their own selective decoding,
    /// who rely upon [user speaking events], or who need to inspect Opus packets.
    /// If you're certain you will never need any RT(C)P events, then consider [`DecodeMode::Pass`].
    ///
    /// Defaults to [`DecodeMode::Decrypt`]. This is due to per-packet decoding costs,
    /// which most users will not want to pay, but allowing speaking events which are commonly used.
    ///
    /// [`DecodeMode::Decode`]: enum.DecodeMode.html#variant.Decode
    /// [`DecodeMode::Decrypt`]: enum.DecodeMode.html#variant.Decrypt
    /// [`DecodeMode::Pass`]: enum.DecodeMode.html#variant.Pass
    /// [user speaking events]: ../events/enum.CoreEvent.html#variant.SpeakingUpdate
    pub decode_mode: DecodeMode,
    /// Number of concurrently active tracks to allocate memory for.
    ///
    /// This should be set at, or just above, the maximum number of tracks
    /// you expect your bot will play at the same time. Exceeding the size of
    /// the internal queue will trigger a larger memory allocation and copy,
    /// possibly causing the mixer thread to miss a packet deadline.
    ///
    /// Defaults to `1`.
    ///
    /// Changes to this field in a running driver will only ever increase
    /// the capacity of the track store.
    pub preallocated_tracks: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            crypto_mode: CryptoMode::Normal,
            decode_mode: DecodeMode::Decrypt,
            preallocated_tracks: 1,
        }
    }
}

impl Config {
    /// Sets this `Config`'s chosen cryptographic tagging scheme.
    pub fn crypto_mode(mut self, crypto_mode: CryptoMode) -> Self {
        self.crypto_mode = crypto_mode;
        self
    }

    /// Sets this `Config`'s received packet decryption/decoding behaviour.
    pub fn decode_mode(mut self, decode_mode: DecodeMode) -> Self {
        self.decode_mode = decode_mode;
        self
    }

    /// Sets this `Config`'s number of tracks to preallocate.
    pub fn preallocated_tracks(mut self, preallocated_tracks: usize) -> Self {
        self.preallocated_tracks = preallocated_tracks;
        self
    }

    /// This is used to prevent changes which would invalidate the current session.
    pub(crate) fn make_safe(&mut self, previous: &Config, connected: bool) {
        if connected {
            self.crypto_mode = previous.crypto_mode;
        }
    }
}
