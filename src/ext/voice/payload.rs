use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use super::connection_info::ConnectionInfo;
use ::constants::VoiceOpCode;

#[inline]
pub fn build_identify(info: &ConnectionInfo) -> Value {
    ObjectBuilder::new()
        .insert("op", VoiceOpCode::Identify.num())
        .insert_object("d", |o| o
            .insert("server_id", info.target_id)
            .insert("session_id", &info.session_id)
            .insert("token", &info.token)
            .insert("user_id", info.user_id.0))
        .build()
}

#[inline]
pub fn build_keepalive() -> Value {
    ObjectBuilder::new()
        .insert("op", VoiceOpCode::KeepAlive.num())
        .insert("d", Value::Null)
        .build()
}

#[inline]
pub fn build_select_protocol(address: ::std::borrow::Cow<str>, port: u16) -> Value {
    ObjectBuilder::new()
        .insert("op", VoiceOpCode::SelectProtocol.num())
        .insert_object("d", |o| o
            .insert("protocol", "udp")
            .insert_object("data", |o| o
                .insert("address", address)
                .insert("mode", super::CRYPTO_MODE)
                .insert("port", port)))
        .build()
}

#[inline]
pub fn build_speaking(speaking: bool) -> Value {
    ObjectBuilder::new()
        .insert("op", VoiceOpCode::Speaking.num())
        .insert_object("d", |o| o
            .insert("delay", 0)
            .insert("speaking", speaking))
        .build()
}
