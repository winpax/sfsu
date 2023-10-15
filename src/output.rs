// TODO: Implement centralized output wrappers

use std::fmt::Display;

/// Multiple sections
pub struct Sections<T>(Vec<Section<T>>);

impl<T: Display> Display for Sections<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for section in &self.0 {
            write!(f, "{section}")?;
        }

        Ok(())
    }
}

/// Sectioned data (i.e buckets)
pub struct Section<T> {
    title: Option<String>,
    child: ChildOrChildren<T>,
}

pub enum ChildOrChildren<T> {
    Child(T),
    Children(Vec<T>),
    None,
}

pub struct Text(String);

impl Text {
    pub fn as_section(&self) -> Section<&str> {
        Section {
            title: None,
            child: ChildOrChildren::Child(&self.0),
        }
    }
}

impl<T: Display> Display for Section<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref title) = self.title {
            writeln!(f, "{title}:")?;
        }

        write!(f, "{}", self.child)?;

        Ok(())
    }
}

impl<T: Display> Display for ChildOrChildren<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChildOrChildren::Child(child) => writeln!(f, "{child}"),
            ChildOrChildren::Children(children) => {
                for child in children {
                    writeln!(f, "\t{child}")?;
                }
                Ok(())
            }
            ChildOrChildren::None => Ok(()),
        }
    }
}

/// A table of data
pub struct Structured;
