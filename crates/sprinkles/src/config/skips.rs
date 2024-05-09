// The methods require pass by ref, so we disable this lint in this module
#![allow(clippy::trivially_copy_pass_by_ref)]

pub trait Skip {
    fn skip(&self) -> bool;
}

impl<T: Default + Eq> Skip for T {
    #[inline]
    fn skip(&self) -> bool {
        self == &T::default()
    }
}

#[inline]
pub fn skip(value: &impl Skip) -> bool {
    value.skip()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip() {
        let value = true;

        assert!(!skip(&value));
    }
}
