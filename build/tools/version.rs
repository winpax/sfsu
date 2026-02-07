use shadow_rs::Shadow;

#[derive(Debug, Copy, Clone)]
pub struct SprinklesVersion<'a> {
    version: &'a str,
    git_rev: Option<&'a str>,
    source: &'a str,
}

impl<'a> SprinklesVersion<'a> {
    pub const fn new() -> Self {
        Self {
            version: "local",
            git_rev: None,
            source: "local",
        }
    }

    pub fn long_version(&self, shadow: &Shadow) -> String {
        let map = &shadow.map;

        let sprinkles_rev = if let Some(git_rev) = self.git_rev() {
            format!(" (git rev: {})", git_rev)
        } else if self.source == "local" {
            " (local)".to_string()
        } else {
            " (crates.io published version)".to_string()
        };

        let (major, minor, patch) = git2::Version::get().libgit2_version();

        format!(
            "{pkg_version} \n\
            sprinkles {sprinkles_version}{sprinkles_rev} \n\
            branch:{branch} \n\
            tag:{tag} \n\
            commit_hash:{short_commit} \n\
            build_time:{build_time} \n\
            build_env:{rust_version},{rust_channel} \n\
            libgit2:{major}.{minor}.{patch}",
            sprinkles_version = self.version(),
            branch = &map.get("BRANCH").expect("missing BRANCH").v,
            build_time = &map.get("BUILD_TIME").expect("missing BUILD_TIME").v,
            pkg_version = &map.get("PKG_VERSION").expect("missing PKG_VERSION").v,
            rust_channel = &map.get("RUST_CHANNEL").expect("missing RUST_CHANNEL").v,
            rust_version = &map.get("RUST_VERSION").expect("missing RUST_VERSION").v,
            short_commit = &map.get("SHORT_COMMIT").expect("missing SHORT_COMMIT").v,
            tag = &map.get("TAG").expect("missing TAG").v,
        )
    }

    pub fn version(&self) -> &str {
        self.version
    }

    pub fn git_rev(&self) -> Option<&str> {
        self.git_rev
    }
}
