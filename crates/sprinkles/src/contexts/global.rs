use std::{
    fs::File,
    path::{Path, PathBuf},
};

use windows::Win32::Foundation::HWND;

use crate::{config, git};

use super::{ScoopContext, User};

#[derive(Debug, Clone)]
/// Global context adapter
pub struct Global {
    path: PathBuf,
    user_context: User,
}

impl Default for Global {
    fn default() -> Self {
        Self::new()
    }
}

impl Global {
    #[must_use]
    /// Construct a new global context adapter
    pub fn new() -> Self {
        use std::env::var_os;

        let user_context = User::new();

        let path = {
            if let Some(path) = var_os("SCOOP_GLOBAL") {
                path.into()
            } else if let Some(ref path) = user_context.config().global_path {
                path.clone()
            } else {
                use std::{ffi::OsString, os::windows::ffi::OsStringExt};
                use windows::Win32::{
                    Foundation::MAX_PATH,
                    UI::Shell::{SHGetSpecialFolderPathW, CSIDL_COMMON_APPDATA},
                };

                let mut buf = [0u16; MAX_PATH as usize];
                let success = unsafe {
                    #[allow(clippy::cast_possible_wrap)]
                    SHGetSpecialFolderPathW(
                        HWND::default(),
                        &mut buf,
                        CSIDL_COMMON_APPDATA as i32,
                        true,
                    )
                    .as_bool()
                };

                let path = if success {
                    let string = OsString::from_wide(&buf);
                    PathBuf::from(string)
                } else {
                    "C:\\ProgramData".into()
                }
                .join("scoop");

                if !path.exists() {
                    std::fs::create_dir(&path).expect("could not create scoop global path");
                }

                path
            }
        };

        let path = if path.exists() {
            dunce::canonicalize(path).expect("failed to find real path to scoop")
        } else {
            panic!("Scoop path does not exist");
        };

        Self { path, user_context }
    }
}

impl ScoopContext<config::Scoop> for Global {
    fn config(&self) -> &config::Scoop {
        self.user_context.config()
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn git_path() -> Result<PathBuf, which::Error> {
        todo!()
    }

    fn scoop_sub_path(&self, segment: impl AsRef<Path>) -> PathBuf {
        todo!()
    }

    fn apps_path(&self) -> PathBuf {
        todo!()
    }

    fn buckets_path(&self) -> PathBuf {
        todo!()
    }

    fn cache_path(&self) -> PathBuf {
        todo!()
    }

    fn persist_path(&self) -> PathBuf {
        todo!()
    }

    fn shims_path(&self) -> PathBuf {
        todo!()
    }

    fn workspace_path(&self) -> PathBuf {
        todo!()
    }

    fn installed_apps(&self) -> std::io::Result<Vec<PathBuf>> {
        todo!()
    }

    fn logging_dir(&self) -> std::io::Result<PathBuf> {
        todo!()
    }

    async fn new_log(&self) -> Result<File, super::Error> {
        todo!()
    }

    fn new_log_sync(&self) -> Result<File, super::Error> {
        todo!()
    }

    fn app_installed(&self, name: impl AsRef<str>) -> std::io::Result<bool> {
        todo!()
    }

    fn open_repo(&self) -> Option<git::Result<git::Repo>> {
        todo!()
    }

    fn context_app_path(&self) -> PathBuf {
        todo!()
    }

    async fn outdated(&self) -> Result<bool, super::Error> {
        todo!()
    }
}
