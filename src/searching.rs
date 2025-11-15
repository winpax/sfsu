use regex::Regex;
use sprinkles::{
    Architecture,
    contexts::ScoopContext,
    packages::{Manifest, MergeDefaults, SearchMode},
};

#[derive(Debug, Clone)]
#[must_use = "MatchCriteria has no side effects"]
/// The criteria for a match
pub struct MatchCriteria {
    name: bool,
    bins: Vec<String>,
}

impl MatchCriteria {
    /// Create a new match criteria
    pub const fn new() -> Self {
        Self {
            name: false,
            bins: vec![],
        }
    }

    /// Check if the name matches
    pub fn matches(
        file_name: &str,
        pattern: &Regex,
        list_binaries: impl FnOnce() -> Vec<String>,
        mode: SearchMode,
    ) -> Self {
        let mut output = MatchCriteria::new();

        if mode.match_names() {
            output.match_names(pattern, file_name);
        }

        if mode.match_binaries() {
            output.match_binaries(pattern, list_binaries());
        }

        output
    }

    fn match_names(&mut self, pattern: &Regex, file_name: &str) -> &mut Self {
        if pattern.is_match(file_name) {
            self.name = true;
        }
        self
    }

    fn match_binaries(&mut self, pattern: &Regex, binaries: Vec<String>) -> &mut Self {
        let binary_matches = binaries
            .into_iter()
            .filter(|binary| pattern.is_match(binary))
            .filter_map(|b| {
                if pattern.is_match(&b) {
                    Some(b.clone())
                } else {
                    None
                }
            });

        self.bins.extend(binary_matches);

        self
    }

    pub fn matched_name(&self) -> bool {
        self.name
    }

    pub fn matched_bins(&self) -> &[String] {
        &self.bins
    }
}

impl Default for MatchCriteria {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, getset::Getters, getset::CopyGetters)]
pub struct MatchedManifest {
    #[getset(get = "pub")]
    manifest: Manifest,
    #[getset(get_copy = "pub")]
    installed: bool,
    #[getset(get_copy = "pub")]
    name_matched: bool,
    #[getset(get = "pub")]
    bins: Vec<String>,
    #[getset(get_copy = "pub")]
    exact_match: bool,
}

impl MatchedManifest {
    pub fn new(
        ctx: &impl ScoopContext,
        manifest: Manifest,
        pattern: &Regex,
        mode: SearchMode,
        arch: Architecture,
    ) -> MatchedManifest {
        // TODO: Better display of output
        let bucket = unsafe { manifest.bucket() };

        let match_output = MatchCriteria::matches(
            unsafe { manifest.name() },
            pattern,
            // Function to list binaries from a manifest
            // Passed as a closure to avoid this parsing if bin matching isn't required
            || {
                manifest
                    .architecture
                    .merge_default(manifest.install_config.clone(), arch)
                    .bin
                    .map(|b| b.to_vec())
                    .unwrap_or_default()
            },
            mode,
        );

        let installed = manifest.is_installed(ctx, Some(bucket));
        let exact_match = unsafe { manifest.name() } == pattern.to_string();

        MatchedManifest {
            manifest,
            installed,
            name_matched: match_output.matched_name(),
            bins: match_output.matched_bins().to_vec(),
            exact_match,
        }
    }

    pub fn should_match(&self, installed_only: bool) -> bool {
        if !self.installed && installed_only {
            return false;
        }
        if !self.name_matched && self.bins.is_empty() {
            return false;
        }

        true
    }
}
