//! Progress helpers for the CLI

mod gitoxide;

use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use gix::progress::{unit::Kind, AtomicStep, Unit};
use indicatif::ProgressStyle;
use parking_lot::Mutex;

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

#[derive(Debug, Copy, Clone)]
/// Message position
pub enum Message<'a> {
    /// Before progress bar
    Prefix(Option<&'a str>),
    /// After progress bar
    Suffix(Option<&'a str>),
}

impl<'a> Default for Message<'a> {
    fn default() -> Self {
        Message::Prefix(None)
    }
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
        match self.message {
            Message::Prefix(message) => {
                if self.prefix {
                    write!(f, "{}", message.unwrap_or("{msg}"))
                } else {
                    Ok(())
                }
            }
            Message::Suffix(message) => {
                if self.prefix {
                    Ok(())
                } else {
                    write!(f, "{}", message.unwrap_or("{msg}"))
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

#[derive(Clone)]
pub struct ProgressBar {
    bar: indicatif::ProgressBar,
    unit: Unit,
    step: Arc<AtomicUsize>,
    id: gix::progress::Id,
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        Self {
            bar: indicatif::ProgressBar::new(total),
            unit: gix::progress::unit::label(""),
            step: Arc::new(AtomicUsize::new(1)),
            id: [0, 0, 0, 0],
        }
    }

    pub fn set_step(&self, step: usize) {
        self.step.store(step, Ordering::Relaxed);
    }

    pub fn set_unit(&mut self, unit: impl Into<Unit>) {
        self.unit = unit.into();
    }

    pub fn inc(&self, amount: u64) {
        self.bar.inc(amount);
    }

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

#[derive(Clone)]
pub struct MultiProgressHandler {
    bars: Vec<ProgressBar>,
}

impl MultiProgressHandler {
    pub fn new() -> Self {
        Self { bars: vec![] }
    }

    pub fn add(&mut self, bar: ProgressBar) {
        self.bars.push(bar);
    }

    pub fn finish(&mut self, id: gix::progress::Id) {
        let position = self.bars.iter().position(|bar| bar.id == id).unwrap();
        let bar = self.bars.remove(position);
        bar.finish();
    }

    pub fn finish_all(&mut self) {
        for bar in &self.bars {
            bar.finish();
        }

        self.bars.clear();
    }
}
