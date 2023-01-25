use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::RngCore;

fn create_source_block_data(length: usize) -> Vec<u8> {
    let mut output = vec![0u8; length];

    // Random buffer
    let mut rng = rand::thread_rng();
    rng.fill_bytes(output.as_mut());

    output
}

fn raptor_benchmark(c: &mut Criterion) {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::builder().is_test(true).try_init().ok();

    let data = create_source_block_data(100 * 1024 * 1024);

    c.bench_function("encode 1k", |b| {
        b.iter(|| {
            raptor_code::encode_source_block(black_box(&data[..1000]), black_box(64), black_box(10))
        })
    });

    c.bench_function("encode 10k", |b| {
        b.iter(|| {
            raptor_code::encode_source_block(
                black_box(&data[0..10 * 1024]),
                black_box(64),
                black_box(10),
            )
        })
    });

    c.bench_function("encode 100k", |b| {
        b.iter(|| {
            raptor_code::encode_source_block(
                black_box(&data[0..100 * 1024]),
                black_box(64),
                black_box(10),
            )
        })
    });

    c.bench_function("encode 1MB", |b| {
        b.iter(|| {
            raptor_code::encode_source_block(
                black_box(&data[0..1024 * 1024]),
                black_box(64),
                black_box(10),
            )
        })
    });

    c.bench_function("encode 10MB", |b| {
        b.iter(|| {
            raptor_code::encode_source_block(
                black_box(&data[0..10 * 1024 * 1024]),
                black_box(64),
                black_box(10),
            )
        })
    });
}

criterion_group!(benches, raptor_benchmark);
criterion_main!(benches);
