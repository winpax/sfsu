const WIN_MANIFEST: &str = include_str!("./sfsu.exe.manifest");

fn main() -> shadow_rs::SdResult<()> {
    let shadow_res = shadow_rs::new();

    let mut res = winres::WindowsResource::new();
    res.set_manifest(WIN_MANIFEST);

    if let Err(error) = res.compile() {
        eprint!("{error}");
        std::process::exit(1);
    }

    let libgit2_version = git2::Version::get();

    {
        let (major, minor, patch) = libgit2_version.libgit2_version();

        println!(
            "cargo:rustc-env=LIBGIT2_VERSION={}.{}.{}",
            major, minor, patch
        );
    }

    shadow_res
}
