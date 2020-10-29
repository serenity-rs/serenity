use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use songbird::{constants::*, input::Input};

pub fn mix_one_frame(c: &mut Criterion) {
    let floats = utils::make_sine(STEREO_FRAME_SIZE, true);
    let mut raw_buf = [0f32; STEREO_FRAME_SIZE];

    c.bench_function("Mix stereo source", |b| {
        b.iter_batched_ref(
            || black_box(Input::float_pcm(true, floats.clone().into())),
            |input| {
                input.mix(black_box(&mut raw_buf), black_box(1.0));
            },
            BatchSize::SmallInput,
        )
    });

    c.bench_function("Mix mono source", |b| {
        b.iter_batched_ref(
            || black_box(Input::float_pcm(false, floats.clone().into())),
            |input| {
                input.mix(black_box(&mut raw_buf), black_box(1.0));
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, mix_one_frame);
criterion_main!(benches);
