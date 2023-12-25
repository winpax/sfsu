//! Summary info structs

use serde::{Deserialize, Serialize};

pub mod bucket;
pub mod package;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summaries {
    pub buckets: Vec<bucket::Summary>,
    pub packages: Vec<package::Summary>,
    pub config: Option<crate::config::Scoop>,
}
