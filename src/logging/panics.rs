use human_panic::setup_panic;

pub fn handle() {
    setup_panic!(Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "Juliette Cordor <support+sfsu@maybejules.com>".into(),
        homepage: "github.com/jewelexx/sfsu".into(),
    });
}
