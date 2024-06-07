//! Nicer ways to display times

use std::{
    fmt::{Debug, Display},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Local, LocalResult, TimeZone};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::Serialize;

use super::serialize::SerializeDisplay;

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

#[derive(Debug, Clone, AsRef, AsMut, Deref, DerefMut)]
/// A nicer way to display times
pub struct NicerTime<Tz: TimeZone>(DateTime<Tz>);

impl From<SystemTime> for NicerTime<Local> {
    fn from(time: SystemTime) -> Self {
        Self(system_time_to_date_time(time).unwrap())
    }
}

impl<Tz: TimeZone> From<DateTime<Tz>> for NicerTime<Tz> {
    fn from(time: DateTime<Tz>) -> Self {
        Self(time)
    }
}

impl<Tz: TimeZone> Display for NicerTime<Tz>
where
    Tz::Offset: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S"), f)
    }
}

impl<Tz: TimeZone> Serialize for NicerTime<Tz>
where
    Tz::Offset: Display,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SerializeDisplay::from(self).serialize(serializer)
    }
}
