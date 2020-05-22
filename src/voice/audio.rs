use crate::model::event::VoiceSpeakingState;

/// A receiver for incoming audio.
pub trait AudioReceiver: Send {
    /// Fired on receipt of a speaking state update from another host.
    ///
    /// Note: this will fire when a user starts speaking for the first time,
    /// or changes their capabilities. 
    fn speaking_state_update(&mut self, _ssrc: u32, _user_id: u64, _speaking_state: VoiceSpeakingState) { }

    /// Fires when a source starts speaking, or stops speaking
    /// (*i.e.*, 5 consecutive silent frames).
    fn speaking_update(&mut self, _ssrc: u32, _speaking: bool) { }    

    #[allow(clippy::too_many_arguments)]
    /// Fires on receipt of a voice packet from another stream in the voice call.
    ///
    /// As RTP packets do not map to Discord's notion of users, SSRCs must be mapped
    /// back using the user IDs seen through client connection, disconnection,
    /// or speaking state update.
    fn voice_packet(&mut self,
                    _ssrc: u32,
                    _sequence: u16,
                    _timestamp: u32,
                    _stereo: bool,
                    _data: &[i16],
                    _compressed_size: usize) { }

    /// Fires on receipt of an RTCP packet, containing various call stats
    /// such as latency reports.
    fn rtcp_packet(&mut self, _data: u32) { }

    /// Fires whenever a user connects to the same stream as the bot.
    fn client_connect(&mut self, _ssrc: u32, _user_id: u64) { }

    /// Fires whenever a user disconnects from the same stream as the bot.
    fn client_disconnect(&mut self, _user_id: u64) { }
}
