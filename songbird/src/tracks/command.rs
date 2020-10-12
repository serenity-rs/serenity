use super::*;
use crate::events::EventData;
use std::time::Duration;
use tokio::sync::oneshot::Sender as OneshotSender;

/// A request from external code using an [`TrackHandle`] to modify
/// or act upon an [`Track`] object.
///
/// [`Track`]: struct.Track.html
/// [`TrackHandle`]: struct.TrackHandle.html
pub enum TrackCommand {
    Play,
    Pause,
    Stop,
    Volume(f32),
    Seek(Duration),
    AddEvent(EventData),
    Do(Box<dyn FnOnce(&mut Track) + Send + Sync + 'static>),
    Request(OneshotSender<Box<TrackState>>),
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
