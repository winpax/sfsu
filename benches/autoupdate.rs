use std::{str::FromStr, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use sprinkles::packages::reference::Package;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse package", |b| {
        b.iter(|| black_box(Package::from_str("extras/sfsu").unwrap()));
    });

    c.bench_function("get package manifest", |b| {
        b.iter_batched(
            || Package::from_str("extras/sfsu").unwrap(),
            |package| black_box(package.manifest().unwrap()),
            BatchSize::SmallInput,
        );
    });

    c.bench_function("create clients", |b| {
        b.iter(|| black_box(sprinkles::requests::BlockingClient::new()));
    });

    c.bench_function("create async clients", |b| {
        b.iter(|| black_box(sprinkles::requests::AsyncClient::new()));
    });

    let mut properties = c.benchmark_group("updating manifest properties");
    properties.sample_size(10);
    properties.measurement_time(Duration::from_secs(10));

    properties.bench_function("set version", |b| {
        b.iter_batched(
            || Package::from_str("extras/sfsu").unwrap(),
            |mut package| {
                black_box(&mut package).set_version("1.10.2".to_string());
                black_box(package.manifest().unwrap());
            },
            BatchSize::SmallInput,
        );
    });
    properties.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
