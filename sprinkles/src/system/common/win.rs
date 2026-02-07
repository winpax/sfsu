use std::path::Path;

pub struct Windows;

impl super::Common for Windows {
    fn symlink_dir(original: impl AsRef<Path>, link: impl AsRef<Path>) -> std::io::Result<()> {
        std::os::windows::fs::symlink_dir(original, link)
    }

    fn symlink_file(original: impl AsRef<Path>, link: impl AsRef<Path>) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(original, link)
    }
}
