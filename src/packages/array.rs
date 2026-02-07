//! Array helpers (currently unused)

use std::{collections::VecDeque, iter::FusedIterator};

use super::models::manifest::SingleOrArray;

impl<T> SingleOrArray<T> {
    /// Get an iterator over the array
    pub fn iter(&self) -> NestedIterator<&T> {
        match self {
            SingleOrArray::Single(s) => NestedIterator::Single(Some(s)),
            SingleOrArray::Array(a) => NestedIterator::Array(a.iter().collect()),
        }
    }

    /// Get the length of the array
    pub fn len(&self) -> usize {
        match self {
            SingleOrArray::Single(_) => 1,
            SingleOrArray::Array(a) => a.len(),
        }
    }

    /// Check if the array is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, T> IntoIterator for &'a SingleOrArray<T> {
    type Item = &'a T;
    type IntoIter = NestedIterator<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for SingleOrArray<T> {
    type Item = T;
    type IntoIter = NestedIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            SingleOrArray::Single(s) => NestedIterator::Single(Some(s)),
            SingleOrArray::Array(a) => NestedIterator::Array(a.into()),
        }
    }
}

#[derive(Debug, Clone)]
/// An iterator over a nested array
pub enum NestedIterator<T> {
    /// A single element
    Single(Option<T>),
    /// An array of elements
    Array(VecDeque<T>),
}

impl<T> Iterator for NestedIterator<T> {
    type Item = T;

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Single(s) => s.as_ref().map_or((0, Some(0)), |_| (1, Some(1))),
            Self::Array(a) => (a.len(), Some(a.len())),
        }
    }

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(s) => std::mem::take(s),
            Self::Array(a) => a.pop_front(),
        }
    }
}

impl<T> DoubleEndedIterator for NestedIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(s) => std::mem::take(s),
            Self::Array(a) => a.pop_back(),
        }
    }
}

impl<T> ExactSizeIterator for NestedIterator<T> {}

impl<T> FusedIterator for NestedIterator<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_iterator() {
        let array = SingleOrArray::from_vec_or_default(vec![1, 2, 3]);

        let mut iter = array.iter();

        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_single_iterator() {
        let array = SingleOrArray::from_vec_or_default(vec![1]);

        let mut iter = array.iter();

        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_fused_iterator() {
        let array = SingleOrArray::from_vec_or_default(vec![1]);

        let mut iter = array.iter();

        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}
