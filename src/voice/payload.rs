use serde_json::Value;
use super::connection_info::ConnectionInfo;
use ::constants::VoiceOpCode;

#[inline]
pub fn build_identify(info: &ConnectionInfo) -> Value {
    json!({
        "op": VoiceOpCode::Identify.num(),
        "d": {
            "server_id": info.guild_id.0,
            "session_id": &info.session_id,
            "token": &info.token,
            "user_id": info.user_id.0,
        }
    })
}

#[inline]
pub fn build_keepalive() -> Value {
    json!({
        "op": VoiceOpCode::KeepAlive.num(),
        "d": Value::Null,
    })
}

#[inline]
pub fn build_select_protocol(address: ::std::borrow::Cow<str>, port: u16) -> Value {
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
    })
}

#[inline]
pub fn build_speaking(speaking: bool) -> Value {
    json!({
        "op": VoiceOpCode::Speaking.num(),
        "d": {
            "delay": 0,
            "speaking": speaking,
        }
    })
}
