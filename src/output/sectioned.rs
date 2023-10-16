// TODO: Implement centralized output wrappers
// TODO: Derive common traits

use std::fmt::Display;

pub trait SectionData: Display {}

/// Multiple sections
pub struct Sections<T>(Vec<Section<T>>);

impl<A> FromIterator<Section<A>> for Sections<A> {
    fn from_iter<T: IntoIterator<Item = Section<A>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T> Sections<T> {
    #[must_use]
    pub fn from_vec(vec: Vec<Section<T>>) -> Self {
        Self(vec)
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
pub struct Section<T> {
    pub title: Option<String>,
    pub child: Children<T>,
}

impl<T> Section<T> {
    #[must_use]
    pub fn new(child: Children<T>) -> Self {
        Self { title: None, child }
    }

    #[must_use]
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);

        self
    }
}

pub enum Children<T> {
    Single(T),
    Multiple(Vec<T>),
    None,
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

    #[must_use]
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
            writeln!(f, "{title}")?;
        }

        write!(f, "{}", self.child)?;

        Ok(())
    }
}

impl<T: Display> Display for Children<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const WHITESPACE: &str = "  ";

        match self {
            // TODO: Remove newline here and put it in each usage
            Children::Single(child) => writeln!(f, "{WHITESPACE}{child}"),
            Children::Multiple(children) => {
                for child in children {
                    write!(f, "{WHITESPACE}{child}")?;
                }
                Ok(())
            }
            Children::None => Ok(()),
        }
    }
}
