//! Progress helpers for the CLI

use std::{
    fmt::Display,
    sync::{atomic::AtomicUsize, Arc},
};

use gix::NestedProgress;
use indicatif::ProgressStyle;

#[derive(Debug, Clone)]
pub struct MultiProgress {
    mp: indicatif::MultiProgress,
}

impl MultiProgress {
    pub fn new() -> Self {
        Self {
            mp: indicatif::MultiProgress::new(),
        }
    }

    pub fn add_progress_bar(&self, bar: ProgressBar) {
        self.mp.add(bar.bar);
    }
}

impl From<MultiProgress> for indicatif::MultiProgress {
    fn from(value: MultiProgress) -> Self {
        value.mp
    }
}

impl From<indicatif::MultiProgress> for MultiProgress {
    fn from(value: indicatif::MultiProgress) -> Self {
        Self { mp: value }
    }
}

impl NestedProgress for MultiProgress {
    type SubProgress = ProgressBar;

    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress {
        let pb = ProgressBar::new(0);
        pb.bar.set_prefix(name.into());

        self.add_progress_bar(pb.clone());

        pb
    }

    fn add_child_with_id(
        &mut self,
        name: impl Into<String>,
        id: gix::progress::Id,
    ) -> Self::SubProgress {
        let mut pb = ProgressBar::new(0);
        pb.bar.set_prefix(name.into());
        pb.id = id;

        self.add_progress_bar(pb.clone());

        pb
    }
}

impl gix::Progress for MultiProgress {
    fn init(
        &mut self,
        max: Option<gix::progress::prodash::progress::Step>,
        unit: Option<gix::progress::Unit>,
    ) {
        self.add_progress_bar(ProgressBar::new(max.unwrap_or_default() as u64));
    }

    fn set_name(&mut self, name: String) {
        todo!()
    }

    fn name(&self) -> Option<String> {
        todo!()
    }

    fn id(&self) -> gix::progress::Id {
        todo!()
    }

    fn message(&self, level: gix::progress::MessageLevel, message: String) {
        todo!()
    }
}

impl gix::Count for MultiProgress {
    fn set(&self, step: gix::progress::prodash::progress::Step) {
        todo!()
    }

    fn step(&self) -> gix::progress::prodash::progress::Step {
        todo!()
    }

    fn inc_by(&self, step: gix::progress::prodash::progress::Step) {
        todo!()
    }

    fn counter(&self) -> gix::progress::StepShared {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct ProgressBar {
    bar: indicatif::ProgressBar,
    id: gix::progress::Id,
}

impl ProgressBar {
    pub fn new(len: u64) -> Self {
        Self {
            bar: indicatif::ProgressBar::new(len),
            id: [0; 4],
        }
    }

    pub fn with_style(self, style: ProgressStyle) -> Self {
        Self {
            bar: self.bar.with_style(style),
            id: [0; 4],
        }
    }
}

impl From<ProgressBar> for indicatif::ProgressBar {
    fn from(value: ProgressBar) -> Self {
        value.bar
    }
}

impl From<indicatif::ProgressBar> for ProgressBar {
    fn from(value: indicatif::ProgressBar) -> Self {
        Self {
            bar: value,
            id: [0; 4],
        }
    }
}

impl NestedProgress for ProgressBar {
    type SubProgress = ProgressBar;

    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress {
        let pb = Self::new(0);
        pb.bar.set_prefix(name.into());

        pb
    }
    // fn add_child(&mut self, len: u64) -> Self::SubProgress {
    //     Self::SubProgress::new(len)
    // }

    fn add_child_with_id(
        &mut self,
        name: impl Into<String>,
        id: gix::progress::Id,
    ) -> Self::SubProgress {
        todo!()
    }
}

impl gix::Progress for ProgressBar {
    fn init(
        &mut self,
        max: Option<gix::progress::prodash::progress::Step>,
        unit: Option<gix::progress::Unit>,
    ) {
        *self = Self::new(max.unwrap_or_default() as u64);
    }

    fn set_name(&mut self, name: String) {
        self.bar.set_prefix(name);
    }

    fn name(&self) -> Option<String> {
        let prefix = self.bar.prefix();

        if prefix.len() == 0 {
            None
        } else {
            Some(prefix.to_string())
        }
    }

    fn id(&self) -> gix::progress::Id {
        self.id
    }

    fn message(&self, level: gix::progress::MessageLevel, message: String) {
        self.bar.set_message(message);
    }
}

impl gix::Count for ProgressBar {
    fn set(&self, step: gix::progress::prodash::progress::Step) {
        self.bar.set_length(step as u64)
    }

    fn step(&self) -> gix::progress::prodash::progress::Step {
        self.bar.inc(1);
        1
    }

    fn inc_by(&self, step: gix::progress::prodash::progress::Step) {
        self.bar.inc(step as u64)
    }

    fn counter(&self) -> gix::progress::StepShared {
        Arc::new(AtomicUsize::new(self.bar.position() as usize))
    }
}

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
