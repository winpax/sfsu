use std::io::Error;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rayon::prelude::*;
use regex::Regex;
use sfsu::{
    buckets::Bucket,
    output::sectioned::{Children, Section},
    packages::SearchMode,
};

fn criterion_benchmark(c: &mut Criterion) {
    // let all_buckets = Bucket::list_all().unwrap();

    let pattern = Regex::new("(?i)google").unwrap();

    c.bench_function("list buckets", |b| b.iter(|| Bucket::list_all().unwrap()));

    c.bench_function("match packages", |b| {
        b.iter(|| {
            black_box(Bucket::list_all().unwrap())
                .par_iter()
                .filter_map(|bucket| bucket.matches(&pattern, SearchMode::Name))
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
