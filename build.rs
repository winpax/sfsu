use version_check::triple;

fn main() {
    if let Some((version, channel, date)) = triple() {
        println!("cargo:rustc-env=RUSTC_VERSION={version}");
        println!("cargo:rustc-env=RUSTC_CHANNEL={channel}");
        println!("cargo:rustc-env=RUSTC_DATE={date}");
    }
}
