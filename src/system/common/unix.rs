use std::path::Path;

pub struct Unix;

impl super::Common for Unix {
    fn symlink_dir(original: impl AsRef<Path>, link: impl AsRef<Path>) -> std::io::Result<()> {
        std::os::unix::fs::symlink(original, link)
    }

    fn symlink_file(original: impl AsRef<Path>, link: impl AsRef<Path>) -> std::io::Result<()> {
        std::os::unix::fs::symlink(original, link)
    }
}
