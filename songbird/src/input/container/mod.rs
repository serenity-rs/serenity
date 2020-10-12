mod frame;

pub use frame::*;

use super::CodecType;
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fmt::Debug,
    io::{Read, Result as IoResult},
    mem,
};

/// Marker for decoding framed input files.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum Container {
    Raw,
    Dca { first_frame: usize },
}

impl Container {
    pub fn next_frame_length(
        &mut self,
        mut reader: impl Read,
        input: CodecType,
    ) -> IoResult<Frame> {
        use Container::*;

        match self {
            Raw => Ok(Frame {
                header_len: 0,
                frame_len: input.sample_len(),
            }),
            Dca { .. } => reader.read_i16::<LittleEndian>().map(|frame_len| Frame {
                header_len: mem::size_of::<i16>(),
                frame_len: frame_len.max(0) as usize,
            }),
        }
    }

    pub fn try_seek_trivial(&self, input: CodecType) -> Option<usize> {
        use Container::*;

        match self {
            Raw => Some(input.sample_len()),
            _ => None,
        }
    }

    pub fn input_start(&self) -> usize {
        use Container::*;

        match self {
            Raw => 0,
            Dca { first_frame } => *first_frame,
        }
    }
}
