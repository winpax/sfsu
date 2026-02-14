//! Git specific error helpers

use gix::{date, diff, head, object, open, reference, refspec, remote, revision, traverse};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub enum GixErrorInner {
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

#[derive(Debug, thiserror::Error)]
#[error("Gitoxide Error: {0}")]
pub struct GixError(Box<GixErrorInner>);

impl<T> From<T> for GixError
where
    GixErrorInner: From<T>,
{
    fn from(value: T) -> Self {
        Self(Box::new(GixErrorInner::from(value)))
    }
}

impl<T> From<T> for super::Error
where
    GixError: From<T>,
{
    fn from(value: T) -> Self {
        Self::Gitoxide(Box::new(value.into()))
    }
}
