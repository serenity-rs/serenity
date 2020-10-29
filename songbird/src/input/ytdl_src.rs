use super::{
    child_to_reader,
    error::{Error, Result},
    Codec,
    Container,
    Input,
    Metadata,
};
use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Read},
    process::{Command, Stdio},
};
use tokio::task;
use tracing::trace;

/// Creates a streamed audio source with `youtube-dl` and `ffmpeg`.
pub async fn ytdl(uri: &str) -> Result<Input> {
    _ytdl(uri, &[]).await
}

pub(crate) async fn _ytdl(uri: &str, pre_args: &[&str]) -> Result<Input> {
    let ytdl_args = [
        "--print-json",
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

    let mut youtube_dl = Command::new("youtube-dl")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stderr = youtube_dl.stderr.take();

    let (returned_stderr, value) = task::spawn_blocking(move || {
        if let Some(mut s) = stderr {
            let out: Option<Value> = {
                let mut o_vec = vec![];
                let mut serde_read = BufReader::new(s.by_ref());
                // Newline...
                if let Ok(len) = serde_read.read_until(0xA, &mut o_vec) {
                    serde_json::from_slice(&o_vec[..len]).ok()
                } else {
                    None
                }
            };

            (Some(s), out)
        } else {
            (None, None)
        }
    })
    .await
    .map_err(|_| Error::Metadata)?;

    youtube_dl.stderr = returned_stderr;

    let ffmpeg = Command::new("ffmpeg")
        .args(pre_args)
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(youtube_dl.stdout.ok_or(Error::Stdout)?)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let metadata = Metadata::from_ytdl_output(value.unwrap_or_default());

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
