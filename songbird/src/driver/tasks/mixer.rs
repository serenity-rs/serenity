use super::{
    error::{Error, Result},
    message::*,
};
use crate::{
    constants::*,
    tracks::{PlayMode, Track},
};
use audiopus::{
    coder::Encoder as OpusEncoder,
    softclip::SoftClip,
    Application as CodingMode,
    Bitrate,
    Channels,
};
use discortp::{
    rtp::{MutableRtpPacket, RtpPacket},
    MutablePacket,
    Packet,
};
use flume::{Receiver, Sender, TryRecvError};
use rand::random;
use spin_sleep::SpinSleeper;
use std::time::Instant;
use tokio::runtime::Handle;
use tracing::{debug, error};
use xsalsa20poly1305::{aead::AeadInPlace, Nonce, TAG_SIZE};

struct Mixer {
    async_handle: Handle,
    bitrate: Bitrate,
    conn_active: Option<MixerConnection>,
    deadline: Instant,
    encoder: OpusEncoder,
    mix_rx: Receiver<MixerMessage>,
    muted: bool,
    packet: [u8; VOICE_PACKET_MAX],
    silence_frames: u8,
    sleeper: SpinSleeper,
    soft_clip: SoftClip,
    tracks: Vec<Track>,
    ws: Option<Sender<WsMessage>>,
}

fn new_encoder(bitrate: Bitrate) -> Result<OpusEncoder> {
    let mut encoder = OpusEncoder::new(SAMPLE_RATE, Channels::Stereo, CodingMode::Audio)?;
    encoder.set_bitrate(bitrate)?;

    Ok(encoder)
}

impl Mixer {
    fn new(mix_rx: Receiver<MixerMessage>, async_handle: Handle) -> Self {
        let bitrate = DEFAULT_BITRATE;
        let encoder = new_encoder(bitrate)
            .expect("Failed to create encoder in mixing thread with known-good values.");
        let soft_clip = SoftClip::new(Channels::Stereo);

        let mut packet = [0u8; VOICE_PACKET_MAX];

        let mut rtp = MutableRtpPacket::new(&mut packet[..]).expect(
            "Too few bytes in self.packet for RTP header.\
                (Blame: VOICE_PACKET_MAX?)",
        );
        rtp.set_version(RTP_VERSION);
        rtp.set_payload_type(RTP_PROFILE_TYPE);
        rtp.set_sequence(random::<u16>().into());
        rtp.set_timestamp(random::<u32>().into());

        Self {
            async_handle,
            bitrate,
            conn_active: None,
            deadline: Instant::now(), // FIXME: refresh on connection start
            encoder,
            mix_rx,
            muted: false,
            packet,
            silence_frames: 0, // FIXME: shouldn't this start at 5? Test.
            sleeper: Default::default(),
            soft_clip,
            tracks: vec![],
            ws: None,
        }
    }

    fn run(&mut self, mut interconnect: Interconnect) {
        'runner: loop {
            loop {
                use MixerMessage::*;

                match self.mix_rx.try_recv() {
                    Ok(AddTrack(mut t)) => {
                        t.source.prep_with_handle(self.async_handle.clone());
                        let _ = self.add_track(t, &interconnect);
                    },
                    Ok(SetTrack(t)) => {
                        self.tracks.clear();
                        let _ = interconnect.events.send(EventMessage::RemoveAllTracks);
                        if let Some(mut t) = t {
                            t.source.prep_with_handle(self.async_handle.clone());
                            let _ = self.add_track(t, &interconnect);
                        }
                    },
                    Ok(SetBitrate(b)) => {
                        self.bitrate = b;
                        if let Err(e) = self.set_bitrate(b) {
                            error!("Failed to update bitrate {:?}", e);
                        }
                    },
                    Ok(SetMute(m)) => {
                        self.muted = m;
                    },
                    Ok(SetConn(conn, ssrc)) => {
                        self.conn_active = Some(conn);
                        let mut rtp = MutableRtpPacket::new(&mut self.packet[..]).expect(
                            "Too few bytes in self.packet for RTP header.\
                                (Blame: VOICE_PACKET_MAX?)",
                        );
                        rtp.set_ssrc(ssrc);
                    },
                    Ok(DropConn) => {
                        self.conn_active = None;
                    },
                    Ok(ReplaceInterconnect(i)) => {
                        if let Some(ws) = &self.ws {
                            let _ = ws.send(WsMessage::ReplaceInterconnect(i.clone()));
                        }
                        interconnect = i;
                    },
                    Ok(RebuildEncoder) => match new_encoder(self.bitrate) {
                        Ok(encoder) => {
                            self.encoder = encoder;
                        },
                        Err(e) => {
                            error!("Failed to rebuild encoder. Resetting bitrate. {:?}", e);
                            self.bitrate = DEFAULT_BITRATE;
                            self.encoder = new_encoder(self.bitrate)
                                .expect("Failed fallback rebuild of OpusEncoder with safe inputs.");
                        },
                    },
                    Ok(Ws(new_ws_handle)) => {
                        self.ws = new_ws_handle;
                    },

                    Err(TryRecvError::Disconnected) | Ok(Poison) => {
                        break 'runner;
                    },

                    Err(TryRecvError::Empty) => {
                        break;
                    },
                }
            }

            if let Err(e) = self.cycle(&interconnect) {
                if matches!(e, Error::InterconnectFailure(_)) {
                    let _ = interconnect.core.send(CoreMessage::RebuildInterconnect);
                }

                error!("Mixer thread cycle: {:?}", e);

                let _ = interconnect.core.send(CoreMessage::Reconnect);
            } else {
                self.audio_commands_events(&interconnect);
            }
        }
    }

    #[inline]
    fn add_track(&mut self, mut track: Track, interconnect: &Interconnect) -> Result<()> {
        let evts = track.events.take().unwrap_or_default();
        let state = track.state();
        let handle = track.handle.clone();

        self.tracks.push(track);

        interconnect
            .events
            .send(EventMessage::AddTrack(evts, state, handle))?;

        Ok(())
    }

    #[inline]
    fn mix_tracks<'a>(
        &mut self,
        opus_frame: &'a mut [u8],
        mix_buffer: &mut [f32; STEREO_FRAME_SIZE],
        interconnect: &Interconnect,
    ) -> Result<(usize, &'a [u8])> {
        let mut len = 0;

        // Opus frame passthrough.
        // This requires that we have only one track, who has volume 1.0, and an
        // Opus codec type.
        let do_passthrough = self.tracks.len() == 1 && {
            let track = &self.tracks[0];
            (track.volume - 1.0).abs() < f32::EPSILON && track.source.supports_passthrough()
        };

        if do_passthrough {
            let track = &mut self.tracks[0];
            if track.playing == PlayMode::Play {
                if let Ok(opus_len) = track.source.read_opus_frame(opus_frame) {
                    track.step_frame();
                    Ok((STEREO_FRAME_SIZE, &opus_frame[..opus_len]))
                } else {
                    if track.do_loop() {
                        if let Some(time) = track.seek_time(Default::default()) {
                            let _ = interconnect.events.send(EventMessage::ChangeState(
                                0,
                                TrackStateChange::Position(time),
                            ));
                            let _ = interconnect.events.send(EventMessage::ChangeState(
                                0,
                                TrackStateChange::Loops(track.loops, false),
                            ));
                        }
                    } else {
                        track.end();
                    }

                    Ok((0, &opus_frame[..0]))
                }
            } else {
                Ok((0, &opus_frame[..0]))
            }
        } else {
            for (i, track) in self.tracks.iter_mut().enumerate() {
                let vol = track.volume;
                let stream = &mut track.source;

                if track.playing != PlayMode::Play {
                    continue;
                }

                let temp_len = stream.mix(mix_buffer, vol);

                len = len.max(temp_len);
                if temp_len > 0 {
                    track.step_frame();
                } else if track.do_loop() {
                    if let Some(time) = track.seek_time(Default::default()) {
                        let _ = interconnect.events.send(EventMessage::ChangeState(
                            i,
                            TrackStateChange::Position(time),
                        ));
                        let _ = interconnect.events.send(EventMessage::ChangeState(
                            i,
                            TrackStateChange::Loops(track.loops, false),
                        ));
                    }
                } else {
                    track.end();
                }
            }

            Ok((len, &opus_frame[..0]))
        }
    }

    #[inline]
    fn audio_commands_events(&mut self, interconnect: &Interconnect) {
        // Apply user commands.
        for (i, track) in self.tracks.iter_mut().enumerate() {
            track.process_commands(i, interconnect);
        }

        // FIXME: do without vec?
        let mut i = 0;
        let mut to_remove = Vec::with_capacity(self.tracks.len());
        while i < self.tracks.len() {
            let track = self
                .tracks
                .get_mut(i)
                .expect("Tried to remove an illegal track index.");

            if track.playing.is_done() {
                let p_state = track.playing();
                self.tracks.remove(i);
                to_remove.push(i);
                let _ = interconnect.events.send(EventMessage::ChangeState(
                    i,
                    TrackStateChange::Mode(p_state),
                ));
            } else {
                i += 1;
            }
        }

        // Tick
        let _ = interconnect.events.send(EventMessage::Tick);

        // Then do removals.
        for i in &to_remove[..] {
            let _ = interconnect.events.send(EventMessage::RemoveTrack(*i));
        }
    }

    #[inline]
    fn march_deadline(&mut self) {
        self.sleeper
            .sleep(self.deadline.saturating_duration_since(Instant::now()));
        self.deadline += TIMESTEP_LENGTH;
    }

    fn cycle(&mut self, interconnect: &Interconnect) -> Result<()> {
        if self.conn_active.is_none() {
            self.march_deadline();
            return Ok(());
        }

        let mut opus_frame_backing = [0u8; STEREO_FRAME_SIZE];
        let mut mix_buffer = [0f32; STEREO_FRAME_SIZE];

        // Slice which mix tracks may use to passthrough direct Opus frames.
        let mut opus_space = &mut opus_frame_backing[..];

        // Walk over all the audio files, combining into one audio frame according
        // to volume, play state, etc.
        let (mut len, mut opus_frame) =
            self.mix_tracks(&mut opus_space, &mut mix_buffer, interconnect)?;

        self.soft_clip.apply(&mut mix_buffer[..])?;

        if self.muted {
            len = 0;
        }

        if len == 0 {
            if self.silence_frames > 0 {
                self.silence_frames -= 1;

                // Explicit "Silence" frame.
                opus_frame = &SILENT_FRAME[..];
            } else {
                // Per official guidelines, send 5x silence BEFORE we stop speaking.
                if let Some(ws) = &self.ws {
                    // NOTE: this should prevent a catastrophic thread pileup.
                    // A full reconnect might cause an inner closed connection.
                    // It's safer to leave the central task to clean this up and
                    // pass the mixer a new channel.
                    let _ = ws.send(WsMessage::Speaking(false));
                }

                self.march_deadline();

                return Ok(());
            }
        } else {
            self.silence_frames = 5;
        }

        if let Some(ws) = &self.ws {
            ws.send(WsMessage::Speaking(true))?;
        }

        self.march_deadline();
        self.prep_and_send_packet(mix_buffer, opus_frame)?;

        Ok(())
    }

    fn set_bitrate(&mut self, bitrate: Bitrate) -> Result<()> {
        self.encoder.set_bitrate(bitrate).map_err(Into::into)
    }

    fn prep_and_send_packet(&mut self, buffer: [f32; 1920], opus_frame: &[u8]) -> Result<()> {
        let conn = self
            .conn_active
            .as_mut()
            .expect("Shouldn't be mixing packets without access to a cipher + UDP dest.");

        let mut nonce = Nonce::default();
        let index = {
            let mut rtp = MutableRtpPacket::new(&mut self.packet[..]).expect(
                "Too few bytes in self.packet for RTP header.\
                    (Blame: VOICE_PACKET_MAX?)",
            );

            let pkt = rtp.packet();
            let rtp_len = RtpPacket::minimum_packet_size();
            nonce[..rtp_len].copy_from_slice(&pkt[..rtp_len]);

            let payload = rtp.payload_mut();

            let payload_len = if opus_frame.is_empty() {
                self.encoder
                    .encode_float(&buffer[..STEREO_FRAME_SIZE], &mut payload[TAG_SIZE..])?
            } else {
                let len = opus_frame.len();
                payload[TAG_SIZE..TAG_SIZE + len].clone_from_slice(opus_frame);
                len
            };

            let final_payload_size = TAG_SIZE + payload_len;

            let tag = conn
                .cipher
                .encrypt_in_place_detached(&nonce, b"", &mut payload[TAG_SIZE..final_payload_size])
                .expect("Encryption failed?");
            payload[..TAG_SIZE].copy_from_slice(&tag[..]);

            rtp_len + final_payload_size
        };

        // FIXME: This is dog slow, don't do this.
        conn.udp_tx
            .send(UdpTxMessage::Packet(self.packet[..index].to_vec()))?;

        let mut rtp = MutableRtpPacket::new(&mut self.packet[..]).expect(
            "Too few bytes in self.packet for RTP header.\
                (Blame: VOICE_PACKET_MAX?)",
        );
        rtp.set_sequence(rtp.get_sequence() + 1);
        rtp.set_timestamp(rtp.get_timestamp() + MONO_FRAME_SIZE as u32);

        Ok(())
    }
}

/// The mixing thread is a synchronous contect due to its compute-bound nature.
///
/// We pass in an async handle for the benefit of some Input classes (e.g., restartables)
/// who need to run their restart code elsewhere and return blank data until such time.
pub(crate) fn runner(
    interconnect: Interconnect,
    mix_rx: Receiver<MixerMessage>,
    async_handle: Handle,
) {
    let mut mixer = Mixer::new(mix_rx, async_handle);

    mixer.run(interconnect);
}
