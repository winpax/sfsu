//! Git specific error helpers

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// A collection of gitoxide errors
#[error("Gitoxide error: {0}")]
pub enum GitoxideError {
    Open(#[from] gix::open::Error),
    Traverse(#[from] gix::traverse::commit::simple::Error),
    RevWalk(#[from] gix::revision::walk::Error),
    Head(#[from] gix::reference::head_commit::Error),
    Decode(#[from] gix_object::decode::Error),
    RevWalkGraph(#[from] gix::object::find::existing::Error),
    Commit(#[from] gix::object::commit::Error),
    Rewrites(#[from] gix::diff::new_rewrites::Error),
    ObjectPeel(#[from] gix::object::peel::to_kind::Error),
    ObjectDiff(#[from] gix::object::tree::diff::for_each::Error),
    FindExisting(#[from] gix::reference::find::existing::Error),
    FindRemote(#[from] gix::remote::find::existing::Error),
    RemoteConnection(#[from] gix::remote::connect::Error),
    PeelCommit(#[from] gix::head::peel::to_commit::Error),
    RefMap(#[from] gix::remote::ref_map::Error),
    PrepareFetch(#[from] gix::remote::fetch::prepare::Error),
    Fetch(#[from] gix::remote::fetch::Error),
    Revwalk(#[from] gix::revision::walk::iter::Error),
    DiffOptionsInit(#[from] gix::diff::options::init::Error),
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
