/// Information used in audio frame detection.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    /// Length of this frame's header, in bytes.
    pub header_len: usize,
    /// Payload length, in bytes.
    pub frame_len: usize,
}
