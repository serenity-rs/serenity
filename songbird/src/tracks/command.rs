use super::*;
use crate::events::EventData;
use std::time::Duration;
use tokio::sync::oneshot::Sender as OneshotSender;

/// A request from external code using a [`TrackHandle`] to modify
/// or act upon an [`Track`] object.
///
/// [`Track`]: struct.Track.html
/// [`TrackHandle`]: struct.TrackHandle.html
pub enum TrackCommand {
    /// Set the track's play_mode to play/resume.
    Play,
    /// Set the track's play_mode to pause.
    Pause,
    /// Stop the target track. This cannot be undone.
    Stop,
    /// Set the track's volume.
    Volume(f32),
    /// Seek to the given duration.
    ///
    /// On unsupported input types, this can be fatal.
    Seek(Duration),
    /// Register an event on this track.
    AddEvent(EventData),
    /// Run some closure on this track, with direct access to the core object.
    Do(Box<dyn FnOnce(&mut Track) + Send + Sync + 'static>),
    /// Request a read-only view of this track's state.
    Request(OneshotSender<Box<TrackState>>),
    /// Change the loop count/strategy of this track.
    Loop(LoopState),
}

impl std::fmt::Debug for TrackCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use TrackCommand::*;
        write!(
            f,
            "TrackCommand::{}",
            match self {
                Play => "Play".to_string(),
                Pause => "Pause".to_string(),
                Stop => "Stop".to_string(),
                Volume(vol) => format!("Volume({})", vol),
                Seek(d) => format!("Seek({:?})", d),
                AddEvent(evt) => format!("AddEvent({:?})", evt),
                Do(_f) => "Do([function])".to_string(),
                Request(tx) => format!("Request({:?})", tx),
                Loop(loops) => format!("Loop({:?})", loops),
            }
        )
    }
}
