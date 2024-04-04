use crate::buckets::{Bucket, BucketError};

pub fn check_windows_defender() {
    unimplemented!("Check if Windows Defender exclused Scoop install path")
}

/// Check if the main bucket exists
///
/// # Errors
/// - Unable to list buckets
pub fn check_main_bucket() -> Result<bool, BucketError> {
    let buckets = Bucket::list_all()?;

    Ok(buckets.into_iter().any(|bucket| bucket.name() == "main"))
}

pub enum LongPathsResult {
    /// Long paths are enabled
    Enabled,
    /// This version of windows does not support long paths
    OldWindows,
    /// Long paths are disabled
    Disabled,
}

/// Check if long paths are enabled
///
/// # Errors
/// - Unable to read the registry
/// - Unable to read the OS version
pub fn check_long_paths() -> windows::core::Result<LongPathsResult> {
    use std::mem::MaybeUninit;
    use windows::Win32::System::SystemInformation;
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    let os_version_info = unsafe {
        let mut os_version_info = MaybeUninit::uninit();
        SystemInformation::GetVersionExW(os_version_info.as_mut_ptr())?;
        os_version_info.assume_init()
    };

    let major_version = os_version_info.dwMajorVersion;
    debug!("Windows version: {major_version:?}");

    if major_version < 10 {
        return Ok(LongPathsResult::OldWindows);
    }

    let hlkm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hlkm.open_subkey(r"SYSTEM\CurrentControlSet\Control\FileSystem")?;

    if key.get_value::<u32, _>("LongPathsEnabled")? == 0 {
        Ok(LongPathsResult::Disabled)
    } else {
        Ok(LongPathsResult::Enabled)
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_main_bucket_exists() {
        assert!(crate::diagnostics::check_main_bucket().unwrap());
    }
}
