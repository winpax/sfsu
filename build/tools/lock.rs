use std::{fs::File, io::Write};

const LOCKFILE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"));

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Lockfile {
    #[serde(rename = "package")]
    packages: Vec<Package>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]

pub struct Package {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
}

impl Lockfile {
    pub fn new() -> Self {
        let lockfile = toml::from_str(LOCKFILE).unwrap();
        println!("cargo:rerun-if-changed=Cargo.lock");

        lockfile
    }

    pub fn get_packages(&self) -> String {
        let mut items = vec![];
        for p in &self.packages {
            let name = &p.name;
            let version = &p.version;

            let item = format!("(\"{name}\",\"{version}\")");
            items.push(item);
        }

        let length = items.len();
        let items = items.join(",");
        let items = format!("[{}]", items);
        format!("pub const PACKAGES: [(&str, &str); {length}] = {items};")
    }

    pub fn hook(&self, mut file: &File) -> std::io::Result<()> {
        writeln!(file, "{}", self.get_packages())
    }
}
