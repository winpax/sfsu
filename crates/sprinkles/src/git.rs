//! Scoop git helpers

use std::{
    ffi::OsStr,
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
};

use gix::{
    bstr::BStr, remote::ref_map, traverse::commit::simple::Sorting, Commit, ObjectId, Repository,
};

use crate::{buckets::Bucket, config, contexts::ScoopContext};

use pull::ProgressCallback;

pub mod clone;
pub mod errors;
pub mod options;
pub mod parity;
mod pull;

pub mod implementations {
    //! Re-exports of the different Git implementations used

    pub use git2;
    pub use gix;
}

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

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Repo error
pub enum Error {
    #[error("Could not find the active branch (HEAD)")]
    NoActiveBranch,
    #[error("Could not find the parent directory for the .git directory")]
    GitParent,
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
    git2: git2::Repository,
    gitoxide: Repository,
}

impl Repo {
    #[must_use]
    /// Get the underlying git repository
    pub fn git2(&self) -> &git2::Repository {
        &self.git2
    }

    /// Convert into a gitoxide repository
    ///
    /// # Errors
    /// - Git path could not be found
    /// - Gitoxide error
    pub fn gitoxide(&self) -> &Repository {
        &self.gitoxide
    }

    /// Open the repository from the path
    ///
    /// # Errors
    /// - The path could not be opened as a repository
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let gitoxide = gix::open(path.as_ref())?;
        let git2 = git2::Repository::open(path)?;

        Ok(Self { git2, gitoxide })
    }

    /// Open the repository from the bucket path
    ///
    /// # Errors
    /// - The bucket could not be opened as a repository
    pub fn from_bucket(bucket: &Bucket) -> Result<Self> {
        let git2 = git2::Repository::open(bucket.path())?;
        let gitoxide = gix::open(bucket.path())?;

        Ok(Self { git2, gitoxide })
    }

    /// Open Scoop app repository
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    pub fn scoop_app<C>(context: &impl ScoopContext<C>) -> Result<Self> {
        let scoop_path = context.apps_path().join("scoop").join("current");
        let git2 = git2::Repository::open(&scoop_path)?;
        let gitoxide = gix::open(&scoop_path)?;

        Ok(Self { git2, gitoxide })
    }

    /// Get a reference to a named remote
    pub fn find_remote<'a>(&self, name: impl Into<&'a BStr>) -> Option<gix::Remote<'_>> {
        self.gitoxide.find_remote(name).ok()
    }

    #[must_use]
    /// Get the origin remote
    pub fn origin(&self) -> Option<gix::Remote<'_>> {
        self.find_remote("origin")
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
            self.latest_commit_git2()?.as_object(),
            git2::ResetType::Hard,
            None,
        )?;

        Ok(())
    }

    /// Get the current branch
    ///
    /// # Errors
    /// - No active branch
    /// - Detached head
    pub fn current_branch(&self) -> Result<String> {
        let reference = self
            .gitoxide
            .head_name()?
            .ok_or(Error::NoActiveBranch)?
            .to_string();
        let branch_name = reference
            .split('/')
            .last()
            .map(String::from)
            .ok_or(Error::NoActiveBranch)?;

        Ok(branch_name)
    }

    /// Fetch latest changes in the repo
    ///
    /// # Errors
    /// - No remote named "origin"
    /// - No active branch
    pub fn fetch(&self) -> Result<gix::remote::fetch::Outcome> {
        let remote = self
            .origin()
            .ok_or(Error::MissingRemote("origin".to_string()))?;

        let connection = remote.connect(gix::remote::Direction::Fetch)?;

        let fetch =
            connection.prepare_fetch(gix::progress::Discard, ref_map::Options::default())?;

        let outcome = fetch.receive(gix::progress::Discard, &AtomicBool::new(false))?;

        Ok(outcome)
    }

    /// Get the latest commit in the remote repository
    ///
    /// # Errors
    /// - No remote named "origin"
    /// - Missing head
    pub fn latest_remote_commit(&self) -> Result<ObjectId> {
        let remote = self
            .origin()
            .ok_or(Error::MissingRemote("origin".to_string()))?;

        let connection = remote.connect(gix::remote::Direction::Fetch)?;
        let refs = connection
            .ref_map(gix::progress::Discard, ref_map::Options::default())?
            .remote_refs;

        let current_branch = self.current_branch()?;
        let head = refs
            .iter()
            .find_map(|head| {
                let (name, oid, peeled) = head.unpack();
                if name == format!("refs/heads/{current_branch}") {
                    if let Some(peeled) = peeled {
                        Some(peeled)
                    } else if let Some(oid) = oid {
                        Some(oid)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .ok_or(Error::MissingHead)?;

        Ok(head.to_owned())
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
            self.path().ok_or(Error::GitParent)?.display()
        );

        Ok(local_head.id() != head)
    }

    /// Get the latest commit
    ///
    /// # Errors
    /// - Missing head
    /// - Missing latest commit
    pub fn latest_commit_git2(&self) -> Result<git2::Commit<'_>> {
        Ok(self.git2.head()?.peel_to_commit()?)
    }

    /// Get the latest commit
    ///
    /// # Errors
    /// - Missing head
    /// - Missing latest commit
    pub fn latest_commit(&self) -> Result<Commit<'_>> {
        Ok(self.gitoxide.head()?.peel_to_commit_in_place()?)
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
    pub fn pull(
        &self,
        ctx: &impl ScoopContext<config::Scoop>,
        stats_cb: Option<ProgressCallback<'_>>,
    ) -> Result<()> {
        let current_branch = self.current_branch()?;

        pull::pull(ctx, self, None, Some(current_branch.as_str()), stats_cb)?;

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
        ctx: &impl ScoopContext<config::Scoop>,
        stats_cb: Option<ProgressCallback<'_>>,
    ) -> Result<Vec<String>> {
        let repo = self.gitoxide();

        let current_commit = repo.head_commit()?;

        self.pull(ctx, stats_cb)?;

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

    /// Get the path to the git repository
    ///
    /// Will return `None` if the `.git` directory did not have a parent directory
    pub fn path(&self) -> Option<&Path> {
        self.gitoxide.path().parent()
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
            .current_dir(self.path().expect("parent dir in .git path"))
            .arg("-C")
            .arg(cd)
            .arg("log")
            .arg(format!("-n {n}"))
            .arg("-s")
            .arg(format!("--format='{format}'"));

        Ok(command)
    }
}
