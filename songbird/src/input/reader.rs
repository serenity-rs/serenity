//! Raw handlers for input bytestreams.

use super::*;
use std::{
    fmt::{Debug, Error as FormatError, Formatter},
    fs::File,
    io::{
        BufReader,
        Cursor,
        Error as IoError,
        ErrorKind as IoErrorKind,
        Read,
        Result as IoResult,
        Seek,
        SeekFrom,
    },
    result::Result as StdResult,
};
use streamcatcher::{Catcher, TxCatcher};

/// Usable data/byte sources for an audio stream.
///
/// Users may define their own data sources using [`Extension`]
/// and [`ExtensionSeek`].
///
/// [`Extension`]: #variant.Extension
/// [`ExtensionSeek`]: #variant.ExtensionSeek
pub enum Reader {
    /// Piped output of another program (i.e., [`ffmpeg`]).
    ///
    /// Does not support seeking.
    ///
    /// [`ffmpeg`]: ../fn.ffmpeg.html
    Pipe(BufReader<ChildContainer>),
    /// A cached, raw in-memory store, provided by Songbird.
    ///
    /// Supports seeking.
    Memory(Catcher<Box<Reader>>),
    /// A cached, Opus-compressed in-memory store, provided by Songbird.
    ///
    /// Supports seeking.
    Compressed(TxCatcher<Box<Input>, OpusCompressor>),
    /// A source which supports seeking by recreating its inout stream.
    ///
    /// Supports seeking.
    Restartable(Restartable),
    /// A source contained in a local file.
    ///
    /// Supports seeking.
    File(BufReader<File>),
    /// A source contained as an array in memory.
    ///
    /// Supports seeking.
    Vec(Cursor<Vec<u8>>),
    /// A basic user-provided source.
    ///
    /// Does not support seeking.
    Extension(Box<dyn Read + Send>),
    /// A user-provided source which also implements [`Seek`].
    ///
    /// Supports seeking.
    ///
    /// [`Seek`]: https://doc.rust-lang.org/std/io/trait.Seek.html
    ExtensionSeek(Box<dyn ReadSeek + Send>),
}

impl Reader {
    /// Returns whether the given source implements [`Seek`].
    ///
    /// [`Seek`]: https://doc.rust-lang.org/std/io/trait.Seek.html
    pub fn is_seekable(&self) -> bool {
        use Reader::*;
        match self {
            Restartable(_) | Compressed(_) | Memory(_) => true,
            Extension(_) => false,
            ExtensionSeek(_) => true,
            _ => false,
        }
    }

    #[allow(clippy::single_match)]
    pub(crate) fn prep_with_handle(&mut self, handle: Handle) {
        use Reader::*;
        match self {
            Restartable(r) => r.prep_with_handle(handle),
            _ => {},
        }
    }
}

impl Read for Reader {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        use Reader::*;
        match self {
            Pipe(a) => Read::read(a, buffer),
            Memory(a) => Read::read(a, buffer),
            Compressed(a) => Read::read(a, buffer),
            Restartable(a) => Read::read(a, buffer),
            File(a) => Read::read(a, buffer),
            Vec(a) => Read::read(a, buffer),
            Extension(a) => a.read(buffer),
            ExtensionSeek(a) => a.read(buffer),
        }
    }
}

impl Seek for Reader {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        use Reader::*;
        match self {
            Pipe(_) | Extension(_) => Err(IoError::new(
                IoErrorKind::InvalidInput,
                "Seeking not supported on Reader of this type.",
            )),
            Memory(a) => Seek::seek(a, pos),
            Compressed(a) => Seek::seek(a, pos),
            File(a) => Seek::seek(a, pos),
            Restartable(a) => Seek::seek(a, pos),
            Vec(a) => Seek::seek(a, pos),
            ExtensionSeek(a) => a.seek(pos),
        }
    }
}

impl Debug for Reader {
    fn fmt(&self, f: &mut Formatter<'_>) -> StdResult<(), FormatError> {
        use Reader::*;
        let field = match self {
            Pipe(a) => format!("{:?}", a),
            Memory(a) => format!("{:?}", a),
            Compressed(a) => format!("{:?}", a),
            Restartable(a) => format!("{:?}", a),
            File(a) => format!("{:?}", a),
            Vec(a) => format!("{:?}", a),
            Extension(_) => "Extension".to_string(),
            ExtensionSeek(_) => "ExtensionSeek".to_string(),
        };
        f.debug_tuple("Reader").field(&field).finish()
    }
}

impl From<Vec<u8>> for Reader {
    fn from(val: Vec<u8>) -> Reader {
        Reader::Vec(Cursor::new(val))
    }
}

/// Fusion trait for custom input sources which allow seeking.
pub trait ReadSeek {
    /// See [`Read::read`].
    ///
    /// [`Read::read`]: https://doc.rust-lang.org/nightly/std/io/trait.Read.html#tymethod.read
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize>;
    /// See [`Seek::seek`].
    ///
    /// [`Seek::seek`]: https://doc.rust-lang.org/nightly/std/io/trait.Seek.html#tymethod.seek
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64>;
}

impl Read for dyn ReadSeek {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        ReadSeek::read(self, buf)
    }
}

impl Seek for dyn ReadSeek {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        ReadSeek::seek(self, pos)
    }
}

impl<R: Read + Seek> ReadSeek for R {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        Read::read(self, buf)
    }

    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        Seek::seek(self, pos)
    }
}
