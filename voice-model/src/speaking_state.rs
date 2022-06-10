use bitflags::bitflags;
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Flag set describing how a speaker is sending audio.
    pub struct SpeakingState: u8 {
        /// Normal transmission of voice audio.
        const MICROPHONE = 1;

        /// Transmission of context audio for video, no speaking indicator.
        const SOUNDSHARE = 1 << 1;

        /// Priority speaker, lowering audio of other speakers.
        const PRIORITY = 1 << 2;
    }
}

impl SpeakingState {
    pub fn microphone(self) -> bool {
        self.contains(Self::MICROPHONE)
    }

    pub fn soundshare(self) -> bool {
        self.contains(Self::SOUNDSHARE)
    }

    pub fn priority(self) -> bool {
        self.contains(Self::PRIORITY)
    }
}

impl<'de> Deserialize<'de> for SpeakingState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_bits_truncate(u8::deserialize(deserializer)?))
    }
}

impl Serialize for SpeakingState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.bits())
    }
}
