use std::path::Path;

use cfg_if::cfg_if;

#[allow(dead_code)]
pub trait Common {
    fn symlink_dir(original: impl AsRef<Path>, link: impl AsRef<Path>) -> std::io::Result<()>;

    fn symlink_file(original: impl AsRef<Path>, link: impl AsRef<Path>) -> std::io::Result<()>;
}

cfg_if! {
    if #[cfg(windows)] {
        pub mod win;
        pub use win::Windows as System;
    } else if #[cfg(unix)] {
        pub mod unix;
        pub use unix::Unix as System;
    }
}
