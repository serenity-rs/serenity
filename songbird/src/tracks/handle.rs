use super::*;
use crate::events::{Event, EventData, EventHandler};
use std::time::Duration;
use tokio::sync::{
    mpsc::{error::SendError, UnboundedSender},
    oneshot,
};

#[derive(Clone, Debug)]
/// Handle for safe control of a [`Track`] track from other threads, outside
/// of the audio mixing and voice handling context.
///
/// Almost all method calls here are fallible; in most cases, this will be because
/// the underlying [`Track`] object has been discarded. Those which aren't refer
/// to immutable properties of the underlying stream.
///
/// [`Track`]: struct.Track.html
pub struct TrackHandle {
    command_channel: UnboundedSender<TrackCommand>,
    seekable: bool,
}

impl TrackHandle {
    /// Creates a new handle, using the given command sink and hint as to whether
    /// the underlying [`Input`] supports seek operations.
    ///
    /// [`Input`]: ../input/struct.Input.html
    pub fn new(command_channel: UnboundedSender<TrackCommand>, seekable: bool) -> Self {
        Self {
            command_channel,
            seekable,
        }
    }

    /// Unpauses an audio track.
    pub fn play(&self) -> TrackResult {
        self.send(TrackCommand::Play)
    }

    /// Pauses an audio track.
    pub fn pause(&self) -> TrackResult {
        self.send(TrackCommand::Pause)
    }

    /// Stops an audio track.
    ///
    /// This is *final*, and will cause the audio context to fire
    /// a [`TrackEvent::End`] event.
    ///
    /// [`TrackEvent::End`]: ../events/enum.TrackEvent.html#variant.End
    pub fn stop(&self) -> TrackResult {
        self.send(TrackCommand::Stop)
    }

    /// Sets the volume of an audio track.
    pub fn set_volume(&self, volume: f32) -> TrackResult {
        self.send(TrackCommand::Volume(volume))
    }

    /// Denotes whether the underlying [`Input`] stream is compatible with arbitrary seeking.
    ///
    /// If this returns `false`, all calls to [`seek`] will fail, and the track is
    /// incapable of looping.
    ///
    /// [`seek`]: #method.seek
    /// [`Input`]: ../input/struct.Input.html
    pub fn is_seekable(&self) -> bool {
        self.seekable
    }

    /// Seeks along the track to the specified position.
    ///
    /// If the underlying [`Input`] does not support this behaviour,
    /// then all calls will fail.
    ///
    /// [`Input`]: ../input/struct.Input.html
    pub fn seek_time(&self, position: Duration) -> TrackResult {
        if self.seekable {
            self.send(TrackCommand::Seek(position))
        } else {
            Err(SendError(TrackCommand::Seek(position)))
        }
    }

    /// Attach an event handler to an audio track. These will receive [`EventContext::Track`].
    ///
    /// Users **must** ensure that no costly work or blocking occurs
    /// within the supplied function or closure. *Taking excess time could prevent
    /// timely sending of packets, causing audio glitches and delays*.
    ///
    /// [`Track`]: struct.Track.html
    /// [`EventContext::Track`]: ../events/enum.EventContext.html#variant.Track
    pub fn add_event<F: EventHandler + 'static>(&self, event: Event, action: F) -> TrackResult {
        let cmd = TrackCommand::AddEvent(EventData::new(event, action));
        if event.is_global_only() {
            Err(SendError(cmd))
        } else {
            self.send(cmd)
        }
    }

    /// Perform an arbitrary action on a raw [`Track`] object.
    ///
    /// Users **must** ensure that no costly work or blocking occurs
    /// within the supplied function or closure. *Taking excess time could prevent
    /// timely sending of packets, causing audio glitches and delays*.
    ///
    /// [`Track`]: struct.Track.html
    pub fn action<F>(&self, action: F) -> TrackResult
    where
        F: FnOnce(&mut Track) + Send + Sync + 'static,
    {
        self.send(TrackCommand::Do(Box::new(action)))
    }

    /// Request playback information and state from the audio context.
    ///
    /// Crucially, the audio thread will respond *at a later time*:
    /// It is up to the user when or how this should be read from the returned channel.
    pub fn get_info(&self) -> TrackQueryResult {
        let (tx, rx) = oneshot::channel();
        self.send(TrackCommand::Request(tx)).map(move |_| rx)
    }

    /// Set an audio track to loop indefinitely.
    pub fn enable_loop(&self) -> TrackResult {
        if self.seekable {
            self.send(TrackCommand::Loop(LoopState::Infinite))
        } else {
            Err(SendError(TrackCommand::Loop(LoopState::Infinite)))
        }
    }

    /// Set an audio track to no longer loop.
    pub fn disable_loop(&self) -> TrackResult {
        if self.seekable {
            self.send(TrackCommand::Loop(LoopState::Finite(0)))
        } else {
            Err(SendError(TrackCommand::Loop(LoopState::Finite(0))))
        }
    }

    /// Set an audio track to loop a set number of times.
    pub fn loop_for(&self, count: usize) -> TrackResult {
        if self.seekable {
            self.send(TrackCommand::Loop(LoopState::Finite(count)))
        } else {
            Err(SendError(TrackCommand::Loop(LoopState::Finite(count))))
        }
    }

    #[inline]
    /// Send a raw command to the [`Track`] object.
    ///
    /// [`Track`]: struct.Track.html
    pub fn send(&self, cmd: TrackCommand) -> TrackResult {
        self.command_channel.send(cmd)
    }
}
