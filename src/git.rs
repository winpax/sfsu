use derive_more::Deref;
use git2::{Commit, DiffOptions, FetchOptions, Remote, Repository};

use crate::buckets::Bucket;

use self::pull::ProgressCallback;

mod pull;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("Could not find the active branch (HEAD)")]
    NoActiveBranch,

    #[error("Git error: {0}")]
    Git2(#[from] git2::Error),
}

pub type Result<T> = std::result::Result<T, RepoError>;

#[derive(Deref)]
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

    /// Get the current remote branch
    ///
    /// # Errors
    /// - Missing head
    ///
    /// # Panics
    /// - Non-utf8 branch name
    pub fn main_remote(&self) -> Result<Remote<'_>> {
        Ok(self
            .0
            .find_remote(self.0.head()?.name().expect("utf8 branch name"))?)
    }

    /// Get the current branch
    ///
    /// # Errors
    /// - No active branch
    pub fn current_branch(&self) -> Result<String> {
        self.0
            .head()?
            .shorthand()
            .ok_or(RepoError::NoActiveBranch)
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
    /// - No active branch
    /// - No reference "`FETCH_HEAD`"
    pub fn outdated(&self) -> Result<bool> {
        self.fetch()?;

        // Get the local and remote HEADs
        let local_head = self.latest_commit()?;
        let fetch_head = self.0.find_reference("FETCH_HEAD")?.peel_to_commit()?;

        Ok(local_head.id() != fetch_head.id())
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
}
