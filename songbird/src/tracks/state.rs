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
    /// Play status (e.g., active, paused, stopped) of this track.
    pub playing: PlayMode,
    /// Current volume of this track.
    pub volume: f32,
    /// Current playback position in the source.
    ///
    /// This is altered by loops and seeks
    pub position: Duration,
    /// Total playback time, increasing monotonically.
    pub play_time: Duration,
    /// Remaining loops on this track.
    pub loops: LoopState,
}

impl TrackState {
    pub(crate) fn step_frame(&mut self) {
        self.position += TIMESTEP_LENGTH;
        self.play_time += TIMESTEP_LENGTH;
    }
}
