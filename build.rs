use toml_edit::{value, DocumentMut};

const LOCKFILE: &str = include_str!("./Cargo.lock");
const WIN_MANIFEST: &str = include_str!("./sfsu.exe.manifest");

fn main() -> shadow_rs::SdResult<()> {
    let shadow_result = shadow_rs::new();

    let mut doc = LOCKFILE.parse::<DocumentMut>().unwrap();
    let packages = doc.get("package").unwrap();
    let packages = packages.as_array_of_tables().unwrap();

    let mut items = vec![];
    for p in packages {
        let name = p.get("name").unwrap().as_str().unwrap();
        let version = p.get("version").unwrap().as_str().unwrap();
        // let source = p.get("source").unwrap().as_str().unwrap();

        let item = format!("(\"{name}\",\"{version}\")");
        items.push(item);
    }

    let items = items.join(",");
    let items = format!("[{}]", items);
    let packages_output = format!("pub const PACKAGES: &str = {items};");

    let out_path = std::env::var("OUT_DIR")?;
    std::fs::write(out_path + "/packages.rs", packages_output);

    let mut res = winres::WindowsResource::new();
    res.set_manifest(WIN_MANIFEST);

    if let Err(error) = res.compile() {
        eprint!("{error}");
        std::process::exit(1);
    }

    let libgit2_version = git2::Version::get();

    let (major, minor, patch) = libgit2_version.libgit2_version();

    println!(
        "cargo:rustc-env=LIBGIT2_VERSION={}.{}.{}",
        major, minor, patch
    );
    shadow_result
}
