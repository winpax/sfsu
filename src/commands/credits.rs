use std::fmt::Display;

use clap::Parser;

mod contributors {
    include!(concat!(env!("OUT_DIR"), "/contributors.rs"));
}

mod packages {
    include!(concat!(env!("OUT_DIR"), "/packages.rs"));
}

struct Url<T: Display> {
    text: T,
    url: String,
}

impl<T: Display> Url<T> {
    fn new(text: T, url: String) -> Self {
        Self { text, url }
    }
}

impl<T: Display> Display for Url<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\",
            self.url, self.text
        )
    }
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short, long, help = "Show packages")]
    packages: bool,
}

impl super::Command for Args {
    async fn runner(self) -> anyhow::Result<()> {
        println!(
            "🚀🚀🚀 sfsu v{}, created by Juliette Cordor 🚀🚀🚀",
            env!("CARGO_PKG_VERSION")
        );

        println!();

        if self.packages {
            println!("📦📦📦 sfsu is built with the following packages 📦📦📦");
            for (name, version) in packages::PACKAGES {
                let url = Url::new(name, format!("https://crates.io/crates/{name}"));
                println!("{url}: {version}");
            }

            println!();
        }

        println!("💖💖💖 Many thanks to everyone who as contributed to sfsu 💖💖💖");
        for (name, url) in contributors::CONTRIBUTORS {
            let url = Url::new(name, url.to_string());

            println!("{url}");
        }

        Ok(())
    }
}
