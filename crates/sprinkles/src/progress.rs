//! Progress helpers for the CLI

mod gitoxide;

pub use indicatif;

use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use gix::progress::Unit;
use indicatif::ProgressStyle;

#[derive(Debug, Copy, Clone, Default)]
/// Progress bar options
pub enum ProgressOptions {
    /// Show bytes/total bytes as progress
    Bytes,
    #[default]
    /// Show pos/len as progress
    PosLen,
    /// Hide progress
    Hide,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
/// Message position
pub struct Message<'a> {
    message: Option<&'a str>,
    position: MessagePosition,
}

impl<'a> Message<'a> {
    #[must_use]
    /// Construct a new prefix message
    pub fn prefix() -> Self {
        Self {
            message: None,
            position: MessagePosition::Prefix,
        }
    }

    #[must_use]
    /// Construct a new suffix message
    pub fn suffix() -> Self {
        Self {
            message: None,
            position: MessagePosition::Suffix,
        }
    }

    #[must_use]
    /// Add a message to the message
    ///
    /// By default the message will be "{msg}",
    /// which will be interpolated by [`indicatif`] with the provided message
    pub fn with_message(self, message: &'a str) -> Self {
        Self {
            message: Some(message),
            position: self.position,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
/// Message position
pub enum MessagePosition {
    #[default]
    /// Before progress bar
    Prefix,
    /// After progress bar
    Suffix,
}

impl<'a> Message<'a> {
    fn display(self, prefix: bool) -> MessageDisplay<'a> {
        MessageDisplay {
            message: self,
            prefix,
        }
    }
}

struct MessageDisplay<'a> {
    message: Message<'a>,
    prefix: bool,
}

impl<'a> Display for MessageDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.message.position {
            MessagePosition::Prefix => {
                if self.prefix {
                    write!(f, "{}", self.message.message.unwrap_or("{msg}"))
                } else {
                    Ok(())
                }
            }
            MessagePosition::Suffix => {
                if self.prefix {
                    Ok(())
                } else {
                    write!(f, "{}", self.message.message.unwrap_or("{msg}"))
                }
            }
        }
    }
}

#[must_use]
/// Construct a progress bar style
///
/// # Panics
/// - Invalid template
pub fn style(
    progress_opts: Option<ProgressOptions>,
    message_position: Option<Message<'_>>,
) -> ProgressStyle {
    const PROGRESS_CHARS: &str = "=> ";

    let progress_opts = progress_opts.unwrap_or_default();
    let message_position = message_position.unwrap_or_default();

    ProgressStyle::with_template(&format!(
        "{{prefix}} {prefix_message} {{spinner}} [{{wide_bar}}] {progress} ({{eta}}) {suffix}",
        prefix_message = message_position.display(true),
        suffix = message_position.display(false),
        progress = match progress_opts {
            ProgressOptions::Bytes => "{bytes}/{total_bytes} {bytes_per_sec}",
            ProgressOptions::PosLen => "{pos}/{len}",
            ProgressOptions::Hide => "",
        },
    ))
    .unwrap()
    .progress_chars(PROGRESS_CHARS)
}

/// A `ProgressBar`
#[derive(Clone)]
pub struct ProgressBar {
    bar: indicatif::ProgressBar,
    unit: Unit,
    step: Arc<AtomicUsize>,
    id: gix::progress::Id,
}

impl ProgressBar {
    /// Create a new progress bar
    #[must_use]
    pub fn new(total: u64) -> Self {
        Self {
            bar: indicatif::ProgressBar::new(total),
            unit: gix::progress::unit::label(""),
            step: Arc::new(AtomicUsize::new(1)),
            id: [0, 0, 0, 0],
        }
    }

    /// Set the step of the progress bar
    pub fn set_step(&self, step: usize) {
        self.step.store(step, Ordering::Relaxed);
    }

    /// Set the unit of the progress bar
    pub fn set_unit(&mut self, unit: impl Into<Unit>) {
        self.unit = unit.into();
    }

    /// Increment the progress bar
    pub fn inc(&self, amount: u64) {
        self.bar.inc(amount);
    }

    /// Finish the progress bar
    pub fn finish(&self) {
        self.bar.finish();
    }
}

impl From<indicatif::ProgressBar> for ProgressBar {
    fn from(progress: indicatif::ProgressBar) -> Self {
        Self {
            bar: progress,
            unit: gix::progress::unit::label(""),
            step: Arc::new(AtomicUsize::new(1)),
            id: [0, 0, 0, 0],
        }
    }
}

impl From<ProgressBar> for indicatif::ProgressBar {
    fn from(progress: ProgressBar) -> Self {
        progress.bar
    }
}

/// A multi-progress handler
#[derive(Clone)]
pub struct MultiProgressHandler {
    bars: Vec<ProgressBar>,
}

impl MultiProgressHandler {
    /// Create a new multi-progress handler
    #[must_use]
    pub fn new() -> Self {
        Self { bars: vec![] }
    }

    /// Add a child progress bar to the multi-progress handler
    pub fn add(&mut self, bar: ProgressBar) {
        self.bars.push(bar);
    }

    /// Finish a child progress bar
    ///
    /// # Panics
    /// - If the child progress bar does not exist
    pub fn finish(&mut self, id: gix::progress::Id) {
        let position = self.bars.iter().position(|bar| bar.id == id).unwrap();
        let bar = self.bars.remove(position);
        bar.finish();
    }

    /// Finish all child progress bars
    pub fn finish_all(&mut self) {
        for bar in &self.bars {
            bar.finish();
        }

        self.bars.clear();
    }
}

impl Default for MultiProgressHandler {
    fn default() -> Self {
        Self::new()
    }
}
