use std::path::Path;

/// Returns the size of the file or directory in bytes
pub fn get_recursive_size(path: impl AsRef<Path>) -> std::io::Result<u64> {
    let meta = path.as_ref().symlink_metadata()?;

    let size = if meta.is_file() {
        meta.len()
    } else {
        let mut size = 0;

        for entry in std::fs::read_dir(path)? {
            size += get_recursive_size(entry?.path())?;
        }

        size
    };

    Ok(size)
}
