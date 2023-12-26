// TODO: Implement centralized output wrappers
// TODO: Derive common traits

use std::fmt::Display;

use rayon::prelude::*;

pub const WHITESPACE: &str = "  ";

pub trait SectionData: Display {}

/// Multiple sections
#[must_use = "does nothing unless printed"]
pub struct Sections<T>(Vec<Section<T>>);

impl<T: Send> FromParallelIterator<Section<T>> for Sections<T> {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = Section<T>>,
    {
        Self(par_iter.into_par_iter().collect())
    }
}

impl<T> FromIterator<Section<T>> for Sections<T> {
    fn from_iter<I: IntoIterator<Item = Section<T>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T> Sections<T> {
    pub fn from_vec(vec: Vec<Section<T>>) -> Self {
        Self(vec)
    }

    pub fn sort(&mut self) {
        self.0.sort_by(|a, b| a.title.cmp(&b.title));
    }

    pub fn par_sort(&mut self)
    where
        T: Send,
    {
        self.0.par_sort_by(|a, b| a.title.cmp(&b.title));
    }
}

impl<T: Display> SectionData for Sections<T> {}
impl<T: Display> SectionData for Section<T> {}
impl<T: Display> SectionData for Children<T> {}
impl<T: Display> SectionData for Text<T> {}

impl<T: Display> Display for Sections<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some((last, sections)) = self.0.split_last() else {
            // Ignore empty vectors
            return writeln!(f, "No results found");
        };
        for section in sections {
            writeln!(f, "{section}")?;
        }

        write!(f, "{last}")
    }
}

/// Sectioned data (i.e buckets)
#[must_use = "does nothing unless printed"]
pub struct Section<T> {
    pub title: Option<String>,
    pub child: Children<T>,
}

impl<T> Section<T> {
    pub fn new(child: Children<T>) -> Self {
        Self { title: None, child }
    }

    pub fn with_title(mut self, title: impl Display) -> Self {
        self.title = Some(title.to_string());

        self
    }
}

#[must_use = "does nothing unless printed"]
pub enum Children<T> {
    Single(T),
    Multiple(Vec<T>),
    None,
}

impl<T> From<Vec<T>> for Children<T> {
    fn from(value: Vec<T>) -> Self {
        match value {
            v if v.is_empty() => Children::None,
            mut v if v.len() == 1 => Children::Single(v.remove(0)),
            v => Children::Multiple(v),
        }
    }
}

impl<T: Send> FromParallelIterator<T> for Children<T> {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        let children: Vec<_> = par_iter.into_par_iter().collect();

        match children {
            v if v.is_empty() => Children::None,
            mut v if v.len() == 1 => Children::Single(v.remove(0)),
            v => Children::Multiple(v),
        }
    }
}

impl<T> FromIterator<T> for Children<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let children: Vec<_> = iter.into_iter().collect();

        match children {
            v if v.is_empty() => Children::None,
            mut v if v.len() == 1 => Children::Single(v.remove(0)),
            v => Children::Multiple(v),
        }
    }
}

pub struct Text<T>(T);

impl<T> From<T> for Text<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Display> Text<T> {
    #[must_use]
    pub fn new(text: T) -> Self {
        Self(text)
    }

    pub fn as_section(&self) -> Section<&T> {
        Section {
            title: None,
            child: Children::Single(&self.0),
        }
    }
}

impl<T: Display> Display for Text<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Display> Display for Section<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref title) = self.title {
            match self.child {
                Children::None => write!(f, "{title}")?,
                _ => writeln!(f, "{title}")?,
            }
        }

        write!(f, "{}", self.child)?;

        Ok(())
    }
}

impl<T: Display> Display for Children<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // TODO: Indent children based on how nested they are
            Children::Single(child) => writeln!(f, "{WHITESPACE}{child}"),
            Children::Multiple(children) => {
                for child in children {
                    writeln!(f, "{WHITESPACE}{child}")?;
                }
                Ok(())
            }
            Children::None => Ok(()),
        }
    }
}
