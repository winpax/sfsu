use std::{str::FromStr, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use indicatif::MultiProgress;
use sprinkles::{
    cache::{Downloader, Handle},
    packages::reference::Package,
    requests::BlockingClient,
    Scoop,
};

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
        b.iter(|| black_box(BlockingClient::new()));
    });

    c.bench_function("create async clients", |b| {
        b.iter(|| black_box(sprinkles::requests::AsyncClient::new()));
    });

    c.bench_function("open handle", |b| {
        b.iter_batched(
            || {
                Package::from_str("extras/sfsu")
                    .unwrap()
                    .manifest()
                    .unwrap()
            },
            |manifest| {
                Handle::open_manifest(Scoop::cache_path(), &manifest)
                    .unwrap()
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
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

    properties.bench_function("create downloader", |b| {
        b.iter_batched(
            || {
                (
                    Handle::open_manifest(
                        Scoop::cache_path(),
                        &Package::from_str("extras/sfsu")
                            .unwrap()
                            .manifest()
                            .unwrap(),
                    )
                    .unwrap()
                    .unwrap(),
                    BlockingClient::new(),
                )
            },
            |(dl, client)| black_box(Downloader::new(dl, &client, None)),
            BatchSize::SmallInput,
        );
    });

    properties.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
