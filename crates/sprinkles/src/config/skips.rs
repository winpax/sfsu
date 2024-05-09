// The methods require pass by ref, so we disable this lint in this module
#![allow(clippy::trivially_copy_pass_by_ref)]

pub trait Skip {
    fn skip(&self) -> bool;
}

impl<T: Default + Eq> Skip for T {
    fn skip(&self) -> bool {
        self == &T::default()
    }
}

pub fn skip(value: &impl Skip) -> bool {
    value.skip()
}

#[inline]
pub fn skip_bool(value: &bool) -> bool {
    *value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip() {
        let value = true;

        assert!(!skip(&value));
    }

    #[test]
    fn test_skip_bool() {
        let value = true;

        assert!(!skip_bool(&value));
    }

    #[test]
    fn test_skip_if_default_parity() {
        let value = true;

        assert_eq!(skip_bool(&value), skip(&value));
    }
}
