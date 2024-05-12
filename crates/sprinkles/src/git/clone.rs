//! Clones a git repository
//!
//! This module is a modification of the git2 "clone" example, as mentioned in the license comment below.

/*
* libgit2 "clone" example
*
* Written by the libgit2 contributors
*
* To the extent possible under law, the author(s) have dedicated all copyright
* and related and neighboring rights to this software to the public domain
* worldwide. This software is distributed without any warranty.
*
* You should have received a copy of the CC0 Public Domain Dedication along
* with this software. If not, see
* <http://creativecommons.org/publicdomain/zero/1.0/>.
*
 * Adapted by me (Juliette Cordor)
*/

#![allow(clippy::result_large_err)]

use std::path::Path;

use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    FetchOptions, Progress, RemoteCallbacks,
};

pub use gix::progress;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Clone error
pub enum Error {
    #[error("Clone error: {0}")]
    Clone(#[from] gix::clone::Error),
    #[error("Fetch error: {0}")]
    Fetch(#[from] gix::clone::fetch::Error),
    #[error("Failed to checkout main worktree: {0}")]
    Checkout(#[from] gix::clone::checkout::main_worktree::Error),
    #[error("Git2 error: {0}")]
    Git2(#[from] git2::Error),
    #[error("No pack received from remote")]
    NoPackReceived,
}

/// Clone result
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Clone a git repository
///
/// # Errors
/// - Git error
pub fn clone<P>(
    url: &str,
    path: impl AsRef<Path>,
    fetch_progress: fn(Progress<'_>) -> (),
    checkout_progress: fn(Option<&Path>, usize, usize) -> (),
) -> Result<git2::Repository>
where
    P: gix::NestedProgress,
    P::SubProgress: 'static,
{
    let mut cb = RemoteCallbacks::new();
    cb.transfer_progress(|stats| {
        fetch_progress(stats);
        true
    });
    let mut co = CheckoutBuilder::new();
    co.progress(checkout_progress);

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    let repo = RepoBuilder::new()
        .fetch_options(fo)
        .with_checkout(co)
        .clone(url, path.as_ref())?;

    Ok(repo)
}
