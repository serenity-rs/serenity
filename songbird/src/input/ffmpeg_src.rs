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
    ffi::OsStr,
    process::{Command, Stdio},
};
use tokio::process::Command as TokioCommand;
use tracing::debug;

/// Opens an audio file through `ffmpeg` and creates an audio source.
pub async fn ffmpeg<P: AsRef<OsStr>>(path: P) -> Result<Input> {
    _ffmpeg(path.as_ref()).await
}

pub(crate) async fn _ffmpeg(path: &OsStr) -> Result<Input> {
    // Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path)
        .await
        .unwrap_or_else(|_e| (false, Default::default()));
    let stereo_val = if is_stereo.0 { "2" } else { "1" };

    _ffmpeg_optioned(
        path,
        &[],
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
        ],
        Some(is_stereo),
    )
    .await
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
/// use songbird::input;
///
/// let stereo_val = "2";
///
/// let streamer = futures::executor::block_on(input::ffmpeg_optioned("./some_file.mp3", &[], &[
///     "-f",
///     "s16le",
///     "-ac",
///     stereo_val,
///     "-ar",
///     "48000",
///     "-acodec",
///     "pcm_s16le",
///     "-",
/// ]));
///```
pub async fn ffmpeg_optioned<P: AsRef<OsStr>>(
    path: P,
    pre_input_args: &[&str],
    args: &[&str],
) -> Result<Input> {
    _ffmpeg_optioned(path.as_ref(), pre_input_args, args, None).await
}

pub(crate) async fn _ffmpeg_optioned(
    path: &OsStr,
    pre_input_args: &[&str],
    args: &[&str],
    is_stereo_known: Option<(bool, Metadata)>,
) -> Result<Input> {
    let (is_stereo, metadata) = if let Some(vals) = is_stereo_known {
        vals
    } else {
        is_stereo(path)
            .await
            .ok()
            .unwrap_or_else(|| (false, Default::default()))
    };

    let command = Command::new("ffmpeg")
        .args(pre_input_args)
        .arg("-i")
        .arg(path)
        .args(args)
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(Input::new(
        is_stereo,
        child_to_reader::<f32>(command),
        Codec::FloatPcm,
        Container::Raw,
        Some(metadata),
    ))
}

pub(crate) async fn is_stereo(path: &OsStr) -> Result<(bool, Metadata)> {
    let args = [
        "-v",
        "quiet",
        "-of",
        "json",
        "-show_format",
        "-show_streams",
        "-i",
    ];

    let out = TokioCommand::new("ffprobe")
        .args(&args)
        .arg(path)
        .stdin(Stdio::null())
        .output()
        .await?;

    let value: Value = serde_json::from_reader(&out.stdout[..])?;

    let metadata = Metadata::from_ffprobe_json(&value);

    debug!("FFprobe metadata {:?}", metadata);

    if let Some(count) = metadata.channels {
        Ok((count == 2, metadata))
    } else {
        Err(Error::Streams)
    }
}
