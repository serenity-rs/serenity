use super::*;

/// State of an [`Track`] object, designed to be passed to event handlers
/// and retrieved remotely via [`TrackHandle::get_info`] or
/// [`TrackHandle::get_info_blocking`].
///
/// [`Track`]: struct.Track.html
/// [`TrackHandle::get_info`]: struct.TrackHandle.html#method.get_info
/// [`TrackHandle::get_info_blocking`]: struct.TrackHandle.html#method.get_info_blocking
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct TrackState {
    pub playing: PlayMode,
    pub volume: f32,
    pub position: Duration,
    pub play_time: Duration,
    pub loops: LoopState,
}

impl TrackState {
    pub(crate) fn step_frame(&mut self) {
        self.position += TIMESTEP_LENGTH;
        self.play_time += TIMESTEP_LENGTH;
    }
}
