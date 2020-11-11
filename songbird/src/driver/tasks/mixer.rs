use super::{error::Result, message::*, Config};
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
};
use flume::{Receiver, Sender, TryRecvError};
use rand::random;
use spin_sleep::SpinSleeper;
use std::time::Instant;
use tokio::runtime::Handle;
use tracing::{error, instrument};
use xsalsa20poly1305::TAG_SIZE;

struct Mixer {
    async_handle: Handle,
    bitrate: Bitrate,
    config: Config,
    conn_active: Option<MixerConnection>,
    deadline: Instant,
    encoder: OpusEncoder,
    interconnect: Interconnect,
    mix_rx: Receiver<MixerMessage>,
    muted: bool,
    packet: [u8; VOICE_PACKET_MAX],
    prevent_events: bool,
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
    fn new(
        mix_rx: Receiver<MixerMessage>,
        async_handle: Handle,
        interconnect: Interconnect,
        config: Config,
    ) -> Self {
        let bitrate = DEFAULT_BITRATE;
        let encoder = new_encoder(bitrate)
            .expect("Failed to create encoder in mixing thread with known-good values.");
        let soft_clip = SoftClip::new(Channels::Stereo);

        let mut packet = [0u8; VOICE_PACKET_MAX];

        let mut rtp = MutableRtpPacket::new(&mut packet[..]).expect(
            "FATAL: Too few bytes in self.packet for RTP header.\
                (Blame: VOICE_PACKET_MAX?)",
        );
        rtp.set_version(RTP_VERSION);
        rtp.set_payload_type(RTP_PROFILE_TYPE);
        rtp.set_sequence(random::<u16>().into());
        rtp.set_timestamp(random::<u32>().into());

        let tracks = Vec::with_capacity(1.max(config.preallocated_tracks));

        Self {
            async_handle,
            bitrate,
            config,
            conn_active: None,
            deadline: Instant::now(),
            encoder,
            interconnect,
            mix_rx,
            muted: false,
            packet,
            prevent_events: false,
            silence_frames: 0,
            sleeper: Default::default(),
            soft_clip,
            tracks,
            ws: None,
        }
    }

    fn run(&mut self) {
        let mut events_failure = false;
        let mut conn_failure = false;

        'runner: loop {
            loop {
                use MixerMessage::*;

                let error = match self.mix_rx.try_recv() {
                    Ok(AddTrack(mut t)) => {
                        t.source.prep_with_handle(self.async_handle.clone());
                        self.add_track(t)
                    },
                    Ok(SetTrack(t)) => {
                        self.tracks.clear();

                        let mut out = self.fire_event(EventMessage::RemoveAllTracks);

                        if let Some(mut t) = t {
                            t.source.prep_with_handle(self.async_handle.clone());

                            // Do this unconditionally: this affects local state infallibly,
                            // with the event installation being the remote part.
                            if let Err(e) = self.add_track(t) {
                                out = Err(e);
                            }
                        }

                        out
                    },
                    Ok(SetBitrate(b)) => {
                        self.bitrate = b;
                        if let Err(e) = self.set_bitrate(b) {
                            error!("Failed to update bitrate {:?}", e);
                        }
                        Ok(())
                    },
                    Ok(SetMute(m)) => {
                        self.muted = m;
                        Ok(())
                    },
                    Ok(SetConn(conn, ssrc)) => {
                        self.conn_active = Some(conn);
                        let mut rtp = MutableRtpPacket::new(&mut self.packet[..]).expect(
                            "Too few bytes in self.packet for RTP header.\
                                (Blame: VOICE_PACKET_MAX?)",
                        );
                        rtp.set_ssrc(ssrc);
                        rtp.set_sequence(random::<u16>().into());
                        rtp.set_timestamp(random::<u32>().into());
                        self.deadline = Instant::now();
                        Ok(())
                    },
                    Ok(DropConn) => {
                        self.conn_active = None;
                        Ok(())
                    },
                    Ok(ReplaceInterconnect(i)) => {
                        self.prevent_events = false;
                        if let Some(ws) = &self.ws {
                            conn_failure |=
                                ws.send(WsMessage::ReplaceInterconnect(i.clone())).is_err();
                        }
                        if let Some(conn) = &self.conn_active {
                            conn_failure |= conn
                                .udp_rx
                                .send(UdpRxMessage::ReplaceInterconnect(i.clone()))
                                .is_err();
                        }
                        self.interconnect = i;

                        self.rebuild_tracks()
                    },
                    Ok(SetConfig(new_config)) => {
                        self.config = new_config.clone();

                        if self.tracks.capacity() < self.config.preallocated_tracks {
                            self.tracks
                                .reserve(self.config.preallocated_tracks - self.tracks.len());
                        }

                        if let Some(conn) = &self.conn_active {
                            conn_failure |= conn
                                .udp_rx
                                .send(UdpRxMessage::SetConfig(new_config))
                                .is_err();
                        }

                        Ok(())
                    },
                    Ok(RebuildEncoder) => match new_encoder(self.bitrate) {
                        Ok(encoder) => {
                            self.encoder = encoder;
                            Ok(())
                        },
                        Err(e) => {
                            error!("Failed to rebuild encoder. Resetting bitrate. {:?}", e);
                            self.bitrate = DEFAULT_BITRATE;
                            self.encoder = new_encoder(self.bitrate)
                                .expect("Failed fallback rebuild of OpusEncoder with safe inputs.");
                            Ok(())
                        },
                    },
                    Ok(Ws(new_ws_handle)) => {
                        self.ws = new_ws_handle;
                        Ok(())
                    },

                    Err(TryRecvError::Disconnected) | Ok(Poison) => {
                        break 'runner;
                    },

                    Err(TryRecvError::Empty) => {
                        break;
                    },
                };

                if let Err(e) = error {
                    events_failure |= e.should_trigger_interconnect_rebuild();
                    conn_failure |= e.should_trigger_connect();
                }
            }

            if let Err(e) = self.cycle().and_then(|_| self.audio_commands_events()) {
                events_failure |= e.should_trigger_interconnect_rebuild();
                conn_failure |= e.should_trigger_connect();

                error!("Mixer thread cycle: {:?}", e);
            }

            // event failure? rebuild interconnect.
            // ws or udp failure? full connect
            // (soft reconnect is covered by the ws task.)
            if events_failure {
                self.prevent_events = true;
                self.interconnect
                    .core
                    .send(CoreMessage::RebuildInterconnect)
                    .expect("FATAL: No way to rebuild driver core from mixer.");
                events_failure = false;
            }

            if conn_failure {
                self.interconnect
                    .core
                    .send(CoreMessage::FullReconnect)
                    .expect("FATAL: No way to rebuild driver core from mixer.");
                conn_failure = false;
            }
        }
    }

    #[inline]
    fn fire_event(&self, event: EventMessage) -> Result<()> {
        // As this task is responsible for noticing the potential death of an event context,
        // it's responsible for not forcibly recreating said context repeatedly.
        if !self.prevent_events {
            self.interconnect.events.send(event)?;
            Ok(())
        } else {
            Ok(())
        }
    }

    #[inline]
    fn add_track(&mut self, mut track: Track) -> Result<()> {
        let evts = track.events.take().unwrap_or_default();
        let state = track.state();
        let handle = track.handle.clone();

        self.tracks.push(track);

        self.interconnect
            .events
            .send(EventMessage::AddTrack(evts, state, handle))?;

        Ok(())
    }

    // rebuilds the event thread's view of each track, in event of a full rebuild.
    #[inline]
    fn rebuild_tracks(&mut self) -> Result<()> {
        for track in self.tracks.iter_mut() {
            let evts = track.events.take().unwrap_or_default();
            let state = track.state();
            let handle = track.handle.clone();

            self.interconnect
                .events
                .send(EventMessage::AddTrack(evts, state, handle))?;
        }

        Ok(())
    }

    #[inline]
    fn mix_tracks<'a>(
        &mut self,
        opus_frame: &'a mut [u8],
        mix_buffer: &mut [f32; STEREO_FRAME_SIZE],
    ) -> Result<(usize, &'a [u8])> {
        let mut len = 0;

        // Opus frame passthrough.
        // This requires that we have only one track, who has volume 1.0, and an
        // Opus codec type.
        let do_passthrough = self.tracks.len() == 1 && {
            let track = &self.tracks[0];
            (track.volume - 1.0).abs() < f32::EPSILON && track.source.supports_passthrough()
        };

        for (i, track) in self.tracks.iter_mut().enumerate() {
            let vol = track.volume;
            let stream = &mut track.source;

            if track.playing != PlayMode::Play {
                continue;
            }

            let (temp_len, opus_len) = if do_passthrough {
                (0, track.source.read_opus_frame(opus_frame).ok())
            } else {
                (stream.mix(mix_buffer, vol), None)
            };

            len = len.max(temp_len);
            if temp_len > 0 || opus_len.is_some() {
                track.step_frame();
            } else if track.do_loop() {
                if let Some(time) = track.seek_time(Default::default()) {
                    // have to reproduce self.fire_event here
                    // to circumvent the borrow checker's lack of knowledge.
                    //
                    // In event of error, one of the later event calls will
                    // trigger the event thread rebuild: it is more prudent that
                    // the mixer works as normal right now.
                    if !self.prevent_events {
                        let _ = self.interconnect.events.send(EventMessage::ChangeState(
                            i,
                            TrackStateChange::Position(time),
                        ));
                        let _ = self.interconnect.events.send(EventMessage::ChangeState(
                            i,
                            TrackStateChange::Loops(track.loops, false),
                        ));
                    }
                }
            } else {
                track.end();
            }

            if let Some(opus_len) = opus_len {
                return Ok((STEREO_FRAME_SIZE, &opus_frame[..opus_len]));
            }
        }

        Ok((len, &opus_frame[..0]))
    }

    #[inline]
    fn audio_commands_events(&mut self) -> Result<()> {
        // Apply user commands.
        for (i, track) in self.tracks.iter_mut().enumerate() {
            // This causes fallible event system changes,
            // but if the event thread has died then we'll certainly
            // detect that on the tick later.
            // Changes to play state etc. MUST all be handled.
            track.process_commands(i, &self.interconnect);
        }

        // TODO: do without vec?
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
                self.fire_event(EventMessage::ChangeState(
                    i,
                    TrackStateChange::Mode(p_state),
                ))?;
            } else {
                i += 1;
            }
        }

        // Tick
        self.fire_event(EventMessage::Tick)?;

        // Then do removals.
        for i in &to_remove[..] {
            self.fire_event(EventMessage::RemoveTrack(*i))?;
        }

        Ok(())
    }

    #[inline]
    fn march_deadline(&mut self) {
        self.sleeper
            .sleep(self.deadline.saturating_duration_since(Instant::now()));
        self.deadline += TIMESTEP_LENGTH;
    }

    fn cycle(&mut self) -> Result<()> {
        if self.conn_active.is_none() {
            self.march_deadline();
            return Ok(());
        }

        // TODO: can we make opus_frame_backing *actually* a view over
        // some region of self.packet, derived using the encryption mode?
        // This saves a copy on Opus passthrough.
        let mut opus_frame_backing = [0u8; STEREO_FRAME_SIZE];
        let mut mix_buffer = [0f32; STEREO_FRAME_SIZE];

        // Slice which mix tracks may use to passthrough direct Opus frames.
        let mut opus_space = &mut opus_frame_backing[..];

        // Walk over all the audio files, combining into one audio frame according
        // to volume, play state, etc.
        let (mut len, mut opus_frame) = self.mix_tracks(&mut opus_space, &mut mix_buffer)?;

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

        let index = {
            let mut rtp = MutableRtpPacket::new(&mut self.packet[..]).expect(
                "FATAL: Too few bytes in self.packet for RTP header.\
                    (Blame: VOICE_PACKET_MAX?)",
            );

            let payload = rtp.payload_mut();
            let crypto_mode = conn.crypto_state.kind();

            let payload_len = if opus_frame.is_empty() {
                let total_payload_space = payload.len() - crypto_mode.payload_suffix_len();
                self.encoder.encode_float(
                    &buffer[..STEREO_FRAME_SIZE],
                    &mut payload[TAG_SIZE..total_payload_space],
                )?
            } else {
                let len = opus_frame.len();
                payload[TAG_SIZE..TAG_SIZE + len].clone_from_slice(opus_frame);
                len
            };

            let final_payload_size = conn
                .crypto_state
                .write_packet_nonce(&mut rtp, TAG_SIZE + payload_len);

            conn.crypto_state.kind().encrypt_in_place(
                &mut rtp,
                &conn.cipher,
                final_payload_size,
            )?;

            RtpPacket::minimum_packet_size() + final_payload_size
        };

        // TODO: This is dog slow, don't do this.
        // Can we replace this with a shared ring buffer + semaphore?
        // i.e., do something like double/triple buffering in graphics.
        conn.udp_tx
            .send(UdpTxMessage::Packet(self.packet[..index].to_vec()))?;

        let mut rtp = MutableRtpPacket::new(&mut self.packet[..]).expect(
            "FATAL: Too few bytes in self.packet for RTP header.\
                (Blame: VOICE_PACKET_MAX?)",
        );
        rtp.set_sequence(rtp.get_sequence() + 1);
        rtp.set_timestamp(rtp.get_timestamp() + MONO_FRAME_SIZE as u32);

        Ok(())
    }
}

/// The mixing thread is a synchronous context due to its compute-bound nature.
///
/// We pass in an async handle for the benefit of some Input classes (e.g., restartables)
/// who need to run their restart code elsewhere and return blank data until such time.
#[instrument(skip(interconnect, mix_rx, async_handle))]
pub(crate) fn runner(
    interconnect: Interconnect,
    mix_rx: Receiver<MixerMessage>,
    async_handle: Handle,
    config: Config,
) {
    let mut mixer = Mixer::new(mix_rx, async_handle, interconnect, config);

    mixer.run();
}
