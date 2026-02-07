//! Git specific error helpers

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// A collection of gitoxide errors
pub enum GitoxideError {
    #[error("Gitoxide error: {0}")]
    Open(#[from] gix::open::Error),
    #[error("Gitoxide error: {0}")]
    Traverse(#[from] gix::traverse::commit::simple::Error),
    #[error("Gitoxide error: {0}")]
    RevWalk(#[from] gix::revision::walk::Error),
    #[error("Gitoxide error: {0}")]
    Head(#[from] gix::reference::head_commit::Error),
    #[error("Gitoxide error: {0}")]
    Decode(#[from] gix_object::decode::Error),
    #[error("Gitoxide error: {0}")]
    RevWalkGraph(#[from] gix::object::find::existing::Error),
    #[error("Gitoxide error: {0}")]
    Commit(#[from] gix::object::commit::Error),
    #[error("Gitoxide error: {0}")]
    Rewrites(#[from] gix::diff::new_rewrites::Error),
    #[error("Gitoxide error: {0}")]
    ObjectPeel(#[from] gix::object::peel::to_kind::Error),
    #[error("Gitoxide error: {0}")]
    ObjectDiff(#[from] gix::object::tree::diff::for_each::Error),
    #[error("Gitoxide error: {0}")]
    FindExisting(#[from] gix::reference::find::existing::Error),
    #[error("Gitoxide error: {0}")]
    RemoteConnection(#[from] gix::remote::connect::Error),
    #[error("Gitoxide error: {0}")]
    PeelCommit(#[from] gix::head::peel::to_commit::Error),
    #[error("Gitoxide error: {0}")]
    RefMap(#[from] gix::remote::ref_map::Error),
    #[error("Gitoxide error: {0}")]
    PrepareFetch(#[from] gix::remote::fetch::prepare::Error),
    #[error("Gitoxide error: {0}")]
    Fetch(#[from] gix::remote::fetch::Error),
    #[error("Gitoxide error: {0}")]
    Revwalk(#[from] gix::revision::walk::iter::Error),
    #[error("Gitoxide error: {0}")]
    DiffOptionsInit(#[from] gix::diff::options::init::Error),
    #[error("Gitoxide error: {0}")]
    DateParse(#[from] gix::date::Error),
}

impl<T> From<T> for super::Error
where
    GitoxideError: From<T>,
{
    fn from(value: T) -> Self {
        Self::Gitoxide(Box::new(value.into()))
    }
}
