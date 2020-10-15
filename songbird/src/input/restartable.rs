//! A source which supports seeking by recreating its input stream.
//!
//! This is intended for use with single-use audio tracks which
//! may require looping or seeking, but where additional memory
//! cannot be spared. Forward seeks will drain the track until reaching
//! the desired timestamp.
//!
//! Restarting occurs by temporarily pausing the track, running the restart
//! mechanism, and then passing the handle back to the mixer thread. Until
//! success/failure is confirmed, the track produces silence.

use super::*;
use flume::{Receiver, TryRecvError};
use futures::executor;
use std::{
    ffi::OsStr,
    fmt::{Debug, Error as FormatError, Formatter},
    io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Seek, SeekFrom},
    result::Result as StdResult,
    time::Duration,
};

type Recreator = Box<dyn Restart + Send + 'static>;
type RecreateChannel = Receiver<Result<(Box<Input>, Recreator)>>;

/// A wrapper around a method to create a new [`Input`] which
/// seeks backward by recreating the source.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`Compressed`]
/// and [`Memory`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with single-use audio tracks which
/// may require looping or seeking, but where additional memory
/// cannot be spared. Forward seeks will drain the track until reaching
/// the desired timestamp.
///
/// [`Input`]: struct.Input.html
/// [`Memory`]: cached/struct.Memory.html
/// [`Compressed`]: cached/struct.Compressed.html
pub struct Restartable {
    async_handle: Option<Handle>,
    awaiting_source: Option<RecreateChannel>,
    position: usize,
    recreator: Option<Recreator>,
    source: Box<Input>,
}

impl Restartable {
    /// Create a new source, which can be restarted using a `recreator` function.
    pub fn new(mut recreator: impl Restart + Send + 'static) -> Result<Self> {
        recreator.call_restart(None).map(move |source| Self {
            async_handle: None,
            awaiting_source: None,
            position: 0,
            recreator: Some(Box::new(recreator)),
            source: Box::new(source),
        })
    }

    /// Create a new restartable ffmpeg source for a local file.
    pub fn ffmpeg<P: AsRef<OsStr> + Send + Clone + 'static>(path: P) -> Result<Self> {
        Self::new(FfmpegRestarter { path })
    }

    /// Create a new restartable ytdl source.
    ///
    /// The cost of restarting and seeking will probably be *very* high:
    /// expect a pause if you seek backwards.
    pub fn ytdl<P: AsRef<str> + Send + Clone + 'static>(uri: P) -> Result<Self> {
        Self::new(move |time: Option<Duration>| {
            if let Some(time) = time {
                let ts = format!("{}.{}", time.as_secs(), time.subsec_millis());

                executor::block_on(_ytdl(uri.as_ref(), &["-ss", &ts]))
            } else {
                executor::block_on(ytdl(uri.as_ref()))
            }
        })
    }

    /// Create a new restartable ytdl source, using the first result of a youtube search.
    ///
    /// The cost of restarting and seeking will probably be *very* high:
    /// expect a pause if you seek backwards.
    pub fn ytdl_search(name: &str) -> Result<Self> {
        Self::ytdl(format!("ytsearch1:{}", name))
    }

    pub(crate) fn prep_with_handle(&mut self, handle: Handle) {
        self.async_handle = Some(handle);
    }
}

/// Trait used to create an instance of a [`Reader`] at instantiation and when
/// a backwards seek is needed.
///
/// Many closures derive this automatically.
///
/// [`Reader`]: ../reader/enum.Reader.html
pub trait Restart {
    /// Tries to create a replacement source.
    fn call_restart(&mut self, time: Option<Duration>) -> Result<Input>;
}

struct FfmpegRestarter<P>
where
    P: AsRef<OsStr> + Send,
{
    path: P,
}

impl<P> Restart for FfmpegRestarter<P>
where
    P: AsRef<OsStr> + Send,
{
    fn call_restart(&mut self, time: Option<Duration>) -> Result<Input> {
        executor::block_on(async {
            if let Some(time) = time {
                let is_stereo = is_stereo(self.path.as_ref())
                    .await
                    .unwrap_or_else(|_e| (false, Default::default()));
                let stereo_val = if is_stereo.0 { "2" } else { "1" };

                let ts = format!("{}.{}", time.as_secs(), time.subsec_millis());
                _ffmpeg_optioned(
                    self.path.as_ref(),
                    &["-ss", &ts],
                    &[
                        "-f",
                        "s16le",
                        "-ac",
                        stereo_val,
                        "-ar",
                        "48000",
                        "-acodec",
                        "pcm_f32le",
                        "-",
                    ],
                    Some(is_stereo),
                )
                .await
            } else {
                ffmpeg(self.path.as_ref()).await
            }
        })
    }
}

impl<P> Restart for P
where
    P: FnMut(Option<Duration>) -> Result<Input> + Send + 'static,
{
    fn call_restart(&mut self, time: Option<Duration>) -> Result<Input> {
        (self)(time)
    }
}

impl Debug for Restartable {
    fn fmt(&self, f: &mut Formatter<'_>) -> StdResult<(), FormatError> {
        f.debug_struct("Restartable")
            .field("async_handle", &self.async_handle)
            .field("awaiting_source", &self.awaiting_source)
            .field("position", &self.position)
            .field("recreator", &"<fn>")
            .field("source", &self.source)
            .finish()
    }
}

impl From<Restartable> for Input {
    fn from(mut src: Restartable) -> Self {
        let kind = src.source.kind.clone();
        let meta = Some(src.source.metadata.take());
        let stereo = src.source.stereo;
        let container = src.source.container;
        Input::new(stereo, Reader::Restartable(src), kind, container, meta)
    }
}

// How do these work at a high level?
// If you need to restart, send a request to do this to the async context.
// if a request is pending, then just output all zeroes.

impl Read for Restartable {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        let (out_val, march_pos, remove_async) = if let Some(chan) = &self.awaiting_source {
            match chan.try_recv() {
                Ok(Ok((new_source, recreator))) => {
                    self.source = new_source;
                    self.recreator = Some(recreator);

                    (Read::read(&mut self.source, buffer), true, true)
                },
                Ok(Err(source_error)) => {
                    let e = Err(IoError::new(
                        IoErrorKind::UnexpectedEof,
                        format!("Failed to create new reader: {:?}.", source_error),
                    ));
                    (e, false, true)
                },
                Err(TryRecvError::Empty) => {
                    // Output all zeroes.
                    for el in buffer.iter_mut() {
                        *el = 0;
                    }
                    (Ok(buffer.len()), false, false)
                },
                Err(_) => {
                    let e = Err(IoError::new(
                        IoErrorKind::UnexpectedEof,
                        "Failed to create new reader: dropped.",
                    ));
                    (e, false, true)
                },
            }
        } else {
            // already have a good, valid source.
            (Read::read(&mut self.source, buffer), true, false)
        };

        if remove_async {
            self.awaiting_source = None;
        }

        if march_pos {
            out_val.map(|a| {
                self.position += a;
                a
            })
        } else {
            out_val
        }
    }
}

impl Seek for Restartable {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let _local_pos = self.position as u64;

        use SeekFrom::*;
        match pos {
            Start(offset) => {
                let stereo = self.source.stereo;
                let _current_ts = utils::byte_count_to_timestamp(self.position, stereo);
                let offset = offset as usize;

                if offset < self.position {
                    // We're going back in time.
                    if let Some(handle) = self.async_handle.as_ref() {
                        let (tx, rx) = flume::bounded(1);

                        self.awaiting_source = Some(rx);

                        let recreator = self.recreator.take();

                        if let Some(mut rec) = recreator {
                            handle.spawn(async move {
                                let ret_val = rec.call_restart(Some(
                                    utils::byte_count_to_timestamp(offset, stereo),
                                ));

                                let _ = tx.send(ret_val.map(Box::new).map(|v| (v, rec)));
                            });
                        } else {
                            return Err(IoError::new(
                                IoErrorKind::Interrupted,
                                "Previous seek in progress.",
                            ));
                        }

                        self.position = offset;
                    } else {
                        return Err(IoError::new(
                            IoErrorKind::Interrupted,
                            "Cannot safely call seek until provided an async context handle.",
                        ));
                    }
                } else {
                    self.position += self.source.consume(offset - self.position);
                }

                Ok(offset as u64)
            },
            End(_offset) => Err(IoError::new(
                IoErrorKind::InvalidInput,
                "End point for Restartables is not known.",
            )),
            Current(_offset) => unimplemented!(),
        }
    }
}
