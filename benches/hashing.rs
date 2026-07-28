#![cfg(feature = "manifest-hashes")]

const SCRIPT: &str = r#"$basename = $url.split('/')[-1]
$basenameNoExt = $basename.split('.')[0]
$version = $basenameNoExt.split('_')[-1]

$url = "https://github.com/ScoopInstaller/Main/releases/download/v$version/scoop-windows-x86_64-$version.zip"
$hash = "e2a1c7dd49d547fdfe05fc45f0c9e276cb992bd94af151f0cf7d3e2ecfdc4233"

$basename = $url.split('/')[-1]
$basenameNoExt = $basename.split('.')[0]
$version = $basenameNoExt.split('_')[-1]

$url = "https://github.com/ScoopInstaller/Main/releases/download/v$version/scoop-windows-x86_64-$version.zip"
$hash = "e2a1c7dd49d547fdfe05fc45f0c9e276cb992bd94af151f0cf7d3e2ecfdc4233"

$basename = $url.split('/')[-1]
$ext = $basename.split('.')[-1]

$url = "https://github.com/ScoopInstaller/Main/releases/download/v$version/scoop-windows-x86_64-$version.zip"
$hash = "e2a1c7dd49d547fdfe05fc45f0c9e276cb992bd94af151f0cf7d3e2ecfdc4233"
"#;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use sha2::Digest;

#[inline(always)]
fn digest_hash<D: Digest>(input: impl AsRef<[u8]>) -> digest::array::Array<u8, D::OutputSize> {
    let mut hasher = D::new();

    hasher.update(input);

    hasher.finalize()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sha256 hash script", |b| {
        b.iter(|| digest_hash::<sha2::Sha256>(black_box(SCRIPT)))
    });

    c.bench_function("sha512 hash script", |b| {
        b.iter(|| digest_hash::<sha2::Sha512>(black_box(SCRIPT)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
