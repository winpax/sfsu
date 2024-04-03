use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use sfsu::packages::reference::Package;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("find package across buckets", "sfsu"),
        &Package::Name("sfsu".into()),
        |b, package| b.iter(|| black_box(package.first().unwrap())),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
