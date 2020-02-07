//! A representation of a move event.
//!
//! Data is the initialization data. For example this can be a position or a position combined with a phase.
use emergent_drawing::{Point, Vector};
use std::fmt::Debug;

/// A transactional event that contains payloads for Begin / Update / Commit, but not for Rollback.
#[derive(Clone, Debug)]
pub enum Transaction<Data> {
    Begin(Data),
    Update(Data, Vector),
    Commit(Data, Vector),
    Rollback(Data),
}

impl<Data> Transaction<Data> {
    pub fn v(&self) -> Vector {
        let (_, v) = self.state();
        v
    }

    pub fn data(&self) -> &Data {
        let (d, _) = self.state();
        d
    }

    pub fn state(&self) -> (&Data, Vector) {
        match self {
            Transaction::Begin(d) => (d, Vector::ZERO),
            Transaction::Update(d, v) => (d, *v),
            Transaction::Commit(d, v) => (d, *v),
            Transaction::Rollback(d) => (d, Vector::ZERO),
        }
    }

    pub fn map_data<R>(self, f: impl FnOnce(Data) -> R) -> Transaction<R> {
        use Transaction::*;
        match self {
            Begin(d) => Begin(f(d)),
            Update(d, v) => Update(f(d), v),
            Commit(d, v) => Commit(f(d), v),
            Rollback(d) => Rollback(f(d)),
        }
    }

    /// Returns true if the event indicates that the transaction continues to stay active, i.e. other transaction
    /// events belonging to the same transaction will follow.
    pub fn is_active(&self) -> bool {
        use Transaction::*;
        match self {
            Begin(_) => true,
            Update(_, _) => true,
            Commit(_, _) => false,
            Rollback(_) => false,
        }
    }

    pub fn map_v(self, f: impl FnOnce(Vector) -> Vector) -> Self {
        use Transaction::*;
        match self {
            Begin(d) => Begin(d),
            Update(d, v) => Update(d, f(v)),
            Commit(d, v) => Commit(d, f(v)),
            Rollback(d) => Rollback(d),
        }
    }
}

/// Can the transaction event described as having an absolute position?
pub trait AbsolutePos: Debug {
    fn absolute_pos(&self) -> Point;
}

impl AbsolutePos for Transaction<Point> {
    fn absolute_pos(&self) -> Point {
        *self.data() + self.v()
    }
}

impl AbsolutePos for Transaction<Vector> {
    fn absolute_pos(&self) -> Point {
        Point::from(*self.data() + self.v())
    }
}
