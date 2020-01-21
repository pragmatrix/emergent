use crate::{ContextPath, ContextScope};
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type ScopedValue = (ContextPath, Box<dyn Any>);

pub type StateStore = HashMap<TypeId, Box<dyn Any>>;

/// The state of an call scope.
///
/// TODO: can we flatten this somehow? This depends largely on the use cases of the context.
pub struct ScopedState {
    /// The state that is stored in this scope.
    store: StateStore,

    /// Nested scopes.
    nested: HashMap<ContextScope, ScopedState>,
}

impl ScopedState {
    pub fn new() -> Self {
        Self {
            store: Default::default(),
            nested: Default::default(),
        }
    }

    /// Remove a nested scope.
    pub fn remove_scope(&mut self, scope: impl Into<ContextScope>) -> Option<ScopedState> {
        let scope = scope.into();
        self.nested.remove(&scope)
    }

    /// Remove a typed state.
    pub fn remove_state<T: 'static>(&mut self) -> Option<T> {
        let type_id: TypeId = TypeId::of::<T>();
        self.store
            .remove(&type_id)
            .map(|a| *(a.downcast::<T>().unwrap()))
    }
}
