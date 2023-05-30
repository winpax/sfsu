#[derive(sfsu_derive::IntoInner)]
#[inner(ret = String)]
enum MaybeIntoInner {
    String(String),
    Path(String),
}
