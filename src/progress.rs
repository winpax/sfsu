#![allow(clippy::module_name_repetitions)]

use indicatif::ProgressStyle;

#[derive(Debug, Copy, Clone, Default)]
pub enum ProgressOptions {
    /// Show bytes/total bytes as progress
    Bytes,
    #[default]
    /// Show pos/len as progress
    PosLen,
}

#[derive(Debug, Copy, Clone, Default)]
pub enum MessagePosition {
    #[default]
    Prefix,
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
        }
    ))
    .unwrap()
    .progress_chars(PROGRESS_CHARS)
}
