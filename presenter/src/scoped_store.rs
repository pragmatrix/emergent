use crate::{ContextPath, ContextScope};
use emergent_drawing::ReplaceWith;
use std::any;
use std::any::{Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Deref;

pub type ScopedState = (ContextPath, Box<dyn Any>);
pub type TypedStore = HashMap<TypeId, Box<dyn Any>>;

/// The state of an call scope.
///
/// TODO: can we flatten this somehow? This depends largely on the use cases of the context.
#[derive(Debug)]
pub struct ScopedStore {
    /// The states that is stored in this scope.
    pub(crate) states: TypedStore,

    /// Nested scoped stores.
    nested: HashMap<ContextScope, ScopedStore>,
}

impl Default for ScopedStore {
    fn default() -> Self {
        Self {
            states: Default::default(),
            nested: Default::default(),
        }
    }
}

impl ScopedStore {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_values(values: impl IntoIterator<Item = ScopedState>) -> Self {
        // TODO: can we optimize this here by pre-sorting entries?
        values
            .into_iter()
            .fold(ScopedStore::new(), |mut s, (p, state)| {
                s.at_mut_or_new(&p)
                    .states
                    .insert(state.deref().type_id(), state);
                s
            })
    }

    /// Remove all the local state at the path.
    pub fn remove_states_at(&mut self, path: &[ContextScope]) -> Vec<Box<dyn Any>> {
        match self.at_mut(path) {
            None => Vec::new(),
            Some(store) => store.states.drain().map(|(_, s)| s).collect(),
        }
    }

    /// Extend all the local states at the path.
    pub fn extend_states_at(&mut self, path: &[ContextScope], states: Vec<Box<dyn Any>>) {
        if states.is_empty() {
            return;
        }
        self.at_mut_or_new(path)
            .states
            .extend(states.into_iter().map(|s| (s.deref().type_id(), s)));
    }

    pub fn at_mut_or_new(&mut self, path: &[ContextScope]) -> &mut ScopedStore {
        if path.is_empty() {
            return self;
        }

        // TODO: that clone!!
        match self.nested.entry(path[0].clone()) {
            Entry::Occupied(e) => e.into_mut().at_mut_or_new(&path[1..]),
            Entry::Vacant(e) => e.insert(ScopedStore::new()),
        }
    }

    pub fn at_mut(&mut self, path: &[ContextScope]) -> Option<&mut ScopedStore> {
        if path.is_empty() {
            return Some(self);
        }

        // TODO: this clone here!
        match self.nested.entry(path[0].clone()) {
            Entry::Occupied(e) => e.into_mut().at_mut(&path[1..]),
            Entry::Vacant(_) => None,
        }
    }

    /// Remove a nested scope.
    pub fn remove_scope(&mut self, scope: impl Into<ContextScope>) -> Option<ScopedStore> {
        let scope = scope.into();
        self.nested.remove(&scope)
    }

    /// Remove a typed state.
    pub fn remove_state<S: 'static>(&mut self) -> Option<S> {
        let type_id: TypeId = TypeId::of::<S>();
        self.states
            .remove(&type_id)
            .map(|a| *(a.downcast::<S>().unwrap()))
    }

    /// Merges two scoped states, overwrites the states on the left side.
    pub fn merged(mut self, other: Self) -> Self {
        self.states.extend(other.states);
        Self {
            states: self.states,
            nested: Self::merge_nested(self.nested, other.nested),
        }
    }

    fn merge_nested(
        mut left: HashMap<ContextScope, ScopedStore>,
        right: HashMap<ContextScope, ScopedStore>,
    ) -> HashMap<ContextScope, ScopedStore> {
        right
            .into_iter()
            .for_each(|(scope, right)| match left.entry(scope) {
                Entry::Occupied(x) => x.into_mut().replace_with(|l| l.merged(right)),
                Entry::Vacant(v) => {
                    v.insert(right);
                }
            });

        left
    }
}
