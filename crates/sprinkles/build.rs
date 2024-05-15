use std::error::Error;

fn get_known_buckets() -> Result<String, Box<dyn Error>> {
    const URL: &str = "https://raw.githubusercontent.com/ScoopInstaller/Scoop/master/buckets.json";

    let response = reqwest::blocking::get(URL)?;
    let body: serde_json::Value = response.json()?;
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
        map.entry(name, &heck::AsShoutySnakeCase(name).to_string());
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
