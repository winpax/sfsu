pub trait ResultIntoOption<T> {
    fn into_option(self) -> Option<T>;
}

impl<T, E> ResultIntoOption<T> for Result<T, E> {
    fn into_option(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}
