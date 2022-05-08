use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tegra_rcm::Payload;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-example");
    group.sample_size(1000);
    group.bench_function("payload", |b| {
        b.iter(|| {
            Payload::new(black_box(include_bytes!(
                "../src/test/hekate_ctcaer_5.7.2.bin"
            )))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
