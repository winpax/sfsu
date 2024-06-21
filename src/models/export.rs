//! Package export data

use chrono::{DateTime, Local, SecondsFormat};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use sprinkles::{
    buckets::{Bucket as SfsuBucket, Error as BucketError},
    config,
    contexts::ScoopContext,
    git,
    packages::Error as PackageError,
};

use super::min::Info;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The export data
pub struct Export {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The Scoop configuration
    pub config: Option<config::Scoop>,
    /// The installed apps
    pub apps: Vec<App>,
    /// The installed buckets
    pub buckets: Vec<Bucket>,
}

// TODO: Remove this struct in favour of `MinInfo`

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
/// An installed app
pub struct App {
    /// The name of the app
    pub name: String,
    /// The source of the app, e.g. bucket name
    pub source: String,
    /// The last time the app was updated
    pub updated: String,
    /// The version of the app
    pub version: String,
    /// Additional information about the app
    pub info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
/// An installed bucket
pub struct Bucket {
    /// The name of the bucket
    pub name: String,
    /// The source of the bucket (e.g. git URL)
    pub source: String,
    /// The last time the bucket was updated
    pub updated: String,
    /// The number of manifests in the bucket
    pub manifests: usize,
}

impl Export {
    /// Load the export data
    ///
    /// # Errors
    /// - The Scoop configuration could not be loaded
    /// - The buckets could not be listed
    /// - The installed apps could not be listed
    /// - The buckets could not be converted
    pub fn load(ctx: &impl ScoopContext) -> anyhow::Result<Self> {
        let config = config::Scoop::load()?;
        let buckets = SfsuBucket::list_all(ctx)?
            .into_iter()
            .map(Bucket::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let mut apps = Info::list_installed(ctx, None)?;
        apps.par_sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(Self {
            buckets,
            apps: apps.into_par_iter().map(Into::into).collect(),
            config: Some(config),
        })
    }
}

impl From<Info> for App {
    fn from(info: Info) -> Self {
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
    type Error = PackageError;

    fn try_from(bucket: SfsuBucket) -> Result<Self, Self::Error> {
        let name = bucket.name();
        let manifests = bucket.manifests()?;
        let source = bucket.source()?;

        let updated = {
            let repo = bucket.open_repo()?;
            let latest_commit = repo.latest_commit()?;
            let time = latest_commit.time().map_err(git::Error::from)?;
            let secs = time.seconds;
            // let offset = time.offset_minutes() * 60;

            let utc_time = DateTime::from_timestamp(secs, 0).ok_or(BucketError::InvalidTime)?;

            // let offset = FixedOffset::east_opt(offset).ok_or(BucketError::InvalidTimeZone)?;

            utc_time.with_timezone(&Local).to_rfc3339()
        };

        Ok(Self {
            name: name.to_string(),
            source,
            updated,
            manifests,
        })
    }
}
