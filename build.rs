use std::{error::Error, io::Write};

use contribs::contributors::Contributors;
use dotenv::dotenv;
use toml_edit::DocumentMut;

const LOCKFILE: &str = include_str!("./Cargo.lock");
const WIN_MANIFEST: &str = include_str!("./sfsu.exe.manifest");
const COLOURS: &[&str] = &[
    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
];

const COLOURS_TXT: &str = r#"
#[macro_export]
#[doc = concat!("Create a colored string with the `", stringify!(#ident), "` color.")]
macro_rules! #ident {
    ($($arg:tt)*) => {{
        console::style(format_args!($($arg)*)).#ident()
    }};
}

#[macro_export]
#[doc = concat!("Create a colored string with the `", stringify!(#ident_bright), "` color.")]
macro_rules! #ident_bright {
    ($($arg:tt)*) => {{
        $crate::output::colours::#ident!($($arg)*).bright()
    }};
}

#[macro_export]
#[doc = concat!("Print a colored string with the `", stringify!(#ident), "` color.")]
macro_rules! #println {
    ($($arg:tt)*) => {{
        println!("{}", $crate::output::colours::#ident!($($arg)*))
    }};
}

#[macro_export]
#[doc = concat!("Print a colored string with the `", stringify!(#ident_bright), "` color.")]
macro_rules! #println_bright {
    ($($arg:tt)*) => {{
        println!("{}", $crate::output::colours::#ident_bright!($($arg)*))
    }};
}

#[macro_export]
#[doc = concat!("Print a colored string to stderr with the `", stringify!(#ident), "` color.")]
macro_rules! #eprintln {
    ($($arg:tt)*) => {{
        eprintln!("{}", $crate::output::colours::#ident!($($arg)*))
    }};
}

#[macro_export]
#[doc = concat!("Print a colored string to stderr with the `", stringify!(#ident_bright), "` color.")]
macro_rules! #eprintln_bright {
    ($($arg:tt)*) => {{
        eprintln!("{}", $crate::output::colours::#ident_bright!($($arg)*))
    }};
}

pub use #ident;
pub use #ident_bright;
pub use #println;
pub use #println_bright;
pub use #eprintln;
pub use #eprintln_bright;
"#;

fn get_contributors() -> Result<String, Box<dyn Error>> {
    // Try and load dotenv file
    _ = dotenv();

    if let Ok(api_key) = std::env::var("CONTRIBUTORS_TOKEN") {
        let contributors = Contributors::new(api_key, "jewlexx".into(), "sfsu".into())?;
        let contributors =
            tokio::runtime::Runtime::new()?.block_on(async move { contributors.await })?;

        let contributors = contributors
            .into_iter()
            .filter_map(|contrib| {
                let name = contrib.name.as_ref().or(contrib.login.as_ref())?.clone();

                if name == "renovate[bot]" || name == "jewlexx" {
                    return None;
                }

                let login = contrib.login.as_ref()?.clone();
                let url = format!("https://github.com/{login}");

                Some(format!("(\"{name}\",\"{url}\")"))
            })
            .collect::<Vec<_>>();
        let length = contributors.len();

        let contributors = format!("[{}]", contributors.join(", "));
        let contributors_output =
            format!("pub const CONTRIBUTORS: [(&str, &str); {length}] = {contributors};");

        Ok(contributors_output)
    } else {
        if std::env::var("IS_RELEASE").is_ok() {
            panic!("No CONTRIBUTORS_TOKEN found, contributors will not be updated.");
        }

        Ok("pub const CONTRIBUTORS: [(&str, &str); 0] = [];".to_string())
    }
}

fn get_packages() -> String {
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
    format!("pub const PACKAGES: [(&str, &str); {length}] = {items};")
}

fn write_colours(output_file: &mut impl Write) -> Result<(), Box<dyn Error>> {
    writeln!(output_file, "// This file is autogenerated")?;

    for colour in COLOURS {
        let output = COLOURS_TXT
            .replace("#ident_bright", &format!("bright_{colour}"))
            .replace("#ident", colour)
            .replace("#eprintln_bright", &format!("eprintln_bright_{colour}"))
            .replace("#eprintln", &format!("eprintln_{colour}"))
            .replace("#println_bright", &format!("println_bright_{colour}"))
            .replace("#println", &format!("println_{colour}"));

        output_file.write_all(output.as_bytes())?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = std::env::var("OUT_DIR")?;

    shadow_rs::new()?;

    panic!("{:#?}", std::env::var("IS_RELEASE"));

    std::fs::write(out_path.clone() + "/contributors.rs", {
        let contributors = get_contributors();

        match contributors {
            Ok(contributors) => contributors,
            Err(e) if std::env::var("IS_RELEASE").is_ok_and(|v| v == "true") => {
                panic!("Getting contributors failed with error: {e}");
            }
            _ => "pub const CONTRIBUTORS: [(&str, &str); 0] = [];".to_string(),
        }
    })?;
    std::fs::write(out_path.clone() + "/packages.rs", get_packages())?;

    let mut colours_file = std::fs::File::create(out_path.clone() + "/colours.rs")?;
    write_colours(&mut colours_file)?;

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
