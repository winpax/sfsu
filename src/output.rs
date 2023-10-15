// TODO: Implement centralized output wrappers
// TODO: Derive common traits

use std::fmt::Display;

pub trait SectionData: Display {}

/// Multiple sections
pub struct Sections<T>(Vec<Section<T>>);

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
        for section in &self.0 {
            writeln!(f, "{section}")?;
        }

        Ok(())
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
        // This conversion is maybe unnecessary,
        write!(f, "{}", self.as_section())
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
        const WHITESPACE: &str = "     ";

        match self {
            Children::Single(child) => writeln!(f, "{WHITESPACE}{child}"),
            Children::Multiple(children) => {
                for child in children {
                    // TODO: Maybe make the binaries all show on one line
                    write!(f, "{WHITESPACE}{child}")?;
                }
                Ok(())
            }
            Children::None => Ok(()),
        }
    }
}

/// A table of data
pub struct Structured;
