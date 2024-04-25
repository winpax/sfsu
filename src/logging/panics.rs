use human_panic::{setup_panic, Metadata};

pub fn handle() {
    setup_panic!(
        Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
            .authors("Juliette Cordor <support+sfsu@maybejules.com>")
            .homepage("github.com/jewelexx/sfsu")
            .support("Open an issue on GitHub: https://github.com/jewlexx/sfsu/issues/new, and upload the aforementioned report file.")
    );
}
