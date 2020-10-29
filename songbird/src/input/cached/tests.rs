use super::*;
use crate::{
    constants::*,
    input::{error::Error, ffmpeg, Codec, Container, Input, Reader},
    test_utils::*,
};
use audiopus::{coder::Decoder, Bitrate, Channels, SampleRate};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

#[tokio::test]
async fn streamcatcher_preserves_file() {
    let input = make_sine(50 * MONO_FRAME_SIZE, true);
    let input_len = input.len();

    let mut raw = default_config(raw_cost_per_sec(true))
        .build(Cursor::new(input.clone()))
        .map_err(Error::Streamcatcher)
        .unwrap();

    let mut out_buf = vec![];
    let read = raw.read_to_end(&mut out_buf).unwrap();

    assert_eq!(input_len, read);

    assert_eq!(input, out_buf);
}

#[test]
fn compressed_scans_frames_decodes_mono() {
    let data = one_s_compressed_sine(false);
    run_through_dca(data.raw);
}

#[test]
fn compressed_scans_frames_decodes_stereo() {
    let data = one_s_compressed_sine(true);
    run_through_dca(data.raw);
}

#[test]
fn compressed_triggers_valid_passthrough() {
    let mut input = Input::from(one_s_compressed_sine(true));

    assert!(input.supports_passthrough());

    let mut opus_buf = [0u8; 10_000];
    let mut signal_buf = [0i16; 1920];

    let opus_len = input.read_opus_frame(&mut opus_buf[..]).unwrap();

    let mut decoder = Decoder::new(SampleRate::Hz48000, Channels::Stereo).unwrap();
    decoder
        .decode(Some(&opus_buf[..opus_len]), &mut signal_buf[..], false)
        .unwrap();
}

fn one_s_compressed_sine(stereo: bool) -> Compressed {
    let data = make_sine(50 * MONO_FRAME_SIZE, stereo);

    let input = Input::new(stereo, data.into(), Codec::FloatPcm, Container::Raw, None);

    Compressed::new(input, Bitrate::BitsPerSecond(128_000)).unwrap()
}

fn run_through_dca(mut src: impl Read) {
    let mut decoder = Decoder::new(SampleRate::Hz48000, Channels::Stereo).unwrap();

    let mut pkt_space = [0u8; 10_000];
    let mut signals = [0i16; 1920];

    while let Ok(frame_len) = src.read_i16::<LittleEndian>() {
        let pkt_len = src.read(&mut pkt_space[..frame_len as usize]).unwrap();

        decoder
            .decode(Some(&pkt_space[..pkt_len]), &mut signals[..], false)
            .unwrap();
    }
}
