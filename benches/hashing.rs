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
fn sha2_hash<D: Digest>(input: impl AsRef<[u8]>) -> String
// WTF is this
where
    <D as sha2::digest::OutputSizeUser>::OutputSize: std::ops::Add,
    <<D as sha2::digest::OutputSizeUser>::OutputSize as std::ops::Add>::Output:
        sha2::digest::generic_array::ArrayLength<u8>,
{
    let mut hasher = D::new();

    hasher.update(input);

    format!("{:x}", hasher.finalize())
}

#[inline(always)]
fn sha256_hash(input: impl AsRef<[u8]>) -> String {
    sha2_hash::<sha2::Sha256>(input)
}

#[inline(always)]
fn sha512_hash(input: impl AsRef<[u8]>) -> String {
    sha2_hash::<sha2::Sha512>(input)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sha256 hash script", |b| {
        b.iter(|| sha256_hash(black_box(SCRIPT)))
    });

    c.bench_function("sha512 hash script", |b| {
        b.iter(|| sha512_hash(black_box(SCRIPT)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
