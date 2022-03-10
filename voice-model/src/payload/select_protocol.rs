use crate::protocol_data::ProtocolData;
use serde::{Deserialize, Serialize};

/// Used to select the protocol and encryption mechanism.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(tag = "protocol")]
pub enum SelectProtocol {
    #[serde(rename = "udp")]
    UDP { data: ProtocolData },
    #[serde(rename = "webrtc")]
    WebRTC {
        data: String,
        sdp: String,
        codecs: Vec<WebRTCCodec>,
        rtc_connection_id: String,
    },
}

#[test]
fn selectprotocol_udp() {
    use crate::Event;
    let test_udp = r#"
    {
        "op": 1,
        "d": {
            "protocol": "udp",
            "data": {
                "address": "0.0.0.0",
                "mode": "what",
                "port": 50000
            }
        }
    }
    "#;
    let _udp_expected = Event::SelectProtocol(SelectProtocol::UDP {
        data: ProtocolData {
            address: [0, 0, 0, 0].into(),
            mode: "what".to_string(),
            port: 50000,
        },
    });
    let test_udp_res: Event = serde_json::from_str(test_udp).unwrap();
    matches!(test_udp_res, _udp_expected);
}

#[test]
fn selectprotocol_webrtc() {
    use crate::Event;
    let test_webrtc = r#"
    {
        "op": 1,
        "d": {
            "protocol": "webrtc",
            "data": "a=extmap-allow-mixed\na=ice-ufrag:nEdg\na=ice-pwd:cGgW+G3KQbmO9ptaZfqffUQi\na=ice-options:trickle\na=extmap:1 urn:ietf:params:rtp-hdrext:ssrc-audio-level\na=extmap:2 http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time\na=extmap:3 http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01\na=extmap:4 urn:ietf:params:rtp-hdrext:sdes:mid\na=extmap:5 urn:ietf:params:rtp-hdrext:sdes:rtp-stream-id\na=extmap:6 urn:ietf:params:rtp-hdrext:sdes:repaired-rtp-stream-id\na=rtpmap:111 opus/48000/2\na=extmap:14 urn:ietf:params:rtp-hdrext:toffset\na=extmap:13 urn:3gpp:video-orientation\na=extmap:12 http://www.webrtc.org/experiments/rtp-hdrext/playout-delay\na=extmap:11 http://www.webrtc.org/experiments/rtp-hdrext/video-content-type\na=extmap:7 http://www.webrtc.org/experiments/rtp-hdrext/video-timing\na=extmap:8 http://www.webrtc.org/experiments/rtp-hdrext/color-space\na=rtpmap:96 VP8/90000\na=rtpmap:97 rtx/90000",
            "sdp": "a=extmap-allow-mixed\na=ice-ufrag:nEdg\na=ice-pwd:cGgW+G3KQbmO9ptaZfqffUQi\na=ice-options:trickle\na=extmap:1 urn:ietf:params:rtp-hdrext:ssrc-audio-level\na=extmap:2 http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time\na=extmap:3 http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01\na=extmap:4 urn:ietf:params:rtp-hdrext:sdes:mid\na=extmap:5 urn:ietf:params:rtp-hdrext:sdes:rtp-stream-id\na=extmap:6 urn:ietf:params:rtp-hdrext:sdes:repaired-rtp-stream-id\na=rtpmap:111 opus/48000/2\na=extmap:14 urn:ietf:params:rtp-hdrext:toffset\na=extmap:13 urn:3gpp:video-orientation\na=extmap:12 http://www.webrtc.org/experiments/rtp-hdrext/playout-delay\na=extmap:11 http://www.webrtc.org/experiments/rtp-hdrext/video-content-type\na=extmap:7 http://www.webrtc.org/experiments/rtp-hdrext/video-timing\na=extmap:8 http://www.webrtc.org/experiments/rtp-hdrext/color-space\na=rtpmap:96 VP8/90000\na=rtpmap:97 rtx/90000",
            "codecs": [
                {
                    "name": "opus",
                    "type": "audio",
                    "priority": 1000,
                    "payload_type": 111,
                    "rtx_payload_type": null
                },
                {
                    "name": "VP8",
                    "type": "video",
                    "priority": 2000,
                    "payload_type": 96,
                    "rtx_payload_type": 97
                },
                {
                    "name": "VP9",
                    "type": "video",
                    "priority": 3000,
                    "payload_type": 98,
                    "rtx_payload_type": 99
                }
            ],
            "rtc_connection_id": "b410d781-24db-4e7b-808a-8ac6a5707caa"
        }
    }
    "#;
    let _webrtc_expected = Event::SelectProtocol(SelectProtocol::WebRTC {
        data: "a=extmap-allow-mixed\na=ice-ufrag:nEdg\na=ice-pwd:cGgW+G3KQbmO9ptaZfqffUQi\na=ice-options:trickle\na=extmap:1 urn:ietf:params:rtp-hdrext:ssrc-audio-level\na=extmap:2 http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time\na=extmap:3 http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01\na=extmap:4 urn:ietf:params:rtp-hdrext:sdes:mid\na=extmap:5 urn:ietf:params:rtp-hdrext:sdes:rtp-stream-id\na=extmap:6 urn:ietf:params:rtp-hdrext:sdes:repaired-rtp-stream-id\na=rtpmap:111 opus/48000/2\na=extmap:14 urn:ietf:params:rtp-hdrext:toffset\na=extmap:13 urn:3gpp:video-orientation\na=extmap:12 http://www.webrtc.org/experiments/rtp-hdrext/playout-delay\na=extmap:11 http://www.webrtc.org/experiments/rtp-hdrext/video-content-type\na=extmap:7 http://www.webrtc.org/experiments/rtp-hdrext/video-timing\na=extmap:8 http://www.webrtc.org/experiments/rtp-hdrext/color-space\na=rtpmap:96 VP8/90000\na=rtpmap:97 rtx/90000".to_string(),
        sdp: "a=extmap-allow-mixed\na=ice-ufrag:nEdg\na=ice-pwd:cGgW+G3KQbmO9ptaZfqffUQi\na=ice-options:trickle\na=extmap:1 urn:ietf:params:rtp-hdrext:ssrc-audio-level\na=extmap:2 http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time\na=extmap:3 http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01\na=extmap:4 urn:ietf:params:rtp-hdrext:sdes:mid\na=extmap:5 urn:ietf:params:rtp-hdrext:sdes:rtp-stream-id\na=extmap:6 urn:ietf:params:rtp-hdrext:sdes:repaired-rtp-stream-id\na=rtpmap:111 opus/48000/2\na=extmap:14 urn:ietf:params:rtp-hdrext:toffset\na=extmap:13 urn:3gpp:video-orientation\na=extmap:12 http://www.webrtc.org/experiments/rtp-hdrext/playout-delay\na=extmap:11 http://www.webrtc.org/experiments/rtp-hdrext/video-content-type\na=extmap:7 http://www.webrtc.org/experiments/rtp-hdrext/video-timing\na=extmap:8 http://www.webrtc.org/experiments/rtp-hdrext/color-space\na=rtpmap:96 VP8/90000\na=rtpmap:97 rtx/90000".to_string(),
        codecs: vec![
            WebRTCCodec{
                name: CodecName::Opus,
                kind: WebRTCCodecKind::Audio,
                priority: 1000,
                payload_type: 111,
                rtx_payload_type: None,
            },
            WebRTCCodec{
                name: CodecName::VP8,
                kind: WebRTCCodecKind::Video,
                priority: 2000,
                payload_type: 96,
                rtx_payload_type: Some(97),
            },
            WebRTCCodec{
                name: CodecName::VP9,
                kind: WebRTCCodecKind::Video,
                priority: 3000,
                payload_type: 98,
                rtx_payload_type: Some(99),
            }
        ],
        rtc_connection_id: "b410d781-24db-4e7b-808a-8ac6a5707caa".to_string()
    });
    let test_webrtc_res: Event = serde_json::from_str(test_webrtc).unwrap();
    matches!(test_webrtc_res, _webrtc_expected);
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CodecName {
    #[serde(rename = "opus")]
    Opus,
    VP8,
    VP9,
    #[serde(other)]
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct WebRTCCodec {
    name: CodecName,
    #[serde(rename = "type")]
    kind: WebRTCCodecKind,
    priority: u32,
    payload_type: u16,
    rtx_payload_type: Option<u16>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum WebRTCCodecKind {
    #[serde(rename = "audio")]
    Audio,
    #[serde(rename = "video")]
    Video,
}
