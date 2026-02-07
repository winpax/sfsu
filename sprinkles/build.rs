use std::error::Error;

use serde_json::json;

fn download_buckets() -> reqwest::Result<serde_json::Value> {
    let response = reqwest::blocking::get(
        "https://raw.githubusercontent.com/ScoopInstaller/Scoop/master/buckets.json",
    )?;
    response.json()
}

fn get_known_buckets() -> Result<String, Box<dyn Error>> {
    let body = download_buckets().unwrap_or_else(|_| {
        println!("cargo::warning=Failed to download buckets.json, using default buckets");

        json!({
            "main": "https://github.com/ScoopInstaller/Main",
            "extras": "https://github.com/ScoopInstaller/Extras",
            "versions": "https://github.com/ScoopInstaller/Versions",
            "nirsoft": "https://github.com/ScoopInstaller/Nirsoft",
            "sysinternals": "https://github.com/niheaven/scoop-sysinternals",
            "php": "https://github.com/ScoopInstaller/PHP",
            "nerd-fonts": "https://github.com/matthewjberger/scoop-nerd-fonts",
            "nonportable": "https://github.com/ScoopInstaller/Nonportable",
            "java": "https://github.com/ScoopInstaller/Java",
            "games": "https://github.com/Calinou/scoop-games"
        })
    });

    let buckets = body.as_object().unwrap();

    let mut output = String::new();

    for bucket in buckets {
        let name = bucket.0;
        let url = bucket.1.as_str().unwrap();

        output += &format!(
            "pub const {}: &str = \"{}\";\n",
            heck::AsShoutySnakeCase(name),
            url
        );
    }

    let mut map = phf_codegen::Map::new();

    for (name, _) in buckets {
        let value = heck::AsShoutySnakeCase(name);
        map.entry(name, value.to_string());
    }

    output += &format!(
        "pub static BUCKETS: phf::Map<&'static str, &'static str> = {};",
        map.build()
    );

    Ok(output)
}

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = std::env::var("OUT_DIR")?;

    std::fs::write(out_path.clone() + "/buckets.rs", get_known_buckets()?)?;

    Ok(())
}
