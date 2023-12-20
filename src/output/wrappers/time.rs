use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Local, LocalResult, TimeZone};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Serialize};

fn system_time_to_date_time(time: SystemTime) -> LocalResult<DateTime<Local>> {
    let (secs, nano_secs) = time
        .duration_since(UNIX_EPOCH)
        .map(|duration| {
            (
                duration
                    .as_secs()
                    .try_into()
                    .expect("time within reasonable range"),
                duration.subsec_nanos(),
            )
        })
        .expect("time not to have gone backwards");

    Local.timestamp_opt(secs, nano_secs)
}

#[derive(Debug, Copy, Clone, AsRef, AsMut, Deref, DerefMut)]
pub struct NicerTime(DateTime<Local>);

impl From<SystemTime> for NicerTime {
    fn from(time: SystemTime) -> Self {
        Self(system_time_to_date_time(time).unwrap())
    }
}

impl Serialize for NicerTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NicerTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DateTime::deserialize(deserializer).map(Self)
    }
}
