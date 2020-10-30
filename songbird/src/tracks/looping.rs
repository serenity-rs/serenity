#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Looping behaviour for a [`Track`].
///
/// [`Track`]: struct.Track.html
pub enum LoopState {
    /// Track will loop endlessly until loop state is changed or
    /// manually stopped.
    Infinite,

    /// Track will loop `n` more times.
    ///
    /// `Finite(0)` is the `Default`, stopping the track once its [`Input`] ends.
    ///
    /// [`Input`]: ../input/struct.Input.html
    Finite(usize),
}

impl Default for LoopState {
    fn default() -> Self {
        Self::Finite(0)
    }
}
