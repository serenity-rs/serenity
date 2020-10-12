use super::*;
use std::{
    fmt::{Debug, Error as FormatError, Formatter},
    fs::File,
    io::{
        BufReader,
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
    Pipe(BufReader<ChildContainer>),
    Memory(Catcher<Box<Reader>>),
    Compressed(TxCatcher<Box<Input>, OpusCompressor>),
    Restartable(Restartable),
    File(BufReader<File>),
    Extension(Box<dyn Read + Send>),
    ExtensionSeek(Box<dyn ReadSeek + Send>),
}

impl Reader {
    pub(crate) fn is_seekable(&self) -> bool {
        use Reader::*;

        match self {
            Restartable(_) | Compressed(_) | Memory(_) => true,
            Extension(_) => false,
            ExtensionSeek(_) => true,
            _ => false,
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
            Extension(_) => "Extension".to_string(),
            ExtensionSeek(_) => "ExtensionSeek".to_string(),
        };
        f.debug_tuple("Reader").field(&field).finish()
    }
}

/// Fusion trait for custom input sources which allow seeking.
pub trait ReadSeek {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize>;

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
