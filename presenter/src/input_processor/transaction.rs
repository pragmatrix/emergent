//! A representation of a move event.
//!
//! Data is the initialization data. For example this can be a position or a position combined with a phase.
use std::fmt::Debug;

/// A transactional event that contains payloads for Begin / Update / Commit, but not for Rollback.
#[derive(Clone, Debug)]
pub enum Transaction<Data> {
    Begin(Data),
    Update(Data),
    Commit(Data),
    Rollback(Data),
}

impl<Data> Transaction<Data> {
    // TODO: use deref?
    pub fn data(&self) -> &Data {
        use Transaction::*;
        match self {
            Begin(d) | Update(d) | Commit(d) | Rollback(d) => d,
        }
    }

    pub fn map<D>(self, f: impl FnOnce(Data) -> D) -> Transaction<D> {
        use Transaction::*;
        match self {
            Begin(d) => Begin(f(d)),
            Update(d) => Update(f(d)),
            Commit(d) => Commit(f(d)),
            Rollback(d) => Rollback(f(d)),
        }
    }

    /// Returns true if the event indicates that the transaction continues to stay active, i.e. other transaction
    /// events belonging to the same transaction will follow.
    pub fn is_active(&self) -> bool {
        use Transaction::*;
        match self {
            Begin(_) | Update(_) => true,
            Commit(_) | Rollback(_) => false,
        }
    }
}
