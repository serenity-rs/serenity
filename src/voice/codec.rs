use byteorder::{ByteOrder, NetworkEndian, ReadBytesExt, WriteBytesExt};
use internal::prelude::*;
use opus::{
    packet as opus_packet,
    Application as CodingMode,
    Bitrate,
    Channels,
    Decoder as OpusDecoder,
    Encoder as OpusEncoder,
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
        ErrorKind as IoErrorKind,
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
        let mut encoder = OpusEncoder::new(SAMPLE_RATE, Channels::Stereo, CodingMode::Audio)
            .map(SendEncoder::new)?;

        encoder.set_bitrate(Bitrate::Bits(DEFAULT_BITRATE))?;

        let mut out = VoiceCodec {
            decoder_map: HashMap::new(),
            destination,
            encoder: encoder,
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
    type In = (u32, u16, u32, bool, &'static[i16]);
    type Out = VoicePacket<'static>;

    // TODO: Implement!
    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> StdResult<Self::In, IoError> {
        let mut buffer = [0i16; 960 * 2];

        let nonce = Nonce::from_slice(&buf[..NONCEBYTES])
            .ok_or(IoErrorKind::InvalidData)?;

        let mut handle = &buf[2..];
        let seq = handle.read_u16::<NetworkEndian>()?;
        let timestamp = handle.read_u32::<NetworkEndian>()?;
        let ssrc = handle.read_u32::<NetworkEndian>()?;

        secretbox::open(&buf[HEADER_LEN..], &nonce, &self.key)
            .and_then(|decrypted| {
                let channels = opus_packet::get_nb_channels(&decrypted)
                    .or(Err(()))?;

                let entry =
                    self.decoder_map.entry((ssrc, channels)).or_insert_with(
                        || OpusDecoder::new(SAMPLE_RATE, channels)
                            .map(SendDecoder::new)
                            .expect("[voice] Decoder construction should never fail.")
                    );

                // Strip RTP Header Extensions (one-byte)
                if decrypted[0] == 0xBE && decrypted[1] == 0xDE {
                    // Read the length bytes as a big-endian u16.
                    let header_extension_len = NetworkEndian::read_u16(&decrypted[2..4]);
                    let mut offset = 4;
                    for _ in 0..header_extension_len {
                        let byte = decrypted[offset];
                        offset += 1;
                        if byte == 0 {
                            continue;
                        }

                        offset += 1 + (0b1111 & (byte >> 4)) as usize;
                    }

                    while decrypted[offset] == 0 {
                        offset += 1;
                    }

                    decrypted = decrypted.split_off(offset);
                }

                let len = entry.decode(&decrypted, &mut buffer, false)
                    .or(Err(()))?;

                let is_stereo = channels == Channels::Stereo;

                let b = if is_stereo { len * 2 } else { len };

                Ok((ssrc, seq, timestamp, is_stereo, &buffer[..b]))
            })
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "[voice] Couldn't decode Opus frames."))
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

                let nonce = Nonce::from_slice(&buf[..HEADER_LEN])
                    .expect("[voice] Nonce should be guaranteed from 24-byte slice.");

                let len = self.encoder.encode_float(audio, &mut buf[HEADER_LEN + MACBYTES..])
                    .expect("[voice] Encoding packet somehow failed.");

                // If sodiumoxide 0.1.16 worked on stable, then we could encrypt in place.
                // For now, we have to copy I guess...
                // Unless someone's willing to play with unsafe wizardy.
                let crypt = secretbox::seal(&buf[HEADER_LEN..len], &nonce, &self.key);
                (&mut buf[HEADER_LEN..HEADER_LEN + MACBYTES]).write(&crypt);

                self.sequence = self.sequence.wrapping_add(1);
                self.timestamp = self.timestamp.wrapping_add(960);
            },
        }

        self.destination
    }
}