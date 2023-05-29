use std::path::PathBuf;

#[derive(sfsu_derive::IntoInner)]
#[inner_type(String)]
enum MaybeIntoInner {
    String(String),
    Path(String),
}
