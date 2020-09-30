use serde::Deserialize;
use super::Metadata;

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

impl DcaMetadata {
    pub(crate) fn is_stereo(&self) -> bool { self.opus.channels == 2 }
}

impl From<DcaMetadata> for Metadata {
	fn from(mut d: DcaMetadata) -> Self {
		let (title, artist) = d.info.take()
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
