/// Information used in audio frame detection.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub header_len: usize,
    pub frame_len: usize,
}
