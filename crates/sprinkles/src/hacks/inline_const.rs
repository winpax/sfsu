#[macro_export]
macro_rules! inline_const {
    ($type:tt $expr:expr) => {{
        const OUTPUT: $type = { $expr };
        OUTPUT
    }};
}

pub use inline_const;
