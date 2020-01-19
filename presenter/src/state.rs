use emergent_presentation::Scope;
use std::any;
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;

pub type MemoPool = HashMap<any::TypeId, Box<dyn Any>>;

/// The state of an call scope.
pub struct ScopeState<Msg> {
    /// The components that are in this space.
    pub components: MemoPool,

    /// Nested scopes.
    pub nested: HashMap<Scope, ScopeState<Msg>>,

    // TODO: remove (also maybe Host and Context)
    pd: PhantomData<Msg>,
}

impl<Msg> ScopeState<Msg> {
    pub fn new() -> Self {
        Self {
            components: Default::default(),
            nested: Default::default(),
            pd: PhantomData,
        }
    }
}
