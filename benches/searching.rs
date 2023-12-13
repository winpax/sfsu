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
                .filter_map(|bucket| bucket.matches(&pattern, black_box(SearchMode::Name)))
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        })
    });

    c.bench_function("parsing output", |b| {
        for bucket in Bucket::list_all().unwrap() {
            b.iter_batched(
                || bucket.clone(),
                |ref bucket| {
                    let bucket_contents = black_box(bucket).list_packages_unchecked().unwrap();

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
        // b.iter_batched(|| {
        //     black_box(Bucket::list_all().unwrap())
        //         .par_iter()
        //         .filter_map(|bucket| bucket.matches(&pattern, black_box(SearchMode::Name)))
        //         .collect::<Result<Vec<_>, _>>()
        //         .unwrap();
        // })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
