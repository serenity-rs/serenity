use constants::VoiceOpCode;
use serde_json::Value;
use super::connection_info::ConnectionInfo;
use tungstenite::Message;

#[inline]
pub fn build_identify(info: &ConnectionInfo) -> Message {
    Message::text(
        json!({
            "op": VoiceOpCode::Identify.num(),
            "d": {
                "server_id": info.guild_id.0,
                "session_id": &info.session_id,
                "token": &info.token,
                "user_id": info.user_id.0,
            }
        }).to_string()
    )
}

#[inline]
pub fn build_heartbeat(nonce: u64) -> Message {
    Message::text(
        json!({
            "op": VoiceOpCode::Heartbeat.num(),
            "d": nonce,
        }).to_string()
    )
}

#[inline]
pub fn build_select_protocol(address: ::std::borrow::Cow<str>, port: u16) -> Message {
    Message::text(
        json!({
            "op": VoiceOpCode::SelectProtocol.num(),
            "d": {
                "protocol": "udp",
                "data": {
                    "address": address,
                    "mode": super::CRYPTO_MODE,
                    "port": port,
                }
            }
        }).to_string()
    )
}

#[inline]
pub fn build_speaking(speaking: bool) -> Message {
    Message::text(
        json!({
            "op": VoiceOpCode::Speaking.num(),
            "d": {
                "delay": 0,
                "speaking": speaking,
            }
        }).to_string()
    )
}
