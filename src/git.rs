use derive_more::Deref;
use git2::{Commit, FetchOptions, Remote, Repository};

use crate::buckets::Bucket;

// pub mod pull;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("Could not find the active branch (HEAD)")]
    NoActiveBranch,

    #[error("Git error: {0}")]
    Git2(#[from] git2::Error),
}

pub type RepoResult<T> = std::result::Result<T, RepoError>;

#[derive(Deref)]
pub struct Repo(Repository);

impl Repo {
    /// Open the repository from the bucket path
    ///
    /// # Errors
    /// - The bucket could not be opened as a repository
    pub fn from_bucket(bucket: &Bucket) -> RepoResult<Self> {
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
    pub fn main_remote(&self) -> RepoResult<Remote<'_>> {
        Ok(self
            .0
            .find_remote(self.0.head()?.name().expect("utf8 branch name"))?)
    }

    /// Get the current branch
    ///
    /// # Errors
    /// - No active branch
    pub fn current_branch(&self) -> RepoResult<String> {
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
    pub fn fetch(&self) -> RepoResult<()> {
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
    pub fn outdated(&self) -> RepoResult<bool> {
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
    pub fn latest_commit(&self) -> RepoResult<Commit<'_>> {
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
}
