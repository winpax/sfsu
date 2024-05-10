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

use std::sync::atomic::AtomicBool;

use gix::{
    clone::PrepareFetch,
    create::{self, Options as CreateOptions},
    open::Options as OpenOptions,
    Url,
};

pub fn clone(url: Url, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Fetch the latest changes from the remote repository
    let mut fetch = PrepareFetch::new(
        url,
        path,
        create::Kind::WithWorktree,
        CreateOptions::default(),
        OpenOptions::default(),
    )?;
    // let pb = crate::progress::ProgressBar::new(0);
    fetch.fetch_only(pb, &AtomicBool::new(false));

    Ok(())
}
