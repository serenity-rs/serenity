#[derive(Deserialize)]
pub struct DcaMetadata {
    opus: OpusInfo,
}

#[derive(Deserialize)]
struct OpusInfo {
    /// Bitrate per second
    abr: u32,
    /// Number of channels
    channels: u8,
    /// Frame size in bytes
    frame_size: u32,
    /// Sample rate in Hz
    sample_rate: u32,
    /// Whether or not variable bitrate encoding is used
    vbr: bool,
}

impl DcaMetadata {
    pub fn is_stereo(&self) -> bool {
        self.opus.channels == 2
    }
}