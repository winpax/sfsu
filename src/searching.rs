use regex::Regex;
use sprinkles::{
    Architecture,
    contexts::ScoopContext,
    packages::{Manifest, MergeDefaults, SearchMode},
    version::Version,
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
}

impl Default for MatchCriteria {
    fn default() -> Self {
        Self::new()
    }
}
