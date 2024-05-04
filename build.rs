use std::error::Error;

use contribs::Contributors;
use dotenv::dotenv;
use toml_edit::DocumentMut;

const LOCKFILE: &str = include_str!("./Cargo.lock");
const WIN_MANIFEST: &str = include_str!("./sfsu.exe.manifest");

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = std::env::var("OUT_DIR")?;

    shadow_rs::new()?;

    dotenv()?;

    if let Ok(api_key) = std::env::var("GITHUB_API_KEY") {
        let contributors = Contributors::new(api_key, "jewlexx".into(), "sfsu".into())?;
        let contributors =
            tokio::runtime::Runtime::new()?.block_on(async move { contributors.await })?;

        let contributors = contributors
            .into_iter()
            .map(|contrib| {
                let name = contrib.name.as_ref().unwrap_or(&contrib.login).clone();
                let login = &contrib.login.clone();
                let url = format!("https://github.com/{login}");

                format!("(\"{name}\",\"{url}\")")
            })
            .collect::<Vec<_>>();
        let length = contributors.len();

        let contributors = format!("[{}]", contributors.join(", "));
        let contributors_output =
            format!("pub const CONTRIBUTORS: [(&str, &str); {length}] = {contributors};");

        std::fs::write(out_path.clone() + "/contributors.rs", contributors_output)?;
    }

    let doc = LOCKFILE.parse::<DocumentMut>().unwrap();
    let packages = doc.get("package").unwrap();
    let packages = packages.as_array_of_tables().unwrap();

    let mut items = vec![];
    for p in packages {
        let name = p.get("name").unwrap().as_str().unwrap();
        let version = p.get("version").unwrap().as_str().unwrap();

        let item = format!("(\"{name}\",\"{version}\")");
        items.push(item);
    }

    let length = items.len();
    let items = items.join(",");
    let items = format!("[{}]", items);
    let packages_output = format!("pub const PACKAGES: [(&str, &str); {length}] = {items};");

    std::fs::write(out_path.clone() + "/packages.rs", packages_output)?;

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

    Ok(())
}
