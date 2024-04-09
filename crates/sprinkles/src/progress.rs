//! Progress helpers for the CLI

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

#[derive(Debug, Copy, Clone, Default)]
/// Message position
pub enum MessagePosition {
    #[default]
    /// Before progress bar
    Prefix,
    /// After progress bar
    Suffix,
}

#[must_use]
/// Construct a progress bar style
///
/// # Panics
/// - Invalid template
pub fn style(
    progress_opts: Option<ProgressOptions>,
    message_position: Option<MessagePosition>,
) -> ProgressStyle {
    const PROGRESS_CHARS: &str = "=> ";

    let progress_opts = progress_opts.unwrap_or_default();
    let message_position = message_position.unwrap_or_default();

    ProgressStyle::with_template(&format!(
        "{prefix} {{prefix}} [{{wide_bar}}] {progress} ({{eta}}) {suffix}",
        prefix = match message_position {
            MessagePosition::Prefix => "{msg}",
            MessagePosition::Suffix => "",
        },
        suffix = match message_position {
            MessagePosition::Prefix => "",
            MessagePosition::Suffix => "{msg}",
        },
        progress = match progress_opts {
            ProgressOptions::Bytes => "{bytes}/{total_bytes}",
            ProgressOptions::PosLen => "{pos}/{len}",
            ProgressOptions::Hide => "",
        },
    ))
    .unwrap()
    .progress_chars(PROGRESS_CHARS)
}
