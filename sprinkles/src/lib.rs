#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    clippy::pedantic,
    rust_2018_idioms,
    rustdoc::all,
    rust_2024_compatibility,
    missing_docs
)]
#![allow(clippy::module_name_repetitions)]

// Ensure supported environment
#[cfg(all(not(docsrs), not(windows), not(feature = "unstable_linux")))]
compile_error!(
    "Only Windows is supported at the moment.\nSee https://github.com/winpax/sprinkles/issues/111 for more information."
);

#[cfg(not(windows))]
#[allow(clippy::diverging_sub_expression)]
macro_rules! windows_only {
    () => {
        unimplemented!("Not implemented on non-windows platforms")
    };
}

pub mod handles;
pub mod packages;

#[macro_use]
extern crate log;

use contexts::Error;
