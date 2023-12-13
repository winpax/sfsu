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

    let search_regex = Regex::new("(?i)google").unwrap();

    c.bench_function("list buckets", |b| b.iter(|| Bucket::list_all().unwrap()));

    c.bench_function("match packages", |b| {
        b.iter(|| {
            black_box(Bucket::list_all().unwrap())
                .par_iter()
                .filter_map(|bucket| {
                    // Ignore loose files in the buckets dir
                    if !bucket.path().is_dir() {
                        return None;
                    }

                    let bucket_contents = bucket.list_packages_unchecked().unwrap();

                    let matches = bucket_contents
                        .par_iter()
                        .filter_map(|manifest| {
                            manifest.parse_output(
                                bucket.name(),
                                false,
                                &search_regex,
                                SearchMode::Name,
                            )
                        })
                        .collect::<Vec<_>>();

                    if matches.is_empty() {
                        None
                    } else {
                        Some(Ok::<_, Error>(
                            Section::new(Children::Multiple(matches))
                                // TODO: Remove quotes and bold bucket name
                                .with_title(format!("'{}' bucket:", bucket.name())),
                        ))
                    }
                })
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
