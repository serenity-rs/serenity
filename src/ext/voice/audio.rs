use ::model::UserId;

/// A readable audio source.
pub trait AudioSource: Send {
    fn is_stereo(&mut self) -> bool;

    fn read_frame(&mut self, buffer: &mut [i16]) -> Option<usize>;
}

/// A receiver for incoming audio.
pub trait AudioReceiver: Send {
    fn speaking_update(&mut self, ssrc: u32, user_id: &UserId, speaking: bool);

    fn voice_packet(&mut self, ssrc: u32, sequence: u16, timestamp: u32, stereo: bool, data: &[i16]);
}
