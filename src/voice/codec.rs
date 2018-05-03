use byteorder::{NetworkEndian, ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use internal::prelude::*;
use opus::{
    packet as opus_packet,
    Application as CodingMode,
    Bitrate,
    Channels,
    Decoder as OpusDecoder,
    Encoder as OpusEncoder,
    SoftClip,
};
use sodiumoxide::crypto::secretbox::{
    self,
    Key,
    MACBYTES,
    Nonce,
    NONCEBYTES,
};
use std::{
    collections::HashMap,
    io::{
        Error as IoError,
        Write,
    },
    net::SocketAddr,
};
use super::{
    audio::{
        DEFAULT_BITRATE,
        HEADER_LEN,
        SAMPLE_RATE,
    },
    streamer::{
        SendDecoder,
        SendEncoder,
    },
};
use tokio_core::net::UdpCodec;

pub(crate) enum VoicePacket<'a> {
    KeepAlive,
    Audio(&'a[f32], Bitrate),
}

pub(crate) struct VoiceCodec {
    decoder_map: HashMap<(u32, Channels), SendDecoder>,
    destination: SocketAddr,
    encoder: SendEncoder,
    key: Key,
    sequence: u16,
    ssrc: [u8; 4],
    timestamp: u32,
}

impl VoiceCodec {
    pub(crate) fn new(destination: SocketAddr, key: Key, ssrc_in: u32) -> Result<VoiceCodec> {
        let mut encoder = OpusEncoder::new(SAMPLE_RATE, Channels::Stereo, CodingMode::Audio)?;
                encoder.set_bitrate(Bitrate::Bits(DEFAULT_BITRATE))?;

        let mut out = VoiceCodec {
            decoder_map: HashMap::new(),
            destination,
            encoder: SendEncoder::new(encoder),
            key,
            sequence: 0,
            ssrc: [0u8; 4],
            timestamp: 0,
        };

        (&mut out.ssrc[..]).write_u32::<NetworkEndian>(ssrc_in)?;

        Ok(out)
    }
}

impl UdpCodec for VoiceCodec {
    type In = Vec<u8>;
    type Out = VoicePacket<'static>;

    // TODO: Implement!
    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> StdResult<Self::In, IoError> {
        Ok(vec![0u8])
    }

    // User will either send a heartbeat or audio of variable length.
    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        match msg {
            VoicePacket::KeepAlive => {
                buf.extend_from_slice(&self.ssrc);
            },
            VoicePacket::Audio(audio, bitrate) => {
                // Reconfigure encoder bitrate.
                // From my testing, it seemed like this needed to be set every cycle.
                if let Err(e) = self.encoder.set_bitrate(bitrate) {
                    warn!("[voice] Bitrate set unsuccessfully: {:?}", e);
                }

                let size = match bitrate {
                    // If user specified, we can calculate.
                    // bits -> bytes, then 20ms means 50fps.
                    Bitrate::Bits(b) => b.abs() / (8 * 50),
                    // Otherwise, just have a *lot* preallocated.
                    _ => 4096,
                } as usize;

                buf.reserve_exact(size);

                buf.extend_from_slice(&[0x80, 0x78]);
                buf.write_u16::<NetworkEndian>(self.sequence)
                    .expect("[voice] Cannot fail to append to Vec.");
                buf.write_u32::<NetworkEndian>(self.timestamp)
                    .expect("[voice] Cannot fail to append to Vec.");
                buf.extend_from_slice(&self.ssrc);
                buf.extend_from_slice(&[0u8; 12]);

                // the resize is free, because we already pre alloc'd.
                buf.resize(size, 0);

                let nonce = secretbox::Nonce::from_slice(&buf[..HEADER_LEN])
                    .expect("[voice] Nonce should be guaranteed from 24-byte slice.");

                let len = self.encoder.encode_float(audio, &mut buf[HEADER_LEN + MACBYTES..])
                    .expect("[voice] Encoding packet somehow failed.");

                // If sodiumoxide 0.1.16 worked on stable, then we could encrypt in place.
                // For now, we have to copy I guess...
                let crypt = secretbox::seal(&buf[HEADER_LEN..len], &nonce, &self.key);
                (&mut buf[HEADER_LEN..HEADER_LEN + MACBYTES]).write(&crypt);

                self.sequence = self.sequence.wrapping_add(1);
                self.timestamp = self.timestamp.wrapping_add(960);
            },
        }

        self.destination
    }
}