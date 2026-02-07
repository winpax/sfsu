# Sprinkles Library

[![Build & Test](https://github.com/winpax/sprinkles/actions/workflows/build.yml/badge.svg)](https://github.com/winpax/sprinkles/actions/workflows/build.yml)
[![Crates.io Version](https://img.shields.io/crates/v/sprinkles-rs)](https://crates.io/crates/sprinkles-rs)
[![docs.rs](https://img.shields.io/docsrs/sprinkles-rs)](https://docs.rs/sprinkles-rs)
[![dependency status](https://deps.rs/crate/sprinkles-rs/latest/status.svg)](https://deps.rs/crate/sprinkles-rs/)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/sprinkles-rs)](https://crates.io/crates/sprinkles-rs)
[![Crates.io License](https://img.shields.io/crates/l/sprinkles-rs)](https://crates.io/crates/sprinkles-rs)
[![Crates.io MSRV](https://img.shields.io/crates/msrv/sprinkles-rs)](https://crates.io/crates/sprinkles-rs)

View the latest documentation on [Github Pages](https://winpax.github.io/sprinkles/) (the [docs.rs](https://docs.rs/sprinkles-rs) builds are currently failing).

> [!WARNING]
> This library is currently unstable, and is not recommended for use in production.
> There will be breaking changes in the future, and the API may change.

Sprinkles is a library for interacting with [Scoop](https://scoop.sh/), the Windows package manager.

It provides a high-level API for interacting with [Scoop](https://scoop.sh/), such as installing, updating, and removing packages.

## Reporting Issues

Please, please, please check the [FAQs](https://github.com/winpax/FAQs), before you report an issue.

If you have a question, please ask it on [the discussions page](https://github.com/orgs/winpax/discussions).

If you have a bug report, feature request, or other issue, then [open an issue](https://github.com/winpax/sprinkles/issues/new/choose).

## Example Usage

If you want a more in depth example of how to use the library, check out the [sfsu](https://github.com/winpax/sfsu) project.

```rust
use sprinkles::contexts::{User, ScoopContext};

let ctx = User::new().unwrap();

let apps = ctx.installed_apps().unwrap();

println!("You have {} apps installed", apps.len());
```

## Running Benchmarks

The benchmarks rely on the `large-file.bin` file, which should be a large amount (>100MB) of random data.

To generate the file, run the following command (feel free to change the size to your liking):

### Windows

Install [genfile](https://github.com/winpax/genfile) and run the following command:

```powershell
genfile --size 512mb -o benches/large-file.bin --random
```

### Linux

```bash
dd if=/dev/urandom of=benches/large-file.bin bs=1M count=512
```

## Supported Platforms

Windows is the only supported platform at the moment, and this will likely not change, given that Scoop is only available on Windows.

## Minimum Supported Rust Version

The MSRV may change in the future, but will only ever be increased over the course of a major version.

The MSRV is set for the target `x86_64-pc-windows-msvc`, and is checked for all supported platforms in the CI.

**Made with 💗 by Juliette Cordor**
