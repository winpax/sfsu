//! Git specific error helpers

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// A collection of gitoxide errors
pub enum GitoxideError {
    #[error("Gitoxide error: {0}")]
    GitoxideOpen(#[from] gix::open::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideTraverse(#[from] gix::traverse::commit::simple::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideRevWalk(#[from] gix::revision::walk::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideHead(#[from] gix::reference::head_commit::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideDecode(#[from] gix_object::decode::Error),
}

impl From<GitoxideError> for super::Error {
    fn from(error: GitoxideError) -> Self {
        Self::Gitoxide(Box::new(error))
    }
}
