use crate::{
    driver::Driver,
    events::{Event, EventContext, EventData, EventHandler, TrackEvent},
    input::Input,
    tracks::{self, Track, TrackHandle, TrackResult},
};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::{collections::VecDeque, sync::Arc};
use tracing::{info, warn};

#[derive(Default)]
/// A simple queue for several audio sources, designed to
/// play in sequence.
///
/// This makes use of [`TrackEvent`]s to determine when the current
/// song or audio file has finished before playing the next entry.
///
/// `examples/e16_voice_events` demonstrates how a user might manage,
/// track and use this to run a song queue in many guilds in parallel.
/// This code is trivial to extend if extra functionality is needed.
///
/// # Example
///
/// ```rust,no_run
/// use songbird::{
///     driver::Driver,
///     id::GuildId,
///     ffmpeg,
///     tracks::{create_player, TrackQueue},
/// };
/// use std::collections::HashMap;
///
/// # async {
/// let guild = GuildId(0);
/// // A Call is also valid here!
/// let mut driver: Driver = Default::default();
///
/// let mut queues: HashMap<GuildId, TrackQueue> = Default::default();
///
/// let source = ffmpeg("../audio/my-favourite-song.mp3")
///     .await
///     .expect("This might fail: handle this error!");
///
/// // We need to ensure that this guild has a TrackQueue created for it.
/// let queue = queues.entry(guild)
///     .or_default();
///
/// // Queueing a track is this easy!
/// queue.add_source(source, &mut driver);
/// # };
/// ```

///
/// [`TrackEvent`]: ../events/enum.TrackEvent.html
pub struct TrackQueue {
    // NOTE: the choice of a parking lot mutex is quite deliberate
    inner: Arc<Mutex<TrackQueueCore>>,
}

#[derive(Default)]
/// Inner portion of a [`TrackQueue`].
///
/// This abstracts away thread-safety from the user,
/// and offers a convenient location to store further state if required.
///
/// [`TrackQueue`]: struct.TrackQueue.html
struct TrackQueueCore {
    tracks: VecDeque<TrackHandle>,
}

struct QueueHandler {
    remote_lock: Arc<Mutex<TrackQueueCore>>,
}

#[async_trait]
impl EventHandler for QueueHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let mut inner = self.remote_lock.lock();
        let _old = inner.tracks.pop_front();

        info!("Queued track ended: {:?}.", ctx);
        info!("{} tracks remain.", inner.tracks.len());

        // If any audio files die unexpectedly, then keep going until we
        // find one which works, or we run out.
        let mut keep_looking = true;
        while keep_looking && !inner.tracks.is_empty() {
            if let Some(new) = inner.tracks.front() {
                keep_looking = new.play().is_err();

                // Discard files which cannot be used for whatever reason.
                if keep_looking {
                    warn!("Track in Queue couldn't be played...");
                    let _ = inner.tracks.pop_front();
                }
            }
        }

        None
    }
}

impl TrackQueue {
    /// Create a new, empty, track queue.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TrackQueueCore {
                tracks: VecDeque::new(),
            })),
        }
    }

    /// Adds an audio source to the queue, to be played in the channel managed by `handler`.
    pub fn add_source(&self, source: Input, handler: &mut Driver) {
        let (audio, audio_handle) = tracks::create_player(source);
        self.add(audio, audio_handle, handler);
    }

    /// Adds a [`Track`] object to the queue, to be played in the channel managed by `handler`.
    ///
    /// This is used with [`voice::create_player`] if additional configuration or event handlers
    /// are required before enqueueing the audio track.
    ///
    /// [`Track`]: struct.Track.html
    /// [`voice::create_player`]: fn.create_player.html
    pub fn add(&self, mut track: Track, track_handle: TrackHandle, handler: &mut Driver) {
        info!("Track added to queue.");
        let remote_lock = self.inner.clone();
        let mut inner = self.inner.lock();

        if !inner.tracks.is_empty() {
            track.pause();
        }

        track
            .events
            .as_mut()
            .expect("Queue inspecting EventStore on new Track: did not exist.")
            .add_event(
                EventData::new(Event::Track(TrackEvent::End), QueueHandler { remote_lock }),
                track.position,
            );

        handler.play(track);
        inner.tracks.push_back(track_handle);
    }

    /// Returns the number of tracks currently in the queue.
    pub fn len(&self) -> usize {
        let inner = self.inner.lock();

        inner.tracks.len()
    }

    /// Returns whether there are no tracks currently in the queue.
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();

        inner.tracks.is_empty()
    }

    /// Pause the track at the head of the queue.
    pub fn pause(&self) -> TrackResult {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.pause()
        } else {
            Ok(())
        }
    }

    /// Resume the track at the head of the queue.
    pub fn resume(&self) -> TrackResult {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.play()
        } else {
            Ok(())
        }
    }

    /// Stop the currently playing track, and clears the queue.
    pub fn stop(&self) -> TrackResult {
        let mut inner = self.inner.lock();

        let out = inner.stop_current();

        inner.tracks.clear();

        out
    }

    /// Skip to the next track in the queue, if it exists.
    pub fn skip(&self) -> TrackResult {
        let inner = self.inner.lock();

        inner.stop_current()
    }
}

impl TrackQueueCore {
    /// Skip to the next track in the queue, if it exists.
    fn stop_current(&self) -> TrackResult {
        if let Some(handle) = self.tracks.front() {
            handle.stop()
        } else {
            Ok(())
        }
    }
}
