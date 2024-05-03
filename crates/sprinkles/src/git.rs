//! Scoop git helpers

use std::{ffi::OsStr, fmt::Display, process::Command, sync::atomic::AtomicBool};

use derive_more::Deref;
use git2::{Commit, DiffOptions, Direction, FetchOptions, Progress, Sort};
use gix::{
    clone::PrepareFetch,
    create::{self, Kind, Options},
    repository, Remote, Repository,
};
use indicatif::ProgressBar;

use crate::{buckets::Bucket, Scoop};

use self::pull::ProgressCallback;

mod pull;

#[doc(hidden)]
/// Progress callback
///
/// This is meant primarily for internal sfsu use.
/// You are welcome to use this yourself, but it will likely not meet your requirements.
pub fn __stats_callback(stats: &Progress<'_>, thin: bool, pb: &ProgressBar) {
    if thin {
        pb.set_position(stats.indexed_objects() as u64);
        pb.set_length(stats.total_objects() as u64);

        return;
    }

    if stats.received_objects() == stats.total_objects() {
        pb.set_position(stats.indexed_deltas() as u64);
        pb.set_length(stats.total_deltas() as u64);
        pb.set_message("Resolving deltas");
    } else if stats.total_objects() > 0 {
        pb.set_position(stats.received_objects() as u64);
        pb.set_length(stats.total_objects() as u64);
        pb.set_message("Receiving objects");
    }
}

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Repo error
pub enum Error {
    #[error("Could not find the active branch (HEAD)")]
    NoActiveBranch,
    #[error("Git error: {0}")]
    Git2(#[from] git2::Error),
    #[error("Open Repo error: {0}")]
    OpenRepo(#[from] gix::open::Error),
    #[error("Cannot find reference: {0}")]
    MissingReference(#[from] gix::reference::find::existing::Error),
    #[error("Cannot find remote: {0}")]
    MissingRemote(#[from] gix::remote::find::existing::Error),
    #[error("Missing head in remote")]
    MissingHead,
    #[error("Cloning: {0}")]
    CloneError(#[from] gix::clone::Error),
    #[error("Invalid utf8")]
    NonUtf8,
}

/// Repo result type
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Deref)]
/// A git repository
pub struct Repo(Repository);

impl Repo {
    /// Open the repository from the bucket path
    ///
    /// # Errors
    /// - The bucket could not be opened as a repository
    pub fn from_bucket(bucket: &Bucket) -> Result<Self> {
        let repo = gix::open(bucket.path())?;

        Ok(Self(repo))
    }

    /// Open Scoop app repository
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    pub fn scoop_app() -> Result<Self> {
        let scoop_path = Scoop::apps_path().join("scoop").join("current");
        let repo = gix::open(scoop_path)?;

        Ok(Self(repo))
    }

    #[must_use]
    /// Get the origin remote
    pub fn origin(&self) -> Option<Remote<'_>> {
        self.find_remote("origin").ok()
    }

    /// Get the current branch
    ///
    /// # Errors
    /// - No active branch
    pub fn current_branch(&self) -> Result<String> {
        Ok(self.0.head()?.name().to_path().display().to_string())
    }

    /// Fetch latest changes in the repo
    ///
    /// # Errors
    /// - No remote named "origin"
    /// - No active branch
    pub fn fetch(&self) -> Result<()> {
        let current_branch = self.current_branch()?;

        let mut remote = self.find_remote("origin")?;

        // Fetch the latest changes from the remote repository
        let mut fetch = PrepareFetch::new(
            *remote.url(gix::remote::Direction::Fetch).unwrap(),
            self.path(),
            match self.kind() {
                repository::Kind::Submodule => create::Kind::WithWorktree,
                repository::Kind::Bare => create::Kind::Bare,
                repository::Kind::WorkTree { is_linked } => create::Kind::WithWorktree,
            },
            Options::default(),
            *self.open_options(),
        )?;
        let pb = crate::progress::ProgressBar::new(0);
        fetch.fetch_only(pb, &AtomicBool::new(false));

        Ok(())
    }

    /// Checks if the bucket is outdated
    ///
    /// # Errors
    /// - No remote named "origin"
    pub fn outdated(&self) -> Result<bool> {
        let mut remote = self
            .origin()
            .ok_or(Error::MissingRemote("origin".to_string()))?;

        let connection = remote.connect_auth(Direction::Fetch, None, None)?;

        let head = connection.list()?.first().ok_or(Error::MissingHead)?;

        debug!(
            "{}\t{} from repo '{}'",
            head.oid(),
            head.name(),
            self.path().display()
        );

        let local_head = self.latest_commit()?;

        Ok(local_head.id() != head.oid())
    }

    /// Get the latest commit
    ///
    /// # Errors
    /// - Missing head
    /// - Missing latest commit
    pub fn latest_commit(&self) -> Result<Commit<'_>> {
        Ok(self.0.head()?.peel_to_commit()?)
    }

    /// Update the bucket by pulling any changes
    pub fn update(&self) {
        unimplemented!()
    }

    /// Get the remote url of the bucket
    pub fn get_remote(&self) {
        unimplemented!()
    }

    pub(crate) fn default_diff_options() -> DiffOptions {
        let mut diff_options = DiffOptions::new();

        diff_options
            .ignore_submodules(true)
            .enable_fast_untracked_dirs(true)
            .context_lines(0)
            .interhunk_lines(0)
            .disable_pathspec_match(true)
            .ignore_whitespace(true)
            .ignore_whitespace_change(true)
            .ignore_whitespace_eol(true)
            .force_binary(true)
            .include_ignored(false)
            .include_typechange(false)
            .include_ignored(false)
            .include_typechange_trees(false)
            .include_unmodified(false)
            .include_unreadable(false)
            .include_unreadable_as_untracked(false)
            .include_untracked(false);

        diff_options
    }

    /// Pull the latest changes from the remote repository
    ///
    /// # Errors
    /// - No active branch
    /// - No remote named "origin"
    /// - No reference "`FETCH_HEAD`"
    /// - Missing head
    /// - Missing latest commit
    /// - Git error
    pub fn pull(&self, stats_cb: Option<ProgressCallback<'_>>) -> Result<()> {
        let current_branch = self.current_branch()?;

        pull::pull(self, None, Some(current_branch.as_str()), stats_cb)?;

        Ok(())
    }

    /// Pull the latest changes from the remote repository
    ///
    /// # Errors
    /// - No active branch
    /// - No remote named "origin"
    /// - No reference "`FETCH_HEAD`"
    /// - Missing head
    /// - Missing latest commit
    /// - Git error
    pub fn pull_with_changelog(
        &self,
        stats_cb: Option<ProgressCallback<'_>>,
    ) -> Result<Vec<String>> {
        let current_branch = self.current_branch()?;

        let current_commit = self.latest_commit()?;

        pull::pull(self, None, Some(current_branch.as_str()), stats_cb)?;

        // let post_pull_commit = self.latest_commit()?;

        let mut revwalk = self.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(Sort::TOPOLOGICAL)?;

        let mut changelog = Vec::new();
        for oid in revwalk {
            let oid = oid?;

            if oid == current_commit.id() {
                break;
            }

            let commmit = self.find_commit(oid)?;

            if let Some(msg) = commmit.message() {
                if let Some(first_line) = msg.lines().next() {
                    changelog.push(first_line.trim().to_string());
                }
            }
        }

        changelog.reverse();

        Ok(changelog)
    }

    /// Equivalent of `git log -n {n} -s --format='{format}'`
    ///
    /// # Panics
    /// - Git repo path could not be found
    ///
    /// # Errors
    /// - Git path could not be found
    pub fn log(
        &self,
        cd: impl AsRef<OsStr>,
        n: usize,
        format: impl Display,
    ) -> which::Result<Command> {
        let git_path = Scoop::git_path()?;

        let mut command = Command::new(git_path);

        command
            .current_dir(self.path().parent().expect("parent dir in .git path"))
            .arg("-C")
            .arg(cd)
            .arg("log")
            .arg(format!("-n {n}"))
            .arg("-s")
            .arg(format!("--format='{format}'"));

        Ok(command)
    }
}
