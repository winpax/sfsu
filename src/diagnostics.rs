use std::os::raw::c_void;

use windows::Win32::System::SystemInformation::{OSVERSIONINFOEXW, OSVERSIONINFOW};

use crate::{
    buckets::{Bucket, BucketError},
    Scoop,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Internal Windows API Error: {0}")]
    Windows(#[from] windows::core::Error),

    #[error("Interacting with buckets: {0}")]
    Bucket(#[from] BucketError),
}

pub enum LongPathsStatus {
    /// Long paths are enabled
    Enabled,
    /// This version of windows does not support long paths
    OldWindows,
    /// Long paths are disabled
    Disabled,
}

pub struct Diagnostics {
    pub long_paths: LongPathsStatus,
    pub main_bucket: bool,
    pub windows_developer: bool,
    pub windows_defender: bool,
}

impl Diagnostics {
    /// Collect all diagnostics
    ///
    /// # Errors
    /// - Unable to check long paths
    /// - Unable to check main bucket
    /// - Unable to check windows developer status
    /// - Unable to check windows defender status
    pub fn collect() -> Result<Self, Error> {
        let main_bucket = Self::check_main_bucket()?;
        debug!("Checked main bucket");
        let long_paths = Self::check_long_paths()?;
        debug!("Checked long paths");
        let windows_developer = Self::get_windows_developer_status()?;
        debug!("Checked developer mode");
        let windows_defender = false /* Self::check_windows_defender()? */;
        debug!("Checked defender");

        Ok(Self {
            long_paths,
            main_bucket,
            windows_developer,
            windows_defender,
        })
    }

    #[allow(unreachable_code)]
    /// Check if Windows Defender is ignoring the Scoop directory
    ///
    /// # Errors
    /// - Unable to read the registry
    /// - Unable to open the registry key
    /// - Unable to check if the key exists
    pub fn check_windows_defender() -> windows::core::Result<bool> {
        use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

        unimplemented!("requires elevation");

        let scoop_dir = Scoop::path();
        let key = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r"SOFTWARE\Microsoft\Windows Defender\Exclusions\Paths")?;

        Ok(key.open_subkey(scoop_dir).is_ok())
    }

    /// Check if the main bucket exists
    ///
    /// # Errors
    /// - Unable to list buckets
    pub fn check_main_bucket() -> Result<bool, BucketError> {
        let buckets = Bucket::list_all()?;

        Ok(buckets.into_iter().any(|bucket| bucket.name() == "main"))
    }

    /// Check if long paths are enabled
    ///
    /// # Errors
    /// - Unable to read the registry
    /// - Unable to read the OS version
    pub fn check_long_paths() -> windows::core::Result<LongPathsStatus> {
        use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

        // let os_version_info = unsafe {
        //     IsWindows10OrGreater;

        //     os_version_info
        // };

        let is_windows_10 = unsafe { crate::win::version::is_windows_10_or_later() };
        debug!(
            "Windows version: {}",
            if is_windows_10 {
                "10 or later"
            } else {
                "earlier than 10"
            }
        );

        if !is_windows_10 {
            return Ok(LongPathsStatus::OldWindows);
        }

        let hlkm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = hlkm.open_subkey(r"SYSTEM\CurrentControlSet\Control\FileSystem")?;

        if key.get_value::<u32, _>("LongPathsEnabled")? == 0 {
            Ok(LongPathsStatus::Disabled)
        } else {
            Ok(LongPathsStatus::Enabled)
        }
    }

    /// Check if the user has developer mode enabled
    ///
    /// # Errors
    /// - Unable to read the registry
    /// - Unable to read the value
    pub fn get_windows_developer_status() -> windows::core::Result<bool> {
        use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

        let hlkm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = hlkm.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock")?;

        Ok(key.get_value::<u32, _>("AllowDevelopmentWithoutDevLicense")? == 1)
    }
}
