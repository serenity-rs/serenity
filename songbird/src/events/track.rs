#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Track events correspond to certain actions or changes
/// of state, such as a track finishing, looping, or being
/// manually stopped. Voice core events occur on receipt of
/// voice packets and telemetry.
///
/// Track events persist while the `action` in [`EventData`]
/// returns `None`.
///
/// [`EventData`]: struct.EventData.html
pub enum TrackEvent {
    /// The attached track has ended.
    End,
    /// The attached track has looped.
    Loop,
}
