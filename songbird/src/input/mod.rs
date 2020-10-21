//! Raw audio input data streams and sources.
//!
//! [`Input`] is handled in Songbird by combining metadata with:
//!  * A 48kHz audio bytestream, via [`Reader`],
//!  * A [`Container`] describing the framing mechanism of the bytestream,
//!  * A [`Codec`], defining the format of audio frames.
//!
//! When used as a [`Read`], the output bytestream will be a floating-point
//! PCM stream at 48kHz, matching the channel count of the input source.
//!
//! ## Opus frame passthrough.
//! Some sources, such as [`Compressed`] or the output of [`dca`], support
//! direct frame passthrough to the driver. This lets you directly send the
//! audio data you have *without decoding, re-encoding, or mixing*. In many
//! cases, this can greatly reduce the processing/compute cost of the driver.
//!
//! This functionality requires that:
//!  * only one track is active (including paused tracks),
//!  * that track's input supports direct Opus frame reads,
//!  * its [`Input`] [meets the promises described herein](codec/struct.OpusDecoderState.html#structfield.allow_passthrough),
//!  * and that track's volume is set to `1.0`.
//!
//! [`Input`]: struct.Input.html
//! [`Reader`]: reader/enum.Reader.html
//! [`Container`]: enum.Container.html
//! [`Codec`]: codec/enum.Codec.html
//! [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
//! [`Compressed`]: cached/struct.Compressed.html
//! [`dca`]: fn.dca.html

pub mod cached;
mod child;
pub mod codec;
mod container;
mod dca;
pub mod error;
mod ffmpeg_src;
mod metadata;
pub mod reader;
pub mod restartable;
pub mod utils;
mod ytdl_src;

pub use self::{
    child::*,
    codec::{Codec, CodecType},
    container::{Container, Frame},
    dca::dca,
    ffmpeg_src::*,
    metadata::Metadata,
    reader::Reader,
    restartable::Restartable,
    ytdl_src::*,
};

use crate::constants::*;
use audiopus::coder::GenericCtl;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cached::OpusCompressor;
use error::{Error, Result};
use tokio::runtime::Handle;

use std::{
    convert::TryFrom,
    io::{
        self,
        Error as IoError,
        ErrorKind as IoErrorKind,
        Read,
        Result as IoResult,
        Seek,
        SeekFrom,
    },
    mem,
    time::Duration,
};
use tracing::{debug, error};

/// Data and metadata needed to correctly parse a [`Reader`]'s audio bytestream.
///
/// See the [module root] for more information.
///
/// [`Reader`]: enum.Reader.html
/// [module root]: index.html
#[derive(Debug)]
pub struct Input {
    /// Information about the played source.
    pub metadata: Metadata,
    /// Indicates whether `source` is stereo or mono.
    pub stereo: bool,
    /// Underlying audio data bytestream.
    pub reader: Reader,
    /// Decoder used to parse the output of `reader`.
    pub kind: Codec,
    /// Framing strategy needed to identify frames of compressed audio.
    pub container: Container,
    pos: usize,
}

impl Input {
    /// Creates a floating-point PCM Input from a given reader.
    pub fn float_pcm(is_stereo: bool, reader: Reader) -> Input {
        Input {
            metadata: Default::default(),
            stereo: is_stereo,
            reader,
            kind: Codec::FloatPcm,
            container: Container::Raw,
            pos: 0,
        }
    }

    /// Creates a new Input using (at least) the given reader, codec, and container.
    pub fn new(
        stereo: bool,
        reader: Reader,
        kind: Codec,
        container: Container,
        metadata: Option<Metadata>,
    ) -> Self {
        Input {
            metadata: metadata.unwrap_or_default(),
            stereo,
            reader,
            kind,
            container,
            pos: 0,
        }
    }

    /// Returns whether the inner [`Reader`] implements [`Seek`].
    ///
    /// [`Reader`]: reader/enum.Reader.html
    /// [`Seek`]: https://doc.rust-lang.org/std/io/trait.Seek.html
    pub fn is_seekable(&self) -> bool {
        self.reader.is_seekable()
    }

    /// Returns whether the read audio signal is stereo (or mono).
    pub fn is_stereo(&self) -> bool {
        self.stereo
    }

    /// Returns the type of the inner [`Codec`].
    ///
    /// [`Codec`]: codec/enum.Codec.html
    pub fn get_type(&self) -> CodecType {
        (&self.kind).into()
    }

    /// Mixes the output of this stream into a 20ms stereo audio buffer.
    #[inline]
    pub fn mix(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], volume: f32) -> usize {
        match self.add_float_pcm_frame(float_buffer, self.stereo, volume) {
            Some(len) => len,
            None => 0,
        }
    }

    /// Seeks the stream to the given time, if possible.
    ///
    /// Returns the actual time reached.
    pub fn seek_time(&mut self, time: Duration) -> Option<Duration> {
        let future_pos = utils::timestamp_to_byte_count(time, self.stereo);
        Seek::seek(self, SeekFrom::Start(future_pos as u64))
            .ok()
            .map(|a| utils::byte_count_to_timestamp(a as usize, self.stereo))
    }

    fn read_inner(&mut self, buffer: &mut [u8], ignore_decode: bool) -> IoResult<usize> {
        // This implementation of Read converts the input stream
        // to floating point output.
        let sample_len = mem::size_of::<f32>();
        let float_space = buffer.len() / sample_len;
        let mut written_floats = 0;

        // TODO: better decouple codec and container here.
        // this is a little bit backwards, and assumes the bottom cases are always raw...
        let out = match &mut self.kind {
            Codec::Opus(decoder_state) => {
                if matches!(self.container, Container::Raw) {
                    return Err(IoError::new(
                        IoErrorKind::InvalidInput,
                        "Raw container cannot demarcate Opus frames.",
                    ));
                }

                if ignore_decode {
                    // If we're less than one frame away from the end of cheap seeking,
                    // then we must decode to make sure the next starting offset is correct.

                    // Step one: use up the remainder of the frame.
                    let mut aud_skipped =
                        decoder_state.current_frame.len() - decoder_state.frame_pos;

                    decoder_state.frame_pos = 0;
                    decoder_state.current_frame.truncate(0);

                    // Step two: take frames if we can.
                    while buffer.len() - aud_skipped >= STEREO_FRAME_BYTE_SIZE {
                        decoder_state.should_reset = true;

                        let frame = self
                            .container
                            .next_frame_length(&mut self.reader, CodecType::Opus)?;
                        self.reader.consume(frame.frame_len);

                        aud_skipped += STEREO_FRAME_BYTE_SIZE;
                    }

                    Ok(aud_skipped)
                } else {
                    // get new frame *if needed*
                    if decoder_state.frame_pos == decoder_state.current_frame.len() {
                        let mut decoder = decoder_state.decoder.lock();

                        if decoder_state.should_reset {
                            decoder
                                .reset_state()
                                .expect("Critical failure resetting decoder.");
                            decoder_state.should_reset = false;
                        }
                        let frame = self
                            .container
                            .next_frame_length(&mut self.reader, CodecType::Opus)?;

                        let mut opus_data_buffer = [0u8; 4000];

                        decoder_state
                            .current_frame
                            .resize(decoder_state.current_frame.capacity(), 0.0);

                        let seen =
                            Read::read(&mut self.reader, &mut opus_data_buffer[..frame.frame_len])?;

                        let samples = decoder
                            .decode_float(
                                Some(&opus_data_buffer[..seen]),
                                &mut decoder_state.current_frame[..],
                                false,
                            )
                            .unwrap_or(0);

                        decoder_state.current_frame.truncate(2 * samples);
                        decoder_state.frame_pos = 0;
                    }

                    // read from frame which is present.
                    let mut buffer = &mut buffer[..];

                    let start = decoder_state.frame_pos;
                    let to_write = float_space.min(decoder_state.current_frame.len() - start);
                    for val in &decoder_state.current_frame[start..start + float_space] {
                        buffer.write_f32::<LittleEndian>(*val)?;
                    }
                    decoder_state.frame_pos += to_write;
                    written_floats = to_write;

                    Ok(written_floats * mem::size_of::<f32>())
                }
            },
            Codec::Pcm => {
                let mut buffer = &mut buffer[..];
                while written_floats < float_space {
                    if let Ok(signal) = self.reader.read_i16::<LittleEndian>() {
                        buffer.write_f32::<LittleEndian>(f32::from(signal) / 32768.0)?;
                        written_floats += 1;
                    } else {
                        break;
                    }
                }
                Ok(written_floats * mem::size_of::<f32>())
            },
            Codec::FloatPcm => Read::read(&mut self.reader, buffer),
        };

        out.map(|v| {
            self.pos += v;
            v
        })
    }

    fn cheap_consume(&mut self, count: usize) -> IoResult<usize> {
        let mut scratch = [0u8; STEREO_FRAME_BYTE_SIZE * 4];
        let len = scratch.len();
        let mut done = 0;

        loop {
            let read = self.read_inner(&mut scratch[..len.min(count - done)], true)?;
            if read == 0 {
                break;
            }
            done += read;
        }

        Ok(done)
    }

    pub(crate) fn supports_passthrough(&self) -> bool {
        match &self.kind {
            Codec::Opus(state) => state.allow_passthrough,
            _ => false,
        }
    }

    pub(crate) fn read_opus_frame(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        // Called in event of opus passthrough.
        if let Codec::Opus(state) = &mut self.kind {
            // step 1: align to frame.
            self.pos += state.current_frame.len() - state.frame_pos;

            state.frame_pos = 0;
            state.current_frame.truncate(0);

            // step 2: read new header.
            let frame = self
                .container
                .next_frame_length(&mut self.reader, CodecType::Opus)?;

            // step 3: read in bytes.
            self.reader
                .read_exact(&mut buffer[..frame.frame_len])
                .map(|_| {
                    self.pos += STEREO_FRAME_BYTE_SIZE;
                    frame.frame_len
                })
        } else {
            Err(IoError::new(
                IoErrorKind::InvalidInput,
                "Frame passthrough not supported for this file.",
            ))
        }
    }

    pub(crate) fn prep_with_handle(&mut self, handle: Handle) {
        self.reader.prep_with_handle(handle);
    }
}

impl Read for Input {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        self.read_inner(buffer, false)
    }
}

impl Seek for Input {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let mut target = self.pos;
        match pos {
            SeekFrom::Start(pos) => {
                target = pos as usize;
            },
            SeekFrom::Current(rel) => {
                target = target.wrapping_add(rel as usize);
            },
            SeekFrom::End(_pos) => unimplemented!(),
        }

        debug!("Seeking to {:?}", pos);

        (if target == self.pos {
            Ok(0)
        } else if let Some(conversion) = self.container.try_seek_trivial(self.get_type()) {
            let inside_target = (target * conversion) / mem::size_of::<f32>();
            Seek::seek(&mut self.reader, SeekFrom::Start(inside_target as u64)).map(|inner_dest| {
                let outer_dest = ((inner_dest as usize) * mem::size_of::<f32>()) / conversion;
                self.pos = outer_dest;
                outer_dest
            })
        } else if target > self.pos {
            // seek in the next amount, disabling decoding if need be.
            let shift = target - self.pos;
            self.cheap_consume(shift)
        } else {
            // start from scratch, then seek in...
            Seek::seek(
                &mut self.reader,
                SeekFrom::Start(self.container.input_start() as u64),
            )?;

            self.cheap_consume(target)
        })
        .map(|_| self.pos as u64)
    }
}

/// Extension trait to pull frames of audio from a byte source.
pub(crate) trait ReadAudioExt {
    fn add_float_pcm_frame(
        &mut self,
        float_buffer: &mut [f32; STEREO_FRAME_SIZE],
        true_stereo: bool,
        volume: f32,
    ) -> Option<usize>;

    fn consume(&mut self, amt: usize) -> usize
    where
        Self: Sized;
}

impl<R: Read + Sized> ReadAudioExt for R {
    fn add_float_pcm_frame(
        &mut self,
        float_buffer: &mut [f32; STEREO_FRAME_SIZE],
        stereo: bool,
        volume: f32,
    ) -> Option<usize> {
        // IDEA: Read in 8 floats at a time, then use iterator code
        // to gently nudge the compiler into vectorising for us.
        // Max SIMD float32 lanes is 8 on AVX, older archs use a divisor of this
        // e.g., 4.
        const SAMPLE_LEN: usize = mem::size_of::<f32>();
        const FLOAT_COUNT: usize = 512;
        let mut simd_float_bytes = [0u8; FLOAT_COUNT * SAMPLE_LEN];
        let mut simd_float_buf = [0f32; FLOAT_COUNT];

        let mut frame_pos = 0;

        // Code duplication here is because unifying these codepaths
        // with a dynamic chunk size is not zero-cost.
        if stereo {
            let mut max_bytes = STEREO_FRAME_BYTE_SIZE;

            while frame_pos < float_buffer.len() {
                let progress = self
                    .read(&mut simd_float_bytes[..max_bytes.min(FLOAT_COUNT * SAMPLE_LEN)])
                    .and_then(|byte_len| {
                        let target = byte_len / SAMPLE_LEN;
                        (&simd_float_bytes[..byte_len])
                            .read_f32_into::<LittleEndian>(&mut simd_float_buf[..target])
                            .map(|_| target)
                    })
                    .map(|f32_len| {
                        let new_pos = frame_pos + f32_len;
                        for (el, new_el) in float_buffer[frame_pos..new_pos]
                            .iter_mut()
                            .zip(&simd_float_buf[..f32_len])
                        {
                            *el += volume * new_el;
                        }
                        (new_pos, f32_len)
                    });

                match progress {
                    Ok((new_pos, delta)) => {
                        frame_pos = new_pos;
                        max_bytes -= delta * SAMPLE_LEN;

                        if delta == 0 {
                            break;
                        }
                    },
                    Err(ref e) =>
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            error!("EOF unexpectedly: {:?}", e);
                            Some(frame_pos)
                        } else {
                            error!("Input died unexpectedly: {:?}", e);
                            None
                        },
                }
            }
        } else {
            let mut max_bytes = MONO_FRAME_BYTE_SIZE;

            while frame_pos < float_buffer.len() {
                let progress = self
                    .read(&mut simd_float_bytes[..max_bytes.min(FLOAT_COUNT * SAMPLE_LEN)])
                    .and_then(|byte_len| {
                        let target = byte_len / SAMPLE_LEN;
                        (&simd_float_bytes[..byte_len])
                            .read_f32_into::<LittleEndian>(&mut simd_float_buf[..target])
                            .map(|_| target)
                    })
                    .map(|f32_len| {
                        let new_pos = frame_pos + (2 * f32_len);
                        for (els, new_el) in float_buffer[frame_pos..new_pos]
                            .chunks_exact_mut(2)
                            .zip(&simd_float_buf[..f32_len])
                        {
                            let sample = volume * new_el;
                            els[0] += sample;
                            els[1] += sample;
                        }
                        (new_pos, f32_len)
                    });

                match progress {
                    Ok((new_pos, delta)) => {
                        frame_pos = new_pos;
                        max_bytes -= delta * SAMPLE_LEN;

                        if delta == 0 {
                            break;
                        }
                    },
                    Err(ref e) =>
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(frame_pos)
                        } else {
                            error!("Input died unexpectedly: {:?}", e);
                            None
                        },
                }
            }
        }

        Some(frame_pos * SAMPLE_LEN)
    }

    fn consume(&mut self, amt: usize) -> usize {
        io::copy(&mut self.by_ref().take(amt as u64), &mut io::sink()).unwrap_or(0) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn float_pcm_input_unchanged_mono() {
        let data = make_sine(50 * MONO_FRAME_SIZE, false);
        let mut input = Input::new(
            false,
            data.clone().into(),
            Codec::FloatPcm,
            Container::Raw,
            None,
        );

        let mut out_vec = vec![];

        let len = input.read_to_end(&mut out_vec).unwrap();
        assert_eq!(out_vec[..len], data[..]);
    }

    #[test]
    fn float_pcm_input_unchanged_stereo() {
        let data = make_sine(50 * MONO_FRAME_SIZE, true);
        let mut input = Input::new(
            true,
            data.clone().into(),
            Codec::FloatPcm,
            Container::Raw,
            None,
        );

        let mut out_vec = vec![];

        let len = input.read_to_end(&mut out_vec).unwrap();
        assert_eq!(out_vec[..len], data[..]);
    }

    #[test]
    fn pcm_input_becomes_float_mono() {
        let data = make_pcm_sine(50 * MONO_FRAME_SIZE, false);
        let mut input = Input::new(false, data.clone().into(), Codec::Pcm, Container::Raw, None);

        let mut out_vec = vec![];
        let len = input.read_to_end(&mut out_vec).unwrap();

        let mut i16_window = &data[..];
        let mut float_window = &out_vec[..];

        while i16_window.len() != 0 {
            let before = i16_window.read_i16::<LittleEndian>().unwrap() as f32;
            let after = float_window.read_f32::<LittleEndian>().unwrap();

            let diff = (before / 32768.0) - after;

            assert!(diff.abs() < f32::EPSILON);
        }
    }

    #[test]
    fn pcm_input_becomes_float_stereo() {
        let data = make_pcm_sine(50 * MONO_FRAME_SIZE, true);
        let mut input = Input::new(true, data.clone().into(), Codec::Pcm, Container::Raw, None);

        let mut out_vec = vec![];
        let len = input.read_to_end(&mut out_vec).unwrap();

        let mut i16_window = &data[..];
        let mut float_window = &out_vec[..];

        while i16_window.len() != 0 {
            let before = i16_window.read_i16::<LittleEndian>().unwrap() as f32;
            let after = float_window.read_f32::<LittleEndian>().unwrap();

            let diff = (before / 32768.0) - after;

            assert!(diff.abs() < f32::EPSILON);
        }
    }
}
