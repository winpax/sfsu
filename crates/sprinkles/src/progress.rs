//! Progress helpers for the CLI

use std::fmt::Display;

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
        "{{prefix}} {prefix} [{{wide_bar}}] {progress} ({{eta}}) {suffix}",
        prefix = message_position.display(true),
        suffix = message_position.display(false),
        progress = match progress_opts {
            ProgressOptions::Bytes => "{bytes}/{total_bytes}",
            ProgressOptions::PosLen => "{pos}/{len}",
            ProgressOptions::Hide => "",
        },
    ))
    .unwrap()
    .progress_chars(PROGRESS_CHARS)
}
