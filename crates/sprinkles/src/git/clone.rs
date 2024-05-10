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

use std::{path::Path, sync::atomic::AtomicBool};

use gix::{
    clone::PrepareFetch,
    create::{self, Options as CreateOptions},
    open::Options as OpenOptions,
    Repository,
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
    #[error("No pack received from remote")]
    NoPackReceived,
}

/// Clone result
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Clone a git repository
///
/// # Errors
/// - Git error
pub fn clone<P>(url: &str, path: impl AsRef<Path>, pb: P) -> Result<Repository>
where
    P: gix::NestedProgress,
    P::SubProgress: 'static,
{
    let interrupt = AtomicBool::new(false);

    // Fetch the latest changes from the remote repository
    let mut fetch = PrepareFetch::new(
        url,
        path,
        create::Kind::WithWorktree,
        CreateOptions::default(),
        OpenOptions::default(),
    )?;

    let (mut checkout, outcome) = fetch.fetch_then_checkout(pb, &interrupt)?;

    match outcome.status {
        gix::remote::fetch::Status::NoPackReceived { dry_run, .. } => {
            if !dry_run {
                return Err(Error::NoPackReceived);
            }
        }
        gix::remote::fetch::Status::Change { .. } => {}
    }

    let (repo, _) = checkout.main_worktree(gix::progress::Discard, &interrupt)?;

    Ok(repo)
}
