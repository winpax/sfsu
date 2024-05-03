use std::{str::FromStr, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use sprinkles::{
    cache::{Downloader, Handle},
    packages::reference::Package,
    requests::{AsyncClient, Client},
    Architecture, Scoop,
};

fn criterion_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    c.bench_function("parse package", |b| {
        b.iter(|| black_box(Package::from_str("extras/sfsu").unwrap()));
    });

    c.bench_function("get package manifest", |b| {
        b.to_async(&runtime).iter_batched(
            || Package::from_str("extras/sfsu").unwrap(),
            |package| async move { black_box(package.manifest().await.unwrap()) },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("create clients", |b| {
        b.iter(|| black_box(Client::blocking()));
    });

    c.bench_function("create async clients", |b| {
        b.iter(|| black_box(Client::asynchronous()));
    });

    c.bench_function("open handle", |b| {
        b.to_async(&runtime).iter_batched(
            || async {
                Package::from_str("extras/sfsu")
                    .unwrap()
                    .manifest()
                    .await
                    .unwrap()
            },
            |manifest| async {
                Handle::open_manifest(Scoop::cache_path(), &manifest.await, Architecture::ARCH)
                    .unwrap()
            },
            BatchSize::SmallInput,
        );
    });

    let mut slow = c.benchmark_group("updating manifest properties");
    slow.sample_size(10);
    slow.measurement_time(Duration::from_secs(10));

    slow.bench_function("set version", |b| {
        b.to_async(&runtime).iter_batched(
            || Package::from_str("extras/sfsu").unwrap(),
            |mut package| async move {
                black_box(&mut package).set_version("1.10.2".to_string());
                black_box(package.manifest().await.unwrap());
            },
            BatchSize::SmallInput,
        );
    });

    slow.bench_function("create downloader", |b| {
        b.to_async(&runtime).iter_batched(
            || async {
                Handle::open_manifest(
                    Scoop::cache_path(),
                    &Package::from_str("extras/sfsu")
                        .unwrap()
                        .manifest()
                        .await
                        .unwrap(),
                    Architecture::ARCH,
                )
                .unwrap()
            },
            |dl| async {
                let dl = dl.await;
                black_box(Downloader::new::<AsyncClient>(dl, None).await.unwrap())
            },
            BatchSize::SmallInput,
        );
    });

    slow.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
