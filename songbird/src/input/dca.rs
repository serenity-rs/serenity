use super::{codec::OpusDecoderState, error::DcaError, Codec, Container, Input, Metadata, Reader};
use serde::Deserialize;
use std::{ffi::OsStr, io::BufReader, mem};
use tokio::{fs::File as TokioFile, io::AsyncReadExt};

/// Creates a streamed audio source from a DCA file.
/// Currently only accepts the [DCA1 format](https://github.com/bwmarrin/dca).
pub async fn dca<P: AsRef<OsStr>>(path: P) -> Result<Input, DcaError> {
    _dca(path.as_ref()).await
}

async fn _dca(path: &OsStr) -> Result<Input, DcaError> {
    let mut reader = TokioFile::open(path).await.map_err(DcaError::IoError)?;

    let mut header = [0u8; 4];

    // Read in the magic number to verify it's a DCA file.
    reader
        .read_exact(&mut header)
        .await
        .map_err(DcaError::IoError)?;

    if header != b"DCA1"[..] {
        return Err(DcaError::InvalidHeader);
    }

    let size = reader
        .read_i32_le()
        .await
        .map_err(|_| DcaError::InvalidHeader)?;

    // Sanity check
    if size < 2 {
        return Err(DcaError::InvalidSize(size));
    }

    let mut raw_json = Vec::with_capacity(size as usize);

    let mut json_reader = reader.take(size as u64);

    json_reader
        .read_to_end(&mut raw_json)
        .await
        .map_err(DcaError::IoError)?;

    let reader = BufReader::new(json_reader.into_inner().into_std().await);

    let metadata: Metadata = serde_json::from_slice::<DcaMetadata>(raw_json.as_slice())
        .map_err(DcaError::InvalidMetadata)?
        .into();

    let stereo = metadata.channels == Some(2);

    Ok(Input::new(
        stereo,
        Reader::File(reader),
        Codec::Opus(OpusDecoderState::new().map_err(DcaError::Opus)?),
        Container::Dca {
            first_frame: (size as usize) + mem::size_of::<i32>() + header.len(),
        },
        Some(metadata),
    ))
}

#[derive(Debug, Deserialize)]
pub(crate) struct DcaMetadata {
    pub(crate) dca: Dca,
    pub(crate) opus: Opus,
    pub(crate) info: Option<Info>,
    pub(crate) origin: Option<Origin>,
    pub(crate) extra: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Dca {
    pub(crate) version: u64,
    pub(crate) tool: Tool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Tool {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) url: String,
    pub(crate) author: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Opus {
    pub(crate) mode: String,
    pub(crate) sample_rate: u32,
    pub(crate) frame_size: u64,
    pub(crate) abr: u64,
    pub(crate) vbr: u64,
    pub(crate) channels: u8,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Info {
    pub(crate) title: Option<String>,
    pub(crate) artist: Option<String>,
    pub(crate) album: Option<String>,
    pub(crate) genre: Option<String>,
    pub(crate) cover: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Origin {
    pub(crate) source: Option<String>,
    pub(crate) abr: Option<u64>,
    pub(crate) channels: Option<u8>,
    pub(crate) encoding: Option<String>,
    pub(crate) url: Option<String>,
}

impl From<DcaMetadata> for Metadata {
    fn from(mut d: DcaMetadata) -> Self {
        let (title, artist) = d
            .info
            .take()
            .map(|mut m| (m.title.take(), m.artist.take()))
            .unwrap_or_else(|| (None, None));

        let channels = Some(d.opus.channels);
        let sample_rate = Some(d.opus.sample_rate);

        Self {
            title,
            artist,

            channels,
            sample_rate,

            ..Default::default()
        }
    }
}
