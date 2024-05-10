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
    #[error("Gitoxide error: {0}")]
    GitoxideRevWalkGraph(#[from] gix::object::find::existing::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideCommit(#[from] gix::object::commit::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideRewrites(#[from] gix::diff::new_rewrites::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideObjectPeel(#[from] gix::object::peel::to_kind::Error),
    #[error("Gitoxide error: {0}")]
    GitoxideObjectDiff(#[from] gix::object::tree::diff::for_each::Error),
}

impl<T> From<T> for super::Error
where
    GitoxideError: From<T>,
{
    fn from(value: T) -> Self {
        Self::Gitoxide(Box::new(value.into()))
    }
}
