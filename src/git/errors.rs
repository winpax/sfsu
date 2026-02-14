//! Git specific error helpers

use gix::{date, diff, head, object, open, reference, refspec, remote, revision, traverse};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// A collection of gitoxide errors
#[error("Gitoxide error: {0}")]
pub enum GitoxideError {
    Open(#[from] open::Error),
    Traverse(#[from] traverse::commit::simple::Error),
    RevWalk(#[from] revision::walk::Error),
    Head(#[from] reference::head_commit::Error),
    Decode(#[from] gix_object::decode::Error),
    RevWalkGraph(#[from] object::find::existing::Error),
    Commit(#[from] object::commit::Error),
    Rewrites(#[from] diff::new_rewrites::Error),
    ObjectPeel(#[from] object::peel::to_kind::Error),
    ObjectDiff(#[from] object::tree::diff::for_each::Error),
    FindExisting(#[from] reference::find::existing::Error),
    RemoteConnection(#[from] remote::connect::Error),
    PeelCommit(#[from] head::peel::to_commit::Error),
    Fetch(#[from] remote::fetch::Error),
    Revwalk(#[from] revision::walk::iter::Error),
    DiffOptionsInit(#[from] diff::options::init::Error),
    DateParse(#[from] date::Error),
    RefspecParse(#[from] refspec::parse::Error),
    RefMap(#[from] remote::ref_map::Error),
    PrepareFetch(#[from] remote::fetch::prepare::Error),
    FindRemote(#[from] remote::find::existing::Error),
}

impl<T> From<T> for super::Error
where
    GitoxideError: From<T>,
{
    fn from(value: T) -> Self {
        Self::Gitoxide(Box::new(value.into()))
    }
}
