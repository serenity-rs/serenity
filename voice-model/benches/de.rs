use criterion::{Criterion, black_box, criterion_group, criterion_main};
use serenity_voice_model::Event;

pub fn json_deser(c: &mut Criterion) {
    let json_data = r#"{
        "op": 2,
        "d": {
            "ssrc": 1,
            "ip": "127.0.0.1",
            "port": 1234,
            "modes": ["xsalsa20_poly1305", "xsalsa20_poly1305_suffix", "xsalsa20_poly1305_lite"],
            "heartbeat_interval": 1
        }
    }"#;

    let wonky_json_data = r#"{
        "d": {
            "ssrc": 1,
            "ip": "127.0.0.1",
            "port": 1234,
            "modes": ["xsalsa20_poly1305", "xsalsa20_poly1305_suffix", "xsalsa20_poly1305_lite"],
            "heartbeat_interval": 1
        },
        "op": 2
    }"#;

    c.bench_function("Ready event", |b| {
        b.iter(|| serde_json::from_str::<Event>(black_box(json_data)))
    });

    c.bench_function("Ready event (bad order)", |b| {
        b.iter(|| serde_json::from_str::<Event>(black_box(wonky_json_data)))
    });
}

criterion_group!(benches, json_deser);
criterion_main!(benches);
