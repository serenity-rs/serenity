#[derive(Debug, Deserialize)]
pub struct DcaMetadata {
    opus: OpusInfo,
}

#[derive(Debug, Deserialize)]
struct OpusInfo {
    /// Number of channels
    channels: u8,
}

impl DcaMetadata {
    pub fn is_stereo(&self) -> bool { self.opus.channels == 2 }
}
