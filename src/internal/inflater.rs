use std::convert::TryInto;

use flate2::{Decompress, DecompressError, FlushDecompress};
use log::trace;

const ZLIB_SUFFIX: [u8; 4] = [0x00, 0x00, 0xff, 0xff];
const INTERNAL_BUFFER_SIZE: usize = 32 * 1024;

pub struct Inflater {
    decompress: Decompress,
    compressed: Vec<u8>,
    internal_buffer: Vec<u8>,
    buffer: Vec<u8>,
    countdown_to_resize: u8,
}

impl Inflater {
    pub fn new() -> Self {
        Self {
            decompress: Decompress::new(true),
            compressed: Vec::new(),
            internal_buffer: Vec::with_capacity(INTERNAL_BUFFER_SIZE),
            buffer: Vec::with_capacity(32 * 1024),
            countdown_to_resize: u8::max_value(),
        }
    }

    pub fn extend(&mut self, slice: &[u8]) {
        self.compressed.extend_from_slice(&slice);
    }

    pub fn msg(&mut self) -> Result<Option<&[u8]>, DecompressError> {
        let length = self.compressed.len();
        if length >= 4 && self.compressed[(length - 4)..] == ZLIB_SUFFIX {
            // There is a payload to be decompressed.
            let before = self.decompress.total_in();
            let mut offset = 0;
            loop {
                self.internal_buffer.clear();

                self.decompress.decompress_vec(
                    &self.compressed[offset..],
                    &mut self.internal_buffer,
                    FlushDecompress::Sync,
                )?;

                offset = (self.decompress.total_in() - before)
                    .try_into()
                    .unwrap_or(0);
                self.buffer.extend_from_slice(&self.internal_buffer[..]);
                if self.internal_buffer.len() < self.internal_buffer.capacity()
                    || offset > self.compressed.len()
                {
                    break;
                }
            }

            trace!("in:out: {}:{}", self.compressed.len(), self.buffer.len());
            self.compressed.clear();

            #[allow(clippy::cast_precision_loss)]
            {
                // To get around the u64 → f64 precision loss lint
                // it does really not matter that it happens here
                trace!(
                    "Data saved: {}KiB ({:.2}%)",
                    ((self.decompress.total_out() - self.decompress.total_in()) / 1024),
                    ((self.decompress.total_in() as f64) / (self.decompress.total_out() as f64)
                        * 100.0)
                );
            }
            trace!("Capacity: {}", self.buffer.capacity());
            Ok(Some(&self.buffer))
        } else {
            // Received a partial payload.
            Ok(None)
        }
    }

    // Clear the buffer, and shrink it if it has more space
    // enough to grow the length more than 4 times.
    pub fn clear(&mut self) {
        self.countdown_to_resize -= 1;

        // Only shrink capacity if it is less than 4
        // times the size, this is to prevent too
        // frequent shrinking.
        let cap = self.buffer.capacity();
        if self.countdown_to_resize == 0 && self.buffer.len() < cap * 4 {
            // When shrink_to goes stable use that on the following line.
            // https://github.com/rust-lang/rust/issues/56431
            self.compressed.shrink_to_fit();
            self.buffer.shrink_to_fit();
            trace!("compressed: {}", self.compressed.capacity());
            trace!("buffer: {}", self.buffer.capacity());
            self.countdown_to_resize = u8::max_value();
        }
        self.compressed.clear();
        self.internal_buffer.clear();
        self.buffer.clear();
    }

    // Reset the inflater
    pub fn reset(&mut self) {
        self.decompress.reset(true);
        self.compressed.clear();
        self.internal_buffer.clear();
        self.buffer.clear();
        self.countdown_to_resize = u8::max_value();
    }
}
