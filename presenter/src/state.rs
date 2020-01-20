use emergent_presentation::Scope;
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type StateStore = HashMap<TypeId, Box<dyn Any>>;

/// The state of an call scope.
pub struct ScopeState {
    /// The state that is stored in this space.
    pub store: StateStore,

    /// Nested scopes.
    pub nested: HashMap<Scope, ScopeState>,
}

impl ScopeState {
    pub fn new() -> Self {
        Self {
            store: Default::default(),
            nested: Default::default(),
        }
    }
}
