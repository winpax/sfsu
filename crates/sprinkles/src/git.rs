//! Scoop git helpers

use std::{ffi::OsStr, fmt::Display, process::Command};

use derive_more::Deref;
use git2::{Commit, DiffOptions, Direction, FetchOptions, Remote, Repository, Sort};

use crate::{buckets::Bucket, opt::ResultIntoOption, Scoop};

use self::pull::ProgressCallback;

pub mod pull;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Repo error
pub enum Error {
    #[error("Could not find the active branch (HEAD)")]
    NoActiveBranch,
    #[error("Git error: {0}")]
    Git2(#[from] git2::Error),
    #[error("No remote named {0}")]
    MissingRemote(String),
    #[error("Missing head in remote")]
    MissingHead,
    #[error("Invalid utf8")]
    NonUtf8,
}

/// Repo result type
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Deref)]
/// A git repository
pub struct Repo(Repository);

impl Repo {
    /// Open the repository from the bucket path
    ///
    /// # Errors
    /// - The bucket could not be opened as a repository
    pub fn from_bucket(bucket: &Bucket) -> Result<Self> {
        let repo = Repository::open(bucket.path())?;

        Ok(Self(repo))
    }

    /// Open Scoop app repository
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    pub fn scoop_app() -> Result<Self> {
        let scoop_path = Scoop::apps_path().join("scoop").join("current");
        let repo = Repository::open(scoop_path)?;

        Ok(Self(repo))
    }

    #[must_use]
    /// Get the origin remote
    pub fn origin(&self) -> Option<Remote<'_>> {
        self.find_remote("origin").into_option()
    }

    /// Get the current branch
    ///
    /// # Errors
    /// - No active branch
    pub fn current_branch(&self) -> Result<String> {
        self.0
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
        let mut remote = self.0.find_remote("origin")?;
        remote.fetch(&[current_branch], Some(&mut fetch_options), None)?;

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
            .current_dir(self.path())
            .arg("-C")
            .arg(cd)
            .arg("log")
            .arg(format!("-n {n}"))
            .arg("-s")
            .arg(format!("--format='{format}'"));

        Ok(command)
    }
}
