//! Installed app metadata file resolution
//!
//! Since [ScoopInstaller/Scoop#6732](https://github.com/ScoopInstaller/Scoop/pull/6732),
//! scoop prefix the per-app metadata files with `scoop-` (`scoop-manifest.json` and `scoop-install.json`)
//! to avoid name collisions if the app itself ships a file named `manifest.json` or `install.json`.
//! The old names are still supported for backward compatibility in scoop 0.6.0, but the new names are preferred.
//! We should follow that and handle both layouts, preferring the new names when both exist.
//!

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// The app manifest file name
const MANIFEST: &str = "scoop-manifest.json";

/// The app manifest file name used before Scoop 0.6.0
const LEGACY_MANIFEST: &str = "manifest.json";

/// The install manifest file name
const INSTALL: &str = "scoop-install.json";

/// The install manifest file name used before Scoop 0.6.0
const LEGACY_INSTALL: &str = "install.json";

/// Extension-stripped names of every app metadata file, in either layout
const METADATA_STEMS: [&str; 4] = ["scoop-manifest", "manifest", "scoop-install", "install"];

/// Join `dir` with `name`, falling back to `legacy` only when `name` is known
/// to be absent. An I/O error keeps the prefixed path, so the caller reports it.
fn resolve(dir: &Path, name: &str, legacy: &str) -> PathBuf {
    if matches!(dir.join(name).try_exists(), Ok(false)) {
        dir.join(legacy)
    } else {
        dir.join(name)
    }
}

#[must_use]
/// Resolve the app manifest path within an installed app's version directory
///
/// Falls back to the legacy file name when the prefixed one is absent, so apps
/// installed before Scoop 0.6.0 keep resolving to `manifest.json`. Checks the
/// filesystem for the prefixed file.
pub fn resolve_manifest_path(version_dir: impl AsRef<Path>) -> PathBuf {
    resolve(version_dir.as_ref(), MANIFEST, LEGACY_MANIFEST)
}

#[must_use]
/// Resolve the install manifest path within an installed app's version directory
///
/// Falls back to the legacy file name when the prefixed one is absent. Checks
/// the filesystem for the prefixed file.
pub fn resolve_install_path(version_dir: impl AsRef<Path>) -> PathBuf {
    resolve(version_dir.as_ref(), INSTALL, LEGACY_INSTALL)
}

#[must_use]
/// Check whether a file name is an app manifest, in either layout
pub fn is_manifest(file_name: &OsStr) -> bool {
    file_name == OsStr::new(MANIFEST) || file_name == OsStr::new(LEGACY_MANIFEST)
}

/// Check whether an extension-stripped file name is app or install metadata,
/// in either layout
fn is_metadata_stem(stem: &str) -> bool {
    METADATA_STEMS.contains(&stem)
}

#[must_use]
/// Resolve the app name that a manifest path refers to
///
/// Metadata files are named after their app directory, two levels up
/// (`<app>/<version>/scoop-manifest.json`), rather than after the file itself.
/// Any other manifest, such as one in a bucket, is named after its file stem.
pub fn name_from_path(path: &Path) -> Option<String> {
    let stripped = path.with_extension("");
    let stem = stripped.file_name()?.to_string_lossy();

    if !is_metadata_stem(&stem) {
        return Some(stem.into_owned());
    }

    // Metadata file: the app name is the directory two levels up
    path.ancestors()
        .nth(2)?
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time after unix epoch")
                .as_nanos();

            let path = std::env::temp_dir().join(format!("sfsu-metadata-{label}-{nanos}"));
            std::fs::create_dir_all(&path).expect("create temp dir");

            Self(path)
        }

        fn touch(&self, names: &[&str]) {
            for name in names {
                std::fs::write(self.0.join(name), "{}").expect("write metadata file");
            }
        }

        /// Assert the resolved manifest and install file names
        fn assert_resolves(&self, manifest: &str, install: &str) {
            let file_name = |path: PathBuf| path.file_name().expect("file name").to_owned();
            assert_eq!(
                file_name(resolve_manifest_path(&self.0)),
                OsStr::new(manifest)
            );
            assert_eq!(
                file_name(resolve_install_path(&self.0)),
                OsStr::new(install)
            );
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn prefers_prefixed_names() {
        let dir = TempDir::new("prefixed");
        dir.touch(&[LEGACY_MANIFEST, LEGACY_INSTALL, MANIFEST, INSTALL]);

        dir.assert_resolves(MANIFEST, INSTALL);
    }

    #[test]
    fn falls_back_to_legacy_names() {
        let dir = TempDir::new("legacy");
        dir.touch(&[LEGACY_MANIFEST, LEGACY_INSTALL]);

        dir.assert_resolves(LEGACY_MANIFEST, LEGACY_INSTALL);
    }

    #[test]
    fn falls_back_when_nothing_exists() {
        TempDir::new("missing").assert_resolves(LEGACY_MANIFEST, LEGACY_INSTALL);
    }

    #[test]
    fn recognises_both_layouts() {
        assert!(is_manifest(OsStr::new(MANIFEST)));
        assert!(is_manifest(OsStr::new(LEGACY_MANIFEST)));
        assert!(!is_manifest(OsStr::new(INSTALL)));
        assert!(!is_manifest(OsStr::new("uv.json")));

        assert!(is_metadata_stem("manifest"));
        assert!(is_metadata_stem("scoop-manifest"));
        assert!(is_metadata_stem("install"));
        assert!(is_metadata_stem("scoop-install"));
        assert!(!is_metadata_stem("uv"));
    }
}
