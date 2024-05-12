//! Scoop git helpers

use std::{ffi::OsStr, fmt::Display, path::PathBuf, process::Command};

use git2::{Commit, Direction, FetchOptions, Oid, Progress, Remote, Repository};
use gix::traverse::commit::simple::Sorting;
use indicatif::ProgressBar;

use crate::{buckets::Bucket, contexts::ScoopContext};

use self::pull::ProgressCallback;

pub mod clone;
pub mod errors;
pub mod parity;
mod pull;

/// Get the path to the git executable
///
/// This is just an alias for [`which::which`]
///
/// # Errors
/// - Git path could not be found
/// - The current dir and path list were empty
/// - The found path could not be canonicalized
pub fn which() -> which::Result<PathBuf> {
    which::which("git")
}

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
    #[error("Gitoxide error: {0}")]
    Gitoxide(Box<errors::GitoxideError>),
    #[error("No remote named {0}")]
    MissingRemote(String),
    #[error("Missing head in remote")]
    MissingHead,
    #[error("Invalid utf8")]
    NonUtf8,
}

/// Repo result type
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A git repository
pub struct Repo {
    git2: Repository,
    gitoxide: gix::Repository,
}

impl Repo {
    #[must_use]
    /// Get the underlying git repository
    pub fn git2(&self) -> &Repository {
        &self.git2
    }

    /// Convert into a gitoxide repository
    ///
    /// # Errors
    /// - Git path could not be found
    /// - Gitoxide error
    pub fn gitoxide(&self) -> &gix::Repository {
        &self.gitoxide
    }

    /// Open the repository from the bucket path
    ///
    /// # Errors
    /// - The bucket could not be opened as a repository
    pub fn from_bucket(bucket: &Bucket) -> Result<Self> {
        let git2 = Repository::open(bucket.path())?;
        let gitoxide = gix::open(bucket.path())?;

        Ok(Self { git2, gitoxide })
    }

    /// Open Scoop app repository
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    pub fn scoop_app<C>(context: &impl ScoopContext<C>) -> Result<Self> {
        let scoop_path = context.apps_path().join("scoop").join("current");
        let git2 = Repository::open(&scoop_path)?;
        let gitoxide = gix::open(&scoop_path)?;

        Ok(Self { git2, gitoxide })
    }

    #[must_use]
    /// Get the origin remote
    pub fn origin(&self) -> Option<Remote<'_>> {
        self.git2.find_remote("origin").ok()
    }

    /// Checkout to another branch
    ///
    /// # Errors
    /// - Git error
    /// - No active branch
    /// - No remote named "origin"
    pub fn checkout(&self, branch: &str) -> Result<()> {
        let branch = format!("refs/heads/{branch}");
        self.git2.set_head(&branch)?;
        self.git2.checkout_head(None)?;

        // Reset to ensure the working directory is clean
        self.git2.reset(
            self.latest_commit()?.as_object(),
            git2::ResetType::Hard,
            None,
        )?;

        Ok(())
    }

    /// Get the current branch
    ///
    /// # Errors
    /// - No active branch
    pub fn current_branch(&self) -> Result<String> {
        self.git2
            .head()?
            .shorthand()
            .ok_or(Error::NoActiveBranch)
            .map(std::string::ToString::to_string)
    }

    /// Fetch latest changes in the repo
    ///
    /// # Errors
    /// - No remote named "origin"
    /// - No active branch
    pub fn fetch(&self) -> Result<()> {
        let current_branch = self.current_branch()?;

        // Fetch the latest changes from the remote repository
        let mut fetch_options = FetchOptions::new();
        fetch_options.update_fetchhead(true);
        let mut remote = self.git2.find_remote("origin")?;
        remote.fetch(&[current_branch], Some(&mut fetch_options), None)?;

        Ok(())
    }

    /// Get the latest commit in the remote repository
    ///
    /// # Errors
    /// - No remote named "origin"
    /// - Missing head
    pub fn latest_remote_commit(&self) -> Result<Oid> {
        let mut remote = self
            .origin()
            .ok_or(Error::MissingRemote("origin".to_string()))?;

        let connection = remote.connect_auth(Direction::Fetch, None, None)?;

        let current_branch = self.current_branch()?;
        let head = connection
            .list()?
            .iter()
            .find(|head| {
                let name = head.name();
                name == format!("refs/heads/{current_branch}")
            })
            .ok_or(Error::MissingHead)?;

        Ok(head.oid())
    }

    /// Checks if the bucket is outdated
    ///
    /// # Errors
    /// - No remote named "origin"
    pub fn outdated(&self) -> Result<bool> {
        let head = self.latest_remote_commit()?;

        let local_head = self.latest_commit()?;

        debug!(
            "{}/{} from repo '{}'",
            head,
            local_head.id(),
            self.git2.path().display()
        );

        Ok(local_head.id() != head)
    }

    /// Get the latest commit
    ///
    /// # Errors
    /// - Missing head
    /// - Missing latest commit
    pub fn latest_commit(&self) -> Result<Commit<'_>> {
        Ok(self.git2.head()?.peel_to_commit()?)
    }

    /// Update the bucket by pulling any changes
    pub fn update(&self) {
        unimplemented!()
    }

    /// Get the remote url of the bucket
    pub fn get_remote(&self) {
        unimplemented!()
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
        let repo = self.gitoxide();

        let current_commit = repo.head_commit()?;

        pull::pull(self, None, Some(self.current_branch()?.as_str()), stats_cb)?;

        let post_pull_commit = repo.head_commit()?;

        let revwalk = repo
            .rev_walk([post_pull_commit.id])
            .sorting(Sorting::ByCommitTimeNewestFirst);

        let mut changelog = Vec::new();
        for commit in revwalk.all()? {
            let info = commit?;
            let Ok(commit) = info.object() else {
                continue;
            };

            let oid = info.id();

            if oid == current_commit.id() {
                break;
            }

            if let Ok(msg) = commit.message() {
                let summary = msg.summary();
                changelog.push(summary.to_string());
            }
        }

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
        let git_path = which::which("git")?;

        let mut command = Command::new(git_path);

        command
            .current_dir(self.git2.path().parent().expect("parent dir in .git path"))
            .arg("-C")
            .arg(cd)
            .arg("log")
            .arg(format!("-n {n}"))
            .arg("-s")
            .arg(format!("--format='{format}'"));

        Ok(command)
    }
}
