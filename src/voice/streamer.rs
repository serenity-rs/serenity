use audiopus::{
    Application,
    Channels,
    Error as OpusError,
    ErrorCode as OpusErrorCode,
    coder::GenericCtl,
    coder::Decoder as OpusDecoder,
    coder::Encoder as OpusEncoder,
    Result as OpusResult,
    SampleRate,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::{
    internal::prelude::*,
    prelude::SerenityError,
};
use parking_lot::{
    lock_api::MutexGuard,
    Mutex,
};
use serde_json;
use std::{
    cell::UnsafeCell,
    collections::LinkedList,
    ffi::OsStr,
    io::{
        self,
        BufReader,
        Error as IoError,
        ErrorKind as IoErrorKind,
        Read,
        Result as IoResult,
        Seek,
        SeekFrom,
    },
    mem::{
        self,
        ManuallyDrop,
    },
    ops::{Add, AddAssign, Sub, SubAssign},
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};
use super::{
    AudioType,
    Bitrate,
    DcaError,
    DcaMetadata,
    ReadSeek,
    VoiceError, 
    audio,
    constants::*,
};
use log::{debug, warn};

pub struct ChildContainer(Child);

pub fn child_to_reader<T>(child: Child) -> Reader {
    Reader::Pipe(
        BufReader::with_capacity(
            STEREO_FRAME_SIZE * mem::size_of::<T>() * CHILD_BUFFER_LEN,
            ChildContainer(child),
        )
    )
}

impl Read for ChildContainer {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        self.0.stdout.as_mut().unwrap().read(buffer)
    }
}

impl Drop for ChildContainer {
    fn drop (&mut self) {
        if let Err(e) = self.0.kill() {
            debug!("[Voice] Error awaiting child process: {:?}", e);
        }
    }
}

pub enum Reader {
    Pipe(BufReader<ChildContainer>),
    InMemory(MemorySource),
    Compressed(CompressedSource),
    Restartable(RestartableSource),
    Extension(Box<dyn Read + Send + 'static>),
    ExtensionSeek(Box<dyn ReadSeek + Send + 'static>),
}

impl Reader {
    fn is_seekable(&self) -> bool {
        use Reader::*;

        match self {
            Restartable(_) | Compressed(_) | InMemory(_) => true,
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
            InMemory(a) => Read::read(a, buffer),
            Compressed(a) => Read::read(a, buffer),
            Restartable(a) => Read::read(a, buffer),
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
                "Seeking not supported on Reader of this type.")),
            InMemory(a) => Seek::seek(a, pos),
            Compressed(a) => Seek::seek(a, pos),
            Restartable(a) => Seek::seek(a, pos),
            ExtensionSeek(a) => a.seek(pos),
        }
    }
}

pub struct Input {
    pub stereo: bool,
    pub reader: Reader,
    pub kind: AudioType,
    pub decoder: Option<Arc<Mutex<OpusDecoder>>>,
}

impl Input {
    pub fn float_pcm(is_stereo: bool, reader: Reader) -> Input {
        Input {
            stereo: is_stereo,
            reader,
            kind: AudioType::FloatPcm,
            decoder: None,
        }
    }

    pub fn new(stereo: bool, reader: Reader, kind: AudioType, decoder: Option<Arc<Mutex<OpusDecoder>>>) -> Self {
        Input {
            stereo,
            reader,
            kind,
            decoder,
        }
    }

    pub fn is_seekable(&self) -> bool {
        self.reader.is_seekable()
    }

    pub fn is_stereo(&self) -> bool {
        self.stereo
    }

    pub fn get_type(&self) -> AudioType {
        self.kind
    }

    #[inline]
    pub fn mix(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], volume: f32) -> usize {
        match self.kind {
            AudioType::Opus => unimplemented!(),
                    // if self.reader.decode_and_add_opus_frame(&mut float_buffer, vol).is_some() {
                    //     0 //; opus_frame.len()
                    // } else {
                    //     0
                    // },
            AudioType::Pcm => {
                match self.reader.add_pcm_frame(float_buffer, self.stereo, volume) {
                    Some(len) => len,
                    None => 0,
                }
            },
            AudioType::FloatPcm => {
                match self.reader.add_float_pcm_frame(float_buffer, self.stereo, volume) {
                    Some(len) => len,
                    None => 0,
                }
            },
        }
    }

    // fixme: make this relative.
    pub fn seek_time(&mut self, time: Duration) -> Option<Duration> {
        let future_pos = timestamp_to_byte_count(time, self.stereo);
        Seek::seek(&mut self.reader, SeekFrom::Start(future_pos as u64))
            .ok()
            .map(|a| byte_count_to_timestamp(a as usize, self.stereo))
    }
}

impl Read for Input {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        // This implementation of Read converts the input stream
        // to floating point output.
        let float_space = buffer.len() / mem::size_of::<f32>();
        let mut written_floats = 0;
        // Read::read(&mut self.reader, buffer)
        match self.kind {
            AudioType::Opus => unimplemented!(),
            AudioType::Pcm => {
                //FIXME: probably stifiling an error.
                let mut buffer = &mut buffer[..];
                while written_floats < float_space {
                    if let Ok(signal) = self.reader.read_i16::<LittleEndian>() {
                        buffer.write_f32::<LittleEndian>(f32::from(signal) / 32768.0);
                        written_floats += 1;
                    } else {
                        break;
                    }
                }
                Ok(written_floats)
            },
            AudioType::FloatPcm => {
                Read::read(&mut self.reader, buffer)
            },
        }
    }
}

pub trait ReadAudioExt {
    fn add_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize>;

    fn add_float_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize>;

    fn consume(&mut self, amt: usize) -> usize where Self: Sized;
}

impl<R: Read + Sized> ReadAudioExt for R {
    fn add_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize> {
        // Duplicate this to avoid repeating the stereo check.
        // This should let us unconditionally duplicate samples in the main loop body.
        if true_stereo {
            for (i, float_buffer_element) in float_buffer.iter_mut().enumerate() {
                let raw = match self.read_i16::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };
                let sample = f32::from(raw) / 32768.0;

                *float_buffer_element += sample * volume;
            }
        } else {
            let mut float_index = 0;
            for i in 0..float_buffer.len() / 2 {
                let raw = match self.read_i16::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };
                let sample = volume * f32::from(raw) / 32768.0;

                float_buffer[float_index] += sample;
                float_buffer[float_index+1] += sample;

                float_index += 2;
            }
        }

        Some(float_buffer.len())
    }

    fn add_float_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], stereo: bool, volume: f32) -> Option<usize> {
        if stereo {
            for (i, float_buffer_element) in float_buffer.iter_mut().enumerate() {
                let sample = match self.read_f32::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };

                *float_buffer_element += sample * volume;
            }
        } else {
            let mut float_index = 0;
            for i in 0..float_buffer.len() / 2 {
                let raw = match self.read_f32::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };
                let sample = volume * raw;

                float_buffer[float_index] += sample;
                float_buffer[float_index+1] += sample;

                float_index += 2;
            }
        }

        Some(float_buffer.len())
    }

    fn consume(&mut self, amt: usize) -> usize {
        io::copy(&mut self.by_ref().take(amt as u64), &mut io::sink()).unwrap_or(0) as usize
    }
}

// impl AudioSource for Input {
//     // FIXME: COMPLETELY BROKEN
//     // this assumes DCA exculsively.
//     // DOES NOT WORK FOR OPUS IN THE GENERAL CASE.
//     fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
//         match self.reader.read_i16::<LittleEndian>() {
//             Ok(size) => {
//                 if size <= 0 {
//                     warn!("Invalid opus frame size: {}", size);
//                     return None;
//                 }

//                 let mut frame = Vec::with_capacity(size as usize);

//                 {
//                     let reader = self.reader.by_ref();

//                     if reader.take(size as u64).read_to_end(&mut frame).is_err() {
//                         return None;
//                     }
//                 }

//                 Some(frame)
//             },
//             Err(ref e) => if e.kind() == IoErrorKind::UnexpectedEof {
//                 Some(Vec::new())
//             } else {
//                 None
//             },
//         }
//     }

//     fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], volume: f32) -> Option<usize> {
//         let decoder_lock = self.decoder.as_mut()?.clone();
//         let frame = self.read_opus_frame()?;
//         let mut local_buf = [0f32; 960 * 2];

//         let count = {
//             let mut decoder = decoder_lock.lock();

//             decoder.decode_float(frame.as_slice(), &mut local_buf[..], false).ok()?
//         };

//         for (i, float_buffer_element) in float_buffer.iter_mut().enumerate().take(1920) {
//             *float_buffer_element += local_buf[i] * volume;
//         }

//         Some(count)
//     }
// }


/// Opens an audio file through `ffmpeg` and creates an audio source.
pub fn ffmpeg<P: AsRef<OsStr>>(path: P) -> Result<Input> {
    _ffmpeg(path.as_ref())
}

fn _ffmpeg(path: &OsStr) -> Result<Input> {
    // Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path).unwrap_or(false);
    let stereo_val = if is_stereo { "2" } else { "1" };

    _ffmpeg_optioned(path, &[], &[
        "-f",
        "s16le",
        "-ac",
        stereo_val,
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-",
    ], Some(is_stereo))
}

/// Opens an audio file through `ffmpeg` and creates an audio source, with
/// user-specified arguments to pass to ffmpeg.
///
/// Note that this does _not_ build on the arguments passed by the [`ffmpeg`]
/// function.
///
/// # Examples
///
/// Pass options to create a custom ffmpeg streamer:
///
/// ```rust,no_run
/// use serenity::voice;
///
/// let stereo_val = "2";
///
/// let streamer = voice::ffmpeg_optioned("./some_file.mp3", &[], &[
///     "-f",
///     "s16le",
///     "-ac",
///     stereo_val,
///     "-ar",
///     "48000",
///     "-acodec",
///     "pcm_s16le",
///     "-",
/// ]);
///```
pub fn ffmpeg_optioned<P: AsRef<OsStr>>(
    path: P,
    pre_input_args: &[&str],
    args: &[&str],
) -> Result<Input> {
    _ffmpeg_optioned(path.as_ref(), pre_input_args, args, None)
}

fn _ffmpeg_optioned(
    path: &OsStr,
    pre_input_args: &[&str],
    args: &[&str],
    is_stereo_known: Option<bool>,
) -> Result<Input> {
    let is_stereo = is_stereo_known
        .or_else(|| is_stereo(path).ok())
        .unwrap_or(false);

    let command = Command::new("ffmpeg")
        .args(pre_input_args)
        .arg("-i")
        .arg(path)
        .args(args)
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(Input::new(is_stereo, child_to_reader::<f32>(command), AudioType::FloatPcm, None))
}

// /// Creates a streamed audio source from a DCA file.
// /// Currently only accepts the [DCA1 format](https://github.com/bwmarrin/dca).
// pub fn dca<P: AsRef<OsStr>>(path: P) -> StdResult<Box<dyn AudioSource>, DcaError> {
//     _dca(path.as_ref())
// }

// fn _dca(path: &OsStr) -> StdResult<Box<dyn AudioSource>, DcaError> {
//     let file = File::open(path).map_err(DcaError::IoError)?;

//     let mut reader = BufReader::new(file);

//     let mut header = [0u8; 4];

//     // Read in the magic number to verify it's a DCA file.
//     reader.read_exact(&mut header).map_err(DcaError::IoError)?;

//     if header != b"DCA1"[..] {
//         return Err(DcaError::InvalidHeader);
//     }

//     reader.read_exact(&mut header).map_err(DcaError::IoError)?;

//     let size = (&header[..]).read_i32::<LittleEndian>().unwrap();

//     // Sanity check
//     if size < 2 {
//         return Err(DcaError::InvalidSize(size));
//     }

//     let mut raw_json = Vec::with_capacity(size as usize);

//     {
//         let json_reader = reader.by_ref();
//         json_reader
//             .take(size as u64)
//             .read_to_end(&mut raw_json)
//             .map_err(DcaError::IoError)?;
//     }

//     let metadata = serde_json::from_slice::<DcaMetadata>(raw_json.as_slice())
//         .map_err(DcaError::InvalidMetadata)?;

//     Ok(opus(metadata.is_stereo(), reader))
// }

// /// Creates an Opus audio source. This makes certain assumptions: namely, that the input stream
// /// is composed ONLY of opus frames of the variety that Discord expects.
// ///
// /// If you want to decode a `.opus` file, use [`ffmpeg`]
// ///
// /// [`ffmpeg`]: fn.ffmpeg.html
// pub fn opus<R: Read + Send + 'static>(is_stereo: bool, reader: R) -> Box<dyn AudioSource + Send> {
//     Box::new(Input {
//         stereo: is_stereo,
//         reader,
//         kind: AudioType::Opus,
//         decoder: Some(
//             Arc::new(Mutex::new(
//                 // We always want to decode *to* stereo, for mixing reasons.
//                 OpusDecoder::new(audio::SAMPLE_RATE, Channels::Stereo).unwrap()
//             ))
//         ),
//     })
// }

/// Creates a streamed audio source with `youtube-dl` and `ffmpeg`.
pub fn ytdl(uri: &str) -> Result<Input> {
    let ytdl_args = [
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        uri,
        "-o",
        "-"
    ];

    let ffmpeg_args = [
        "-f",
        "s16le",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-",
    ];

    let youtube_dl = Command::new("youtube-dl")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let ffmpeg = Command::new("ffmpeg")
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(youtube_dl.stdout.ok_or(SerenityError::Other("Failed to open youtube-dl stdout"))?)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(Input::new(true, child_to_reader::<f32>(ffmpeg), AudioType::FloatPcm, None))
}

/// Creates a streamed audio source from YouTube search results with `youtube-dl`,`ffmpeg`, and `ytsearch`.
/// Takes the first video listed from the YouTube search.
pub fn ytdl_search(name: &str) -> Result<Input> {
    let ytdl_args = [
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        &format!("ytsearch1:{}",name),
        "-o",
        "-"
    ];

    let ffmpeg_args = [
        "-f",
        "s16le",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-",
    ];

    let youtube_dl = Command::new("youtube-dl")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let ffmpeg = Command::new("ffmpeg")
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(youtube_dl.stdout.ok_or(SerenityError::Other("Failed to open youtube-dl stdout"))?)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(Input::new(true, child_to_reader::<f32>(ffmpeg), AudioType::FloatPcm, None))
}

fn is_stereo(path: &OsStr) -> Result<bool> {
    let args = ["-v", "quiet", "-of", "json", "-show_streams", "-i"];

    let out = Command::new("ffprobe")
        .args(&args)
        .arg(path)
        .stdin(Stdio::null())
        .output()?;

    let value: Value = serde_json::from_reader(&out.stdout[..])?;

    let streams = value
        .as_object()
        .and_then(|m| m.get("streams"))
        .and_then(|v| v.as_array())
        .ok_or(Error::Voice(VoiceError::Streams))?;

    let check = streams.iter().any(|stream| {
        let channels = stream
            .as_object()
            .and_then(|m| m.get("channels").and_then(|v| v.as_i64()));

        channels == Some(2)
    });

    Ok(check)
}

struct SharedCore {
    raw: UnsafeCell<RawCore>,
}

impl SharedCore {
    // The main reason for employing `unsafe` here is *shared mutability*:
    // due to the granularity of the locks we need, (i.e., a moving critical
    // section otherwise lock-free), we need to assert that these operations
    // are safe.
    //
    // Note that only our code can use this, so that we can ensure correctness
    // and concurrent safety.
    #[allow(clippy::mut_from_ref)]
    fn get_mut_ref(&self) -> &mut RawCore {
        unsafe { &mut *self.raw.get() }
    }

    fn remove_chain(&self) {
        self.get_mut_ref()
            .remove_chain()
    }

    fn read_from_pos(&self, pos: usize, loc: CacheReadLocationType, buffer: &mut [u8]) -> (IoResult<usize>, bool) {
        self.get_mut_ref()
            .read_from_pos(pos, loc, buffer)
    }

    fn len(&self) -> usize {
        self.get_mut_ref()
            .len
            .audio_pos
    }

    fn store_len(&self) -> usize {
        self.get_mut_ref()
            .len
            .audio_pos
    }

    fn is_stereo(&self) -> bool {
        self.get_mut_ref()
            .stereo
    }

    fn is_finalised(&self) -> bool {
        self.get_mut_ref()
            .finalised
    }

    fn lookahead(&self) -> IoResult<usize> {
        self.get_mut_ref()
            .inner_type
            .lookahead()
    }

    fn audio_to_backing_pos(&self, audio_byte_pos: usize, loc: CacheReadLocationType) -> Option<ChunkPosition> {
        self.get_mut_ref()
            .audio_to_backing_pos(audio_byte_pos, loc)
    }
}

enum EncodingData {
    FloatPcm,
    Opus{encoder: OpusEncoder, last_frame: Vec<u8>, frame_pos: usize}
}

impl EncodingData {
    fn lookahead(&self) -> IoResult<usize> {
        match self {
            Self::FloatPcm => Ok(0),
            Self::Opus{encoder, ..} => encoder.lookahead()
                .map(|n| n as usize)
                .map_err(|e| IoError::new(IoErrorKind::Other, e)),
        }
    }
    fn min_bytes_required(&self) -> usize {
        match self {
            Self::FloatPcm => 1,
            Self::Opus{..} => 2,
        }
    }

    fn read_from_source(&mut self, buf: &mut [u8], src: &mut Option<Box<Input>>, stereo: bool) -> (IoResult<ChunkPosition>, bool) {
        let src = src
            .as_mut()
            .expect("Source MUST exist while not finalised.");

        match self {
            Self::FloatPcm => {
                let out = src.read(buf)
                    .map(|sz| ChunkPosition::new(sz, sz));

                let eof = match out {
                    Ok(ChunkPosition{backing_pos:0 , ..}) => true,
                    _ => false,
                };

                (out, eof)
            },
            Self::Opus{ref mut encoder, ref mut last_frame, ref mut frame_pos} => {
                let output_start = mem::size_of::<u16>();
                let mut eof = false;

                let mut raw_len = 0;
                let mut out = None;
                let mut sample_buf = [0f32; STEREO_FRAME_SIZE];
                let samples_in_frame = if stereo { STEREO_FRAME_SIZE } else { MONO_FRAME_SIZE };

                // Purge old frame and read new, if needed.
                if *frame_pos == last_frame.len() + output_start || last_frame.is_empty() {
                    last_frame.resize(last_frame.capacity(), 0);

                    // We can't use `read_f32_into` because we can't guarantee the buffer will be filled.
                    for el in sample_buf[..samples_in_frame].iter_mut() {
                        match src.read_f32::<LittleEndian>() {
                            Ok(sample) => {
                                *el = sample;
                                raw_len += 1;
                            },
                            Err(e) if e.kind() == IoErrorKind::UnexpectedEof => {
                                eof = true;
                                break;
                            },
                            Err(e) => {
                                out = Some(Err(e));
                                break;
                            }
                        }
                    }

                    if out.is_none() && raw_len > 0 {
                        loop {
                            match encoder.encode_float(&sample_buf[..], &mut last_frame[..]) {
                                Ok(pkt_len) => {
                                    *frame_pos = 0;
                                    last_frame.truncate(pkt_len);
                                    break;
                                },
                                Err(OpusError::Opus(OpusErrorCode::BufferTooSmall)) => {
                                    // If we need more capacity to encode this frame, then take it.
                                    last_frame.resize(last_frame.len() + 256, 0);
                                },
                                Err(e) => {
                                    out = Some(Err(IoError::new(IoErrorKind::Other, e)));
                                    break;
                                },
                            }
                        }
                    }
                }

                if out.is_none() {
                    // Write from frame we have.
                    let start = if *frame_pos < output_start {
                        (&mut buf[..output_start]).write_u16::<LittleEndian>(last_frame.len() as u16);
                        *frame_pos += output_start;

                        output_start
                    } else { 0 };

                    let out_pos = *frame_pos - output_start;
                    let remaining = last_frame.len() - out_pos;
                    let write_len = remaining.min(buf.len() - start);
                    buf[start..start+write_len].copy_from_slice(&last_frame[out_pos..out_pos + write_len]);
                    *frame_pos += write_len;
                    out = Some(Ok(write_len + start));
                }

                // NOTE: use of raw_len here preserves true sample length even if
                // stream is extended to 20ms boundary.
                (out.unwrap_or_else(|| Err(IoError::new(IoErrorKind::Other, "Unclear.")))
                    .map(|compressed_sz| ChunkPosition::new(compressed_sz, raw_len * mem::size_of::<f32>())), eof)
            },
        }
    }
}

impl From<&EncodingData> for AudioType {
    fn from(a: &EncodingData) -> Self {
        match a {
            EncodingData::FloatPcm => AudioType::FloatPcm,
            EncodingData::Opus{ .. } => AudioType::Opus,
        }
    }
}

// Shared basis for the below cache-based seekables.
struct RawCore {
    len: ChunkPosition,
    finalised: bool,

    inner_type: EncodingData,

    source: Option<Box<Input>>,
    stereo: bool,

    chunk_size: usize,
    backing_store: Option<Vec<u8>>,
    chain: Option<LinkedList<BufferChunk>>,
    chain_users: Arc<()>,
    lock: Mutex<()>,
}

impl RawCore {
    fn new(source: Input, inner_type: EncodingData, chunk_size: usize, start_size: usize) -> Self {
        let stereo = source.stereo;

        let mut list = LinkedList::new();
        list.push_back(BufferChunk::new(Default::default(), start_size));

        Self {
            len: Default::default(),
            finalised: false,
            source: Some(Box::new(source)),
            stereo,
            inner_type,
            chunk_size,
            backing_store: None,
            chain: Some(list),
            chain_users: Arc::new(()),
            lock: Mutex::new(()),
        }
    }

    fn finalise(&mut self) {
        // Move the chain/rope of bytes into the backing store.BufferChunk
        let rope = self.chain.as_mut()
            .expect("Writes should only occur while the rope exists.");

        if rope.len() > 1 {
            // Allocate one big store, then start moving entries over
            // chunk-by-chunk.
            let mut back = vec![0u8; self.len.backing_pos];

            for el in rope.iter() {
                let start = el.start_pos.backing_pos;
                let end = el.end_pos.backing_pos;
                back[start..end]
                    .copy_from_slice(&el.data[..end-start]);
            }

            // Insert the new backing store, but DO NOT purge the old.
            // This is left to the last Arc<> holder of the chain.
            self.backing_store = Some(back);
        } else {
            // Least work, but unsafe.
            // We move the first chunk's buffer to become the backing store,
            // temporarily aliasing it until the list is destroyed.
            // In this case, when the list is destroyed, the first element
            // MUST be leaked to keep the backing store memory valid.
            //
            // (see remove_chain for this leakage)
            //
            // The alternative (write first chunk into always-present
            // backing store) mandates a lock for the final expansion, because
            // the backing store is IN USE. Thus, we can't employ it.
            if let Some(el) = rope.front_mut() {
                // We can be certain that this pointer is not invalidated because:
                // * All writes to the rope/chain are finished. Thus, no
                //   reallocations/moves.
                // * The Vec will live exactly as long as the RawCore, pointer never escapes.
                // Likewise, we knoe that it is safe to build the new vector as:
                // * The stored type and pointer do not change, so alignment is preserved.
                // * The data pointer is created by an existing Vec<T>.
                self.backing_store = Some(unsafe {
                    let data = el.data.as_mut_ptr();
                    Vec::from_raw_parts(data, el.data.len(), el.data.capacity())
                })
            }
        }

        // Drop the old input.
        self.source = None;

        // It's crucial that we do this *last*, as this is the signal
        // for other threads to migrate from rope to backing store.
        self.finalised = true;
    }

    fn remove_chain(&mut self) {
        // We can only remove the chain if the core holds the last reference.
        if Arc::strong_count(&self.chain_users) == 1 {
            // FIXME: make this use an atomic int with fetch_subtract

            // In worst case, several upgrades might pile up.
            // Only one holder should concern itself with drop logic,
            // the rest should carry on and start using the backing store.
            let maybe_lock = self.lock.try_lock();
            if maybe_lock.is_none() {
                return;
            }

            if let Some(rope) = &mut self.chain {
                // Prevent the backing store from being wiped out
                // if the first link in the chain sufficed.
                // This ensures safety as we undo the aliasing
                // in the above special case.
                if rope.len() == 1 {
                    let el = rope.pop_front().expect("Length of rope was established as >= 1.");
                    ManuallyDrop::new(el.data);
                }
            }

            // Drop everything else.
            self.chain = None;
        }
    }

    fn read_from_pos(&mut self, pos: usize, mut loc: CacheReadLocationType, buf: &mut [u8]) -> (IoResult<usize>, bool) {
        // If should upgrade, then we take from backing.
        let mut should_upgrade = matches!(loc, CacheReadLocationType::Chained) && self.finalised;
        if should_upgrade {
            loc = CacheReadLocationType::Backed;
        }

        let target_len = pos + buf.len();

        let out = if target_len <= self.len.backing_pos || self.finalised {
            // If finalised, there is zero risk of triggering more writes.
            Ok(self.read_from_local(pos, loc, buf))
        } else {
            let mut read = 0;
            let mut base_result = None;

            loop {
                let mut remaining_in_store = self.len.backing_pos - pos - read;

                if remaining_in_store == 0 {
                    // Need to do this to trigger the lock
                    // while holding mutability to the other members.
                    let lock: *mut Mutex<()> = &mut self.lock;
                    let guard = unsafe {
                        let lock = & *lock;
                        lock.lock()
                    };

                    // If length changed between our check and
                    // acquiring the lock, then drop it -- we don't need new bytes *yet*
                    // and might not!
                    remaining_in_store = self.len.backing_pos - pos - read;
                    if remaining_in_store == 0 {
                        let read_count = self.fill_from_source(buf.len() - read);
                        if let Ok(read_count) = read_count {
                            remaining_in_store += read_count;
                        }
                        base_result = Some(read_count);

                        should_upgrade |= self.finalised;
                    }

                    // Unlocked here.
                    MutexGuard::unlock_fair(guard);
                }

                if remaining_in_store > 0 {
                    read += self.read_from_local(pos, loc, &mut buf[read..]);
                }

                // break out if:
                // * no space in reader's buffer
                // * hit an error
                // * or nothing remaining, AND finalised
                if matches!(base_result, Some(Err(_)))
                    || read == buf.len()
                    || (self.finalised && self.len.backing_pos == pos + read) {
                    break;
                }
            }

            base_result
                .unwrap_or(Ok(0))
                .map(|_| read)
        };

        (out, should_upgrade)
    }

    // ONLY SAFE TO CALL WITH LOCK.
    // The critical section concerns:
    // * adding new elements to the rope
    // * drawing bytes from the source
    // * modifying len
    // * modifying encoder state
    fn fill_from_source(&mut self, mut bytes_needed: usize) -> IoResult<usize> {
        let minimum_to_write = self.inner_type.min_bytes_required();
        // Round up to the next full audio frame.
        // FIXME: cache this.
        let frame_len = timestamp_to_byte_count(Duration::from_millis(20), self.stereo);

        let overspill = bytes_needed % frame_len;
        if overspill != 0 {
            bytes_needed += frame_len - overspill;
        }

        let mut remaining_bytes = bytes_needed;
        let mut recorded_error = None;

        loop {
            let rope = self.chain.as_mut()
                .expect("Writes should only occur while the rope exists.");

            let rope_el = rope.back_mut()
                .expect("There will always be at least one element in rope.");

            let old_len = rope_el.data.len();
            let cap = rope_el.data.capacity();
            let space = cap - old_len;

            let new_len = old_len + space.min(remaining_bytes);

            if space < minimum_to_write {
                let end = rope_el.end_pos;
                // Make a new chunk!
                rope.push_back(BufferChunk::new(
                    end,
                    self.chunk_size,
                ));
            } else {
                rope_el.data.resize(new_len, 0);
                let (pos, eofd) = self.inner_type.read_from_source(&mut rope_el.data[old_len..], &mut self.source, self.stereo);
                match pos {
                    Ok(len) => {
                        // When to not write this?
                        if len.audio_pos > 0 {
                            rope_el.first_frame_head.get_or_insert(old_len);
                        }

                        rope_el.end_pos += len;
                        let store_len = len.backing_pos;

                        rope_el.data.truncate(old_len + store_len);

                        remaining_bytes -= store_len;
                        self.len += len;
                    },
                    Err(e) if e.kind() == IoErrorKind::Interrupted => {
                        // DO nothing, so try again.
                    },
                    Err(e) => {
                        recorded_error = Some(Err(e));
                    }
                }

                if eofd {
                    self.finalise();
                }

                if self.finalised || remaining_bytes < minimum_to_write || recorded_error.is_some() {
                    break;
                }
            }
            }

        recorded_error.unwrap_or(Ok(bytes_needed - remaining_bytes))
    }

    #[inline]
    fn read_from_local(&self, mut pos: usize, loc: CacheReadLocationType, buf: &mut [u8]) -> usize {
        use CacheReadLocationType::*;
        match loc {
            Backed => {
                let store = self.backing_store
                    .as_ref()
                    .expect("Reader should not attempt to use a backing store before it exists");

                if pos < self.len.backing_pos {
                    let available = self.len.backing_pos - pos;
                    let to_write = buf.len().min(available);
                    buf[..to_write].copy_from_slice(&store[pos..pos + to_write]);

                    to_write
                } else {
                    0
                }
            },
            Chained => {
                let rope = self.chain
                    .as_ref()
                    .expect("Rope should still exist while any handles hold a ::Chained(_) \
                             (and thus an Arc)");

                let mut written = 0;

                for link in rope.iter() {
                    if pos >= link.start_pos.backing_pos && pos < link.end_pos.backing_pos {
                        let local_available = link.end_pos.backing_pos - pos;
                        let to_write = (buf.len() - written).min(local_available);

                        let first_el = pos - link.start_pos.backing_pos;
                        let last_el = first_el + to_write;

                        let next_len = written + to_write;

                        buf[written..next_len].copy_from_slice(&link.data[first_el..last_el]);

                        written = next_len;
                        pos += to_write;
                    }

                    if written >= buf.len() {
                        break;
                    }
                }

                written
            }
        }
    }

    fn audio_to_backing_pos(&self, audio_byte_pos: usize, loc: CacheReadLocationType) -> Option<ChunkPosition> {
        if audio_byte_pos > self.len.audio_pos {
            if self.finalised {
                Some(self.len)
            } else {
                None
            }
        } else {
            let audio_bytes_per_frame = if self.stereo { STEREO_FRAME_BYTE_SIZE } else { MONO_FRAME_BYTE_SIZE };
            match self.inner_type {
                EncodingData::FloatPcm => {
                    Some(ChunkPosition::new(audio_byte_pos, audio_byte_pos))
                },
                EncodingData::Opus{..} => {
                    match loc {
                        CacheReadLocationType::Backed => {
                            let back = self.backing_store.as_ref()
                                .expect("Can't access this code path unless nacking store exists.");

                            // Walk the backing store.
                            // FIXME: accelerate this...
                            let mut pos = ChunkPosition::default();
                            let mut frame_head = 0;

                            while audio_byte_pos - pos.audio_pos >= audio_bytes_per_frame {
                                let next = (&back[frame_head..]).read_u16::<LittleEndian>()
                                    .expect("Frame head should point to u16...") as usize;
                                frame_head += mem::size_of::<u16>() + next;
                                pos.audio_pos += audio_bytes_per_frame;
                            }

                            pos.backing_pos += frame_head;

                            Some(pos)
                        },
                        CacheReadLocationType::Chained => {
                            let rope = self.chain
                                .as_ref()
                                .expect("Rope should still exist while any handles hold a ::Chained(_) \
                                         (and thus an Arc)");

                            let mut out = None;
                            for el in rope.iter() {
                                if audio_byte_pos >= el.start_pos.audio_pos && audio_byte_pos < el.end_pos.audio_pos {
                                    let mut pos = el.start_pos;
                                    let mut frame_head = el.first_frame_head
                                        .expect("First frame location must be defined if we're able to read the chunk.");

                                    while audio_byte_pos - pos.audio_pos >= audio_bytes_per_frame {
                                        let next = (&el.data[frame_head..]).read_u16::<LittleEndian>()
                                            .expect("Frame head should point to u16...") as usize;
                                        frame_head += mem::size_of::<u16>() + next;
                                        pos.audio_pos += audio_bytes_per_frame;
                                    }

                                    pos.backing_pos += frame_head;

                                    out = Some(pos);
                                    break;
                                }
                            }
                            out
                        },
                    }
                }
            }
        }
    }
}

// We need to declare these as thread-safe, since we don't have a mutex around
// several raw fields. However, the way that they are used should remain
// consistent.
unsafe impl Sync for SharedCore {}
unsafe impl Send for SharedCore {}

struct BufferChunk {
    data: Vec<u8>,

    start_pos: ChunkPosition,
    end_pos: ChunkPosition,

    first_frame_head: Option<usize>,
}

impl BufferChunk {
    fn new(start_pos: ChunkPosition, chunk_len: usize) -> Self {
        BufferChunk {
            data: Vec::with_capacity(chunk_len),

            start_pos,
            end_pos: start_pos,

            first_frame_head: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ChunkPosition {
    /// Marks the byte count as seen by the backing store.
    backing_pos: usize,

    /// Marks the byte count as seen by a reader (i.e., audio bytes).
    audio_pos: usize,
}

impl ChunkPosition {
    fn new(backing_pos: usize, audio_pos: usize) -> Self {
        Self {
            backing_pos,
            audio_pos,
        }
    }
}

impl Add for ChunkPosition {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            backing_pos: self.backing_pos + other.backing_pos,
            audio_pos: self.audio_pos + other.audio_pos,
        }
    }
}

impl AddAssign for ChunkPosition {
    fn add_assign(&mut self, other: Self) {
        self.backing_pos += other.backing_pos;
        self.audio_pos += other.audio_pos;
    }
}

impl Sub for ChunkPosition {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            backing_pos: self.backing_pos - other.backing_pos,
            audio_pos: self.audio_pos - other.audio_pos,
        }
    }
}

impl SubAssign for ChunkPosition {
    fn sub_assign(&mut self, other: Self) {
        self.backing_pos -= other.backing_pos;
        self.audio_pos -= other.audio_pos;   
    }
}

#[derive(Clone, Debug)]
enum CacheReadLocation {
    Chained(Arc<()>),
    Backed,
}

#[derive(Clone, Copy, Debug)]
enum CacheReadLocationType {
    Chained,
    Backed,
}

impl From<&CacheReadLocation> for CacheReadLocationType {
    fn from(a: &CacheReadLocation) -> Self {
        match a {
            CacheReadLocation::Chained(_) => CacheReadLocationType::Chained,
            CacheReadLocation::Backed => CacheReadLocationType::Backed,
        }
    }
}

struct AudioCacheCore {
    core: Arc<SharedCore>,
    pos: usize,
    loc: CacheReadLocation,
}

impl AudioCacheCore {
    fn new(core: RawCore) -> Self {
        let loc = CacheReadLocation::Chained(core.chain_users.clone());
        AudioCacheCore {
            core: Arc::new(SharedCore{ raw: UnsafeCell::new(core) }),
            pos: 0,
            loc,
        }
    }

    fn upgrade_to_backing(&mut self) {
        self.loc = CacheReadLocation::Backed;
        self.core.remove_chain();
    }

    fn new_handle(&self) -> Self {
        Self {
            core: self.core.clone(),
            pos: 0,
            loc: self.loc.clone(),
        }
    }

    fn load_file(&mut self) {
        let pos = self.pos;
        while self.consume(1920 * mem::size_of::<f32>()) > 0 && !self.is_finalised() {}
        self.pos = pos;
    }

    fn spawn_loader(&self) -> std::thread::JoinHandle<()> {
        let mut handle = self.new_handle();
        std::thread::spawn(move || {
            handle.load_file();
        })
    }

    fn is_finalised(&self) -> bool {
        self.core.is_finalised()
    }

    fn audio_to_backing_pos(&self, audio_byte_pos: usize) -> Option<ChunkPosition> {
        self.core.audio_to_backing_pos(audio_byte_pos, (&self.loc).into())
    }
}

// Read and Seek on the audio operate on byte positions
// of the output FloatPcm stream.
impl Read for AudioCacheCore {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let (bytes_read, should_upgrade) = self.core.read_from_pos(self.pos, (&self.loc).into(), buf);

        if should_upgrade {
            self.upgrade_to_backing();
        }

        if let Ok(size) = bytes_read {
            self.pos += size;
        }

        bytes_read
    }
}

/// A wrapper around an existing [`AudioSource`] which caches
/// the decoded and converted audio data locally in memory.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`RestartableSource`]
/// and [`CompressedSource`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with small, repeatedly used audio
/// tracks shared between sources, and stores the sound data
/// retrieved in **uncompressed floating point** form to minimise the
/// cost of audio processing. This is a significant *3 Mbps (375 kiB/s)*,
/// or 131 MiB of RAM for a 6 minute song.
///
/// [`AudioSource`]: trait.AudioSource.html
/// [`CompressedSource`]: struct.CompressedSource.html
/// [`RestartableSource`]: struct.RestartableSource.html
pub struct MemorySource {
    cache: AudioCacheCore,
}

impl MemorySource {
    pub fn new(source: Input, length_hint: Option<Duration>) -> Self {
        let stereo = source.stereo;
        let chunk_size = timestamp_to_byte_count(Duration::from_secs(5), stereo);

        let start_size = if let Some(length) = length_hint {
            timestamp_to_byte_count(length, stereo)
        } else {
            chunk_size
        };

        let core_raw = RawCore::new(source, EncodingData::FloatPcm, chunk_size, start_size);

        Self {
            cache: AudioCacheCore::new(core_raw),
        }
    }

    pub fn new_handle(&self) -> Self {
        Self {
            cache: self.cache.new_handle(),
        }
    }

    pub fn load_file(&mut self) {
        self.cache.load_file();
    }

    pub fn spawn_loader(&self) -> std::thread::JoinHandle<()> {
        self.cache.spawn_loader()
    }
}

impl From<MemorySource> for Input {
    fn from(src: MemorySource) -> Self {
        Self {
            stereo: src.cache.core.is_stereo(),
            kind: AudioType::FloatPcm,
            decoder: None,

            reader: Reader::InMemory(src),
        }
    }
}

impl Read for MemorySource {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.cache.read(buf)
    }
}

impl Seek for MemorySource {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let old_pos = self.cache.pos as u64;

        let (valid, new_pos) = match pos {
            SeekFrom::Current(adj) => {
                // overflow expected in many cases.
                let new_pos = old_pos.wrapping_add(adj as u64);
                (adj >= 0 || (adj.abs() as u64) <= old_pos, new_pos)
            }
            SeekFrom::End(adj) => {
                // FIXME: make this check for metadata as the basis?
                self.load_file();

                let len = self.cache.core.len() as u64;
                let new_pos = len.wrapping_add(adj as u64);
                (adj >= 0 || (adj.abs() as u64) <= len, new_pos)
            }
            SeekFrom::Start(new_pos) => {
                (true, new_pos)
            }
        };

        if valid {
            if new_pos > old_pos {
                self.cache.consume((new_pos - old_pos) as usize);
            }

            let len = self.cache.core.len() as u64;

            self.cache.pos = new_pos.min(len) as usize;
            Ok(self.cache.pos as u64)
        } else {
            Err(IoError::new(IoErrorKind::InvalidInput, "Tried to seek before start of stream."))
        }
    }
}

/// A wrapper around an existing [`AudioSource`] which caches
/// the decoded and converted audio data locally in memory.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`RestartableSource`]
/// and [`MemorySource`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with larger, repeatedly used audio
/// tracks shared between sources, and stores the sound data
/// retrieved as **compressed Opus audio**. There is an associated memory cost,
/// but this is far smaller than using a [`MemorySource`].
///
/// [`AudioSource`]: trait.AudioSource.html
/// [`MemorySource`]: struct.MemorySource.html
/// [`RestartableSource`]: struct.RestartableSource.html
pub struct CompressedSource {
    cache: AudioCacheCore,
    decoder: OpusDecoder,
    current_frame: Vec<f32>,
    frame_pos: usize,
    audio_pos: usize,
    remaining_lookahead: Option<usize>,
    frame_pos_override: Option<usize>,
}

impl CompressedSource {
    pub fn new(source: Input, bitrate: Bitrate, length_hint: Option<Duration>) -> Self {
        let channels = if source.stereo { Channels::Stereo } else { Channels::Mono };
        let mut encoder = OpusEncoder::new(SampleRate::Hz48000, channels, Application::Audio)
            .expect("Failed to create opus encoder.");

        encoder.set_bitrate(bitrate)
            .expect("Failed to set encoder bitrate.");

        Self::with_encoder(source, encoder, length_hint)
    }

    pub fn with_encoder(source: Input, encoder: OpusEncoder, length_hint: Option<Duration>) -> Self {
        let framing_cost_per_sec = AUDIO_FRAME_RATE * mem::size_of::<u16>();
        let bitrate = encoder.bitrate()
            .expect("Failed to get bitrate.");

        let bitrate_raw = match bitrate {
            Bitrate::BitsPerSecond(i) => i,
            Bitrate::Auto => 64_000,
            Bitrate::Max => 510_000,
        } as usize;

        let est_cost_per_sec = (bitrate_raw / 8) + framing_cost_per_sec;

        let chunk_size = est_cost_per_sec * 5;

        let start_size = if let Some(length) = length_hint {
            let sec_count = length.as_secs() + if length.subsec_nanos() != 0 { 1 } else { 0 };
            est_cost_per_sec * sec_count as usize
        } else {
            chunk_size
        };

        let encoder_data = EncodingData::Opus{
            encoder,
            last_frame: Vec::with_capacity(4000),//256 + est_cost_per_sec / AUDIO_FRAME_RATE),
            frame_pos: 0,
        };

        let stereo = source.stereo;

        let core_raw = RawCore::new(source, encoder_data, chunk_size, start_size);

        // FIXME: does not take into account actual encoder params.
        Self {
            cache: AudioCacheCore::new(core_raw),
            decoder: decoder(stereo).expect("FIXME: shouldn't unwrap."),
            current_frame: Vec::with_capacity(STEREO_FRAME_SIZE),
            frame_pos: 0,
            audio_pos: 0,
            remaining_lookahead: None,
            frame_pos_override: None,
        }
    }

    pub fn new_handle(&self) -> Self {
        Self {
            cache: self.cache.new_handle(),
            decoder: decoder(self.cache.core.is_stereo()).expect("FIXME: shouldn't unwrap."),
            current_frame: Vec::with_capacity(STEREO_FRAME_SIZE),
            frame_pos: 0,
            audio_pos: 0,
            remaining_lookahead: None,
            frame_pos_override: None,
        }
    }

    pub fn into_sendable(self) -> CompressedSourceBase {
        CompressedSourceBase {
            cache: self.cache,
        }
    }

    pub fn load_file(&mut self) {
        self.cache.load_file();
    }

    pub fn spawn_loader(&self) -> std::thread::JoinHandle<()> {
        self.cache.spawn_loader()
    }

    pub fn reset(&mut self) {
        self.remaining_lookahead = None;
        self.cache.pos = 0;
        self.audio_pos = 0;
        self.frame_pos = 0;
        self.current_frame.truncate(0);
        self.decoder.reset_state();
    }
}

fn decoder(stereo: bool) -> OpusResult<OpusDecoder> {
    OpusDecoder::new(
        SampleRate::Hz48000,
        if stereo { Channels::Stereo } else { Channels::Mono },
    )
}

impl From<CompressedSource> for Input {
    fn from(src: CompressedSource) -> Self {
        Input {
            stereo: src.cache.core.is_stereo(),
            kind: AudioType::FloatPcm,
            decoder: None,

            reader: Reader::Compressed(src),
        }   
    }
}

impl Read for CompressedSource {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let sample_len = mem::size_of::<f32>();

        // We need to discard this many samples of payload
        // after any seek or fresh start.
        if self.remaining_lookahead.is_none() {
            self.remaining_lookahead = Some(
                self.cache.core.lookahead().unwrap_or(0)
                    * if self.cache.core.is_stereo() { 2 } else { 1 }
            );
        }

        if self.frame_pos == self.current_frame.len() {
            if self.cache.core.is_finalised() && self.cache.core.len() == self.cache.pos {
                return Ok(0);
            }
            let mut temp_buf = [0u8; STEREO_FRAME_BYTE_SIZE];
            self.current_frame.resize(self.current_frame.capacity(), 0.0);

            let sz = (self.cache.read_u16::<LittleEndian>()?) as usize;
            self.cache.read_exact(&mut temp_buf[..sz])?;

            let decode_len = self.decoder.decode_float(Some(&temp_buf[..sz]), &mut self.current_frame, false)
                .map_err(|e| IoError::new(IoErrorKind::Other, e))?;

            self.current_frame.truncate(sample_len * decode_len);
            if let Some(pos) = self.frame_pos_override {
                self.frame_pos = pos;
            } else {
                self.frame_pos = 0;
            }
            self.frame_pos_override = None;
        }

        let buf_float_space = buf.len() / sample_len;
        let unwritten_floats = self.current_frame.len() - self.frame_pos;
        let floats_to_write = buf_float_space.min(unwritten_floats);

        if let Some(ref mut bad_sample_count) = self.remaining_lookahead {
            let old_count = *bad_sample_count;
            let to_drain = unwritten_floats.min(old_count);

            *bad_sample_count -= to_drain;
            self.frame_pos += to_drain;

            if old_count > unwritten_floats {
                return Err(IoError::new(
                    IoErrorKind::Interrupted,
                    "Not enough samples in packet to drain lookahead -- keep reading.",
                ));
            }
        }

        {
            let mut buf = &mut buf[..];

            for val in &self.current_frame[self.frame_pos..self.frame_pos + floats_to_write] {
                buf.write_f32::<LittleEndian>(*val)?;
            }
        }

        self.frame_pos += floats_to_write;
        Ok(floats_to_write * sample_len)
    }
}

impl Seek for CompressedSource {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        // BE AWARE: this refers to a seek in audio space, and NOT in backing store.
        let old_pos = self.audio_pos as u64;

        // FIXME: duped from above.
        let (valid, new_pos) = match pos {
            SeekFrom::Current(adj) => {
                // overflow expected in many cases.
                let new_pos = old_pos.wrapping_add(adj as u64);
                (adj >= 0 || (adj.abs() as u64) <= old_pos, new_pos)
            }
            SeekFrom::End(adj) => {
                // FIXME: make this check for metadata as the basis?
                self.load_file();

                let len = self.cache.core.len() as u64;
                let new_pos = len.wrapping_add(adj as u64);
                (adj >= 0 || (adj.abs() as u64) <= len, new_pos)
            }
            SeekFrom::Start(new_pos) => {
                (true, new_pos)
            }
        };

        let new_pos = new_pos as usize;

        if valid {
            loop {
                if let Some(new_backing_pos) = self.cache.audio_to_backing_pos(new_pos) {
                    // We now have the start of the frame which includes the desired pos.
                    // NOTE: we hit this branch once finalised, too.
                    self.reset();
                    self.cache.pos = new_backing_pos.backing_pos;
                    if self.cache.pos != self.cache.core.store_len() {
                        self.frame_pos_override = Some(new_pos - new_backing_pos.audio_pos);
                    }
                    break;
                } else {
                    let sz = (self.cache.read_u16::<LittleEndian>()?) as usize;
                    self.cache.consume(sz);
                }
            }

            Ok(self.cache.pos as u64)
        } else {
            Err(IoError::new(IoErrorKind::InvalidInput, "Tried to seek before start of stream."))
        }
    }
}

pub struct CompressedSourceBase {
    cache: AudioCacheCore,
}

impl CompressedSourceBase {
    pub fn new_handle(&self) -> CompressedSource {
        CompressedSource {
            cache: self.cache.new_handle(),
            decoder: decoder(self.cache.core.is_stereo()).expect("FIXME: shouldn't unwrap."),
            current_frame: Vec::with_capacity(STEREO_FRAME_SIZE),
            frame_pos: 0,
            audio_pos: 0,
            remaining_lookahead: None,
            frame_pos_override: None,
        }
    }
}

/// A wrapper around a method to create a new [`AudioSource`] which
/// seeks backward by recreating the source.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`CompressedSource`]
/// and [`MemorySource`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with single-use audio tracks which
/// may require looping or seeking, but where additional memory
/// cannot be spared. Forward seeks will drain the track until reaching
/// the desired timestamp.
///
/// [`AudioSource`]: trait.AudioSource.html
/// [`MemorySource`]: struct.MemorySource.html
/// [`CompressedSource`]: struct.CompressedSource.html
pub struct RestartableSource {
    position: usize,
    recreator: Box<dyn Fn(Option<Duration>) -> Result<Input> + Send + 'static>,
    source: Box<Input>,
}

impl RestartableSource {
    pub fn new(recreator: impl Fn(Option<Duration>) -> Result<Input> + Send + 'static) -> Result<Self> {
        recreator(None)
            .map(move |source| {
                Self {
                    position: 0,
                    recreator: Box::new(recreator),
                    source: Box::new(source),
                }
            })
    }

    pub fn ffmpeg<P: AsRef<OsStr> + Send + Clone + Copy + 'static>(path: P) -> Result<Self> {
        Self::new(move |time: Option<Duration>| {
            if let Some(time) = time {

                let is_stereo = is_stereo(path.as_ref()).unwrap_or(false);
                let stereo_val = if is_stereo { "2" } else { "1" };

                let ts = format!("{}.{}", time.as_secs(), time.subsec_millis());
                _ffmpeg_optioned(path.as_ref(), &[
                    "-ss",
                    &ts,
                    ],

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
                ], Some(is_stereo))
            } else {
                ffmpeg(path)
            }
        })
    }
}

impl From<RestartableSource> for Input {
    fn from(src: RestartableSource) -> Self {
        Self {
            stereo: src.source.stereo,
            kind: src.source.kind,
            decoder: None,

            reader: Reader::Restartable(src),   
        }
    }
}

impl Read for RestartableSource {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        self.source.read(buffer)
            .map(|a| { self.position += a; a })
    }
}

impl Seek for RestartableSource {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let _local_pos = self.position as u64;

        use SeekFrom::*;
        match pos {
            Start(offset) => {
                let stereo = self.source.stereo;
                let _current_ts = byte_count_to_timestamp(self.position, stereo);
                let offset = offset as usize;

                if offset < self.position {
                    // FIXME: don't unwrap
                    self.source = Box::new(
                        (self.recreator)(
                            Some(byte_count_to_timestamp(offset, stereo))
                        ).unwrap()
                    );
                    self.position = offset;
                } else {
                    self.position += self.source.consume(offset - self.position);
                }

                Ok(offset as u64)
            },
            End(_offset) => Err(IoError::new(
                IoErrorKind::InvalidInput,
                "End point for RestartableSources is not known.")),
            Current(_offset) => unimplemented!(),
        }
    }
}

fn timestamp_to_sample_count(timestamp: Duration, stereo: bool) -> usize {
    ((timestamp.as_millis() as usize) * (MONO_FRAME_SIZE / FRAME_LEN_MS)) << stereo as usize
}

fn sample_count_to_timestamp(amt: usize, stereo: bool) -> Duration {
    Duration::from_millis((((amt * FRAME_LEN_MS) / MONO_FRAME_SIZE) as u64) >> stereo as u64)
}

fn timestamp_to_byte_count(timestamp: Duration, stereo: bool) -> usize {
    timestamp_to_sample_count(timestamp, stereo) * mem::size_of::<f32>()
}

fn byte_count_to_timestamp(amt: usize, stereo: bool) -> Duration {
    sample_count_to_timestamp(amt / mem::size_of::<f32>(), stereo)
}
