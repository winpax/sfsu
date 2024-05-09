use std::path::PathBuf;

// pub fn default_scoop_repo() -> String {
//     "https://github.com/ScoopInstaller/Scoop".into()
// }

pub fn default_scoop_root_path() -> PathBuf {
    let mut path = PathBuf::from(
        directories::BaseDirs::new()
            .expect("user directories")
            .home_dir(),
    );
    path.push("scoop");
    path
}

/// Gets the default scoop path
///
/// Note, we do not create the directory here,
/// as it causes too many issues when not running as admin
///
/// This should be handled manually by implementations, when running as admin
pub fn default_scoop_global_path() -> PathBuf {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};

    use windows::Win32::{
        Foundation::{HWND, MAX_PATH},
        UI::Shell::{SHGetSpecialFolderPathW, CSIDL_COMMON_APPDATA},
    };

    let mut buf = [0u16; MAX_PATH as usize];
    let success = unsafe {
        #[allow(clippy::cast_possible_wrap)]
        SHGetSpecialFolderPathW(HWND::default(), &mut buf, CSIDL_COMMON_APPDATA as i32, true)
            .as_bool()
    };

    let path = if success {
        let string = OsString::from_wide(&buf);
        let utf8_string = string.to_string_lossy();
        let trimmed = utf8_string.trim_end_matches('\0');

        PathBuf::from(trimmed)
    } else {
        "C:\\ProgramData".into()
    }
    .join("scoop");

    path
}
