const WIN_MANIFEST: &str = include_str!("./sfsu.exe.manifest");

fn main() -> shadow_rs::SdResult<()> {
    let shadow_res = shadow_rs::new();

    let mut res = winres::WindowsResource::new();
    res.set_manifest(WIN_MANIFEST);

    if let Err(error) = res.compile() {
        eprint!("{error}");
        std::process::exit(1);
    }

    shadow_res
}
