#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Voice core events occur on receipt of
/// voice packets and telemetry.
///
/// Core events persist while the `action` in [`EventData`]
/// returns `None`.
///
/// [`EventData`]: struct.EventData.html
pub enum CoreEvent {
    /// Fired on receipt of a speaking state update from another host.
    ///
    /// Note: this will fire when a user starts speaking for the first time,
    /// or changes their capabilities.
    SpeakingStateUpdate,
    /// Fires when a source starts speaking, or stops speaking
    /// (*i.e.*, 5 consecutive silent frames).
    SpeakingUpdate,
    /// Fires on receipt of a voice packet from another stream in the voice call.
    ///
    /// As RTP packets do not map to Discord's notion of users, SSRCs must be mapped
    /// back using the user IDs seen through client connection, disconnection,
    /// or speaking state update.
    VoicePacket,
    /// Fires on receipt of an RTCP packet, containing various call stats
    /// such as latency reports.
    RtcpPacket,
    /// Fires whenever a user connects to the same stream as the bot.
    ClientConnect,
    /// Fires whenever a user disconnects from the same stream as the bot.
    ClientDisconnect,
}
