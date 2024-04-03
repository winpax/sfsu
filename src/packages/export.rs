use chrono::{DateTime, FixedOffset, SecondsFormat};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    buckets::{Bucket as SfsuBucket, BucketError},
    config,
    packages::MinInfo,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<config::Scoop>,
    pub apps: Vec<App>,
    pub buckets: Vec<Bucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct App {
    pub name: String,
    pub source: String,
    pub updated: String,
    pub version: String,
    pub info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    pub name: String,
    pub source: String,
    pub updated: String,
    pub manifests: usize,
}

impl Export {
    pub fn load() -> anyhow::Result<Self> {
        let config = config::Scoop::load()?;
        let buckets = SfsuBucket::list_all()?
            .into_iter()
            .map(Bucket::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let apps = MinInfo::list_installed(None)?;

        Ok(Self {
            buckets,
            apps: apps.into_par_iter().map(Into::into).collect(),
            config: Some(config),
        })
    }
}

impl From<MinInfo> for App {
    fn from(info: MinInfo) -> Self {
        Self {
            name: info.name,
            source: info.source,
            updated: info.updated.to_rfc3339_opts(SecondsFormat::Micros, false),
            version: info.version,
            info: info.notes,
        }
    }
}

impl TryFrom<SfsuBucket> for Bucket {
    type Error = BucketError;

    fn try_from(bucket: SfsuBucket) -> Result<Self, Self::Error> {
        let name = bucket.name();
        let manifests = bucket.manifests()?;
        let source = bucket.source()?;

        let updated = {
            let repo = bucket.open_repo()?;
            let latest_commit = repo.latest_commit()?;
            let time = latest_commit.time();
            let secs = time.seconds();
            let offset = time.offset_minutes() * 60;

            let utc_time = DateTime::from_timestamp(secs, 0).ok_or(BucketError::InvalidTime)?;

            let offset = FixedOffset::east_opt(offset).ok_or(BucketError::InvalidTimeZone)?;

            utc_time
                .with_timezone(&offset)
                .to_rfc3339_opts(SecondsFormat::Micros, false)
        };

        Ok(Self {
            name: name.to_string(),
            source,
            updated,
            manifests,
        })
    }
}
