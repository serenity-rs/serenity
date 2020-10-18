use super::{
    child_to_reader,
    error::{Error, Result},
    Codec,
    Container,
    Input,
    Metadata,
};
use std::process::{Command, Stdio};
use tracing::trace;

/// Creates a streamed audio source with `youtube-dl` and `ffmpeg`.
pub async fn ytdl(uri: &str) -> Result<Input> {
    _ytdl(uri, &[]).await
}

pub(crate) async fn _ytdl(uri: &str, pre_args: &[&str]) -> Result<Input> {
    let ytdl_args = [
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        uri,
        "-o",
        "-",
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
        .args(pre_args)
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(youtube_dl.stdout.ok_or(Error::Stdout)?)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let metadata = Metadata::from_ytdl_uri(uri).await;

    trace!("ytdl metadata {:?}", metadata);

    Ok(Input::new(
        true,
        child_to_reader::<f32>(ffmpeg),
        Codec::FloatPcm,
        Container::Raw,
        Some(metadata),
    ))
}

/// Creates a streamed audio source from YouTube search results with `youtube-dl`,`ffmpeg`, and `ytsearch`.
/// Takes the first video listed from the YouTube search.
pub async fn ytdl_search(name: &str) -> Result<Input> {
    ytdl(&format!("ytsearch1:{}", name)).await
}
