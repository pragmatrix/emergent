use crate::{ChangeSet, TailChange};

/// A data container that is ordered.
pub trait IndexedTarget<E> {
    /// Applies a new slice of elements to the ordered container and returns the change set.
    fn apply(&mut self, elements: &[E]) -> ChangeSet<E>
    where
        E: Clone + PartialEq;
}

/// An indexed sequence that can be read from.
pub trait IndexedSource<E> {
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// The length of the indexed sequence.
    fn len(&self) -> usize;
    /// Creates an iterator to the indexed sequence beginning at the index `start`.
    fn iter_at(&mut self, start: usize) -> Box<dyn Iterator<Item = &E> + '_>;
}

impl<E> IndexedSource<E> for Vec<E> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn iter_at(&mut self, start: usize) -> Box<dyn Iterator<Item = &E> + '_> {
        Box::new(self[start..].iter())
    }
}

impl<E> IndexedTarget<E> for Vec<E> {
    fn apply(&mut self, elements: &[E]) -> ChangeSet<E>
    where
        E: Clone + PartialEq,
    {
        let tail = {
            if elements.len() > self.len() {
                let start = self.len();
                self.extend_from_slice(&elements[start..]);
                TailChange::Added(start, self[start..self.len()].to_vec())
            } else {
                let start = elements.len();
                let removed = self.split_off(start);
                TailChange::Removed(start, removed)
            }
        };

        let mut updated = Vec::new();
        self[0..tail.start()]
            .iter_mut()
            .enumerate()
            .for_each(|(i, er)| {
                if *er != elements[i] {
                    updated.push((i, (er.clone(), elements[i].clone())));
                    *er = elements[i].clone();
                }
            });

        ChangeSet::new(updated, tail)
    }
}
