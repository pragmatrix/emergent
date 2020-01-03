//! A data model for incrementally updated caches.

pub struct ChangeSet<E> {
    updated: Vec<(usize, (E, E))>,
    tail: TailChange<E>,
}

impl<E> ChangeSet<E> {
    pub(crate) fn new(updated: Vec<(usize, (E, E))>, tail: TailChange<E>) -> Self {
        Self { updated, tail }
    }

    /// Length of the set when all the changes are applied.
    pub fn len_applied(&self) -> usize {
        self.tail.end_applied()
    }

    pub fn updated(&self) -> impl Iterator<Item = &(usize, (E, E))> {
        self.updated.iter()
    }

    pub fn updated_indices<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        self.updated().map(|u| u.0)
    }
}

pub(crate) enum TailChange<E> {
    Added(usize, Vec<E>),
    Removed(usize, Vec<E>),
}

impl<E> TailChange<E> {
    pub fn end_applied(&self) -> usize {
        match *self {
            TailChange::Added(start, ref v) => start + v.len(),
            TailChange::Removed(start, _) => start,
        }
    }

    pub fn start(&self) -> usize {
        match self {
            TailChange::Added(start, _) => *start,
            TailChange::Removed(start, _) => *start,
        }
    }
}
