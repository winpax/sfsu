#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PackageInfo<'a> {
    name: &'a str,
    version: &'a str,
    architecture: &'a str,
    description: &'a str,
    license: &'a str,
    msrv: &'a str,
    rustc_version: &'a str,
    rustc_channel: &'a str,
    rustc_date: &'a str,
}

impl<'a> PackageInfo<'a> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            architecture: std::env::consts::ARCH,
            description: env!("CARGO_PKG_DESCRIPTION"),
            license: env!("CARGO_PKG_LICENSE"),
            msrv: env!("CARGO_PKG_RUST_VERSION"),
            rustc_version: env!("RUSTC_VERSION"),
            rustc_channel: env!("RUSTC_CHANNEL"),
            rustc_date: env!("RUSTC_DATE"),
        }
    }
}
