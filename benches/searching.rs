use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use rayon::prelude::*;
use regex::Regex;
use sfsu::{buckets::Bucket, packages::SearchMode};

fn criterion_benchmark(c: &mut Criterion) {
    // let all_buckets = Bucket::list_all().unwrap();

    let pattern = Regex::new("(?i)google").unwrap();

    c.bench_function("list buckets", |b| b.iter(|| Bucket::list_all().unwrap()));

    c.bench_function("match packages", |b| {
        b.iter(|| {
            black_box(Bucket::list_all().unwrap())
                .par_iter()
                .filter_map(
                    |bucket| match bucket.matches(&pattern, black_box(SearchMode::Name)) {
                        Ok(Some(section)) => Some(section),
                        _ => None,
                    },
                )
                .collect::<Vec<_>>();
        })
    });

    c.bench_function("parsing output", |b| {
        for bucket in Bucket::list_all().unwrap() {
            b.iter_batched(
                || bucket.list_packages_unchecked().unwrap(),
                |ref bucket_contents| {
                    bucket_contents
                        .par_iter()
                        .filter_map(|manifest| {
                            manifest.parse_output(
                                bucket.name(),
                                false,
                                &pattern,
                                black_box(SearchMode::Name),
                            )
                        })
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        }
    });

    c.bench_function("listing packages unchecked", |b| {
        for bucket in Bucket::list_all().unwrap() {
            b.iter_batched(
                || bucket.clone(),
                |ref bucket| bucket.list_packages_unchecked(),
                BatchSize::SmallInput,
            )
        }
    });

    c.bench_function("listing packages", |b| {
        for bucket in Bucket::list_all().unwrap() {
            b.iter_batched(
                || bucket.clone(),
                |ref bucket| bucket.list_packages(),
                BatchSize::SmallInput,
            )
        }
    });

    c.bench_function("listing packages from names", |b| {
        for bucket in Bucket::list_all().unwrap() {
            b.iter_batched(
                || bucket.clone(),
                |ref bucket| bucket.list_package_names().unwrap(),
                BatchSize::SmallInput,
            )
        }
    });

    c.bench_function("parsing packages from names", |b| {
        let search_mode = SearchMode::Name;
        for bucket in Bucket::list_all().unwrap() {
            b.iter_batched(
                || bucket.clone(),
                |ref bucket| {
                    bucket
                        .list_package_names()
                        .unwrap()
                        .par_iter()
                        .filter_map(|manifest_name| {
                            // Ignore non-matching manifests
                            if black_box(search_mode).match_names()
                                && !black_box(search_mode).match_binaries()
                                && !black_box(&pattern).is_match(manifest_name)
                            {
                                return None;
                            }

                            bucket
                                .get_manifest(manifest_name)
                                .expect("manifest to exist")
                                .parse_output(
                                    bucket.name(),
                                    false,
                                    black_box(&pattern),
                                    black_box(search_mode),
                                )
                        })
                        .collect::<Vec<_>>()
                },
                BatchSize::SmallInput,
            )
        }
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
