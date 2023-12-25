use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::buckets::Bucket;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Summary {
    name: String,
    source: Url,
    updated: DateTime<Local>,
    manifests: usize,
}

impl Summary {
    pub fn from_bucket(bucket: Bucket) {}
}
