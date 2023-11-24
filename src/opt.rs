pub trait ResultIntoOption<T> {
    fn into_option(self) -> Option<T>;
}
