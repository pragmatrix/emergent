use crate::{AsAny, ScopeFragment, ViewComponent, WindowMsg};
use emergent_presentation::Presentation;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;

// TODO: may change this to what's actually rendered (ComponentContext).

pub struct RenderContext<Msg>
where
    Msg: 'static,
{
    nested: HashMap<ScopeFragment, Node<Msg>>,
    current_index: usize,
}

impl<Msg> RenderContext<Msg> {
    /// Renders the component at the current index.
    pub fn render<C>(&mut self, component: C) -> ComponentContext<Msg>
    where
        C: ViewComponent<Msg>,
    {
        let node = self.reconcile_nested(self.current_index.into(), component);
        ComponentContext::new(node)
    }

    /// Reconciles a nested component with the given scope and returns its node.
    fn reconcile_nested<C>(&mut self, scope: ScopeFragment, new: C) -> &mut Node<Msg>
    where
        C: ViewComponent<Msg>,
    {
        match self.nested.entry(scope) {
            Entry::Occupied(mut e) => {
                match e.get_mut().component.as_any_mut().downcast_mut::<C>() {
                    Some(existing) => {
                        // same type, reconcile.
                        existing.reconcile(new);
                        e.into_mut()
                    }
                    None => {
                        // type is different, overwrite the component and clear the nested ones.
                        let node = e.into_mut();
                        node.component = Box::new(new);
                        node.nested.clear();
                        // TODO: can we avoid that additional downcast, after all we had
                        //       the concrete type before boxing.
                        node
                    }
                }
            }
            Entry::Vacant(e) => e.insert(Node::new(new)),
        }
    }

    pub fn resolve<C>(&mut self, f: impl FnOnce() -> C) -> &mut Node<Msg>
    where
        C: ViewComponent<Msg>,
    {
        match self.nested.entry(self.current_index.into()) {
            Entry::Occupied(mut e) => {
                match e.get_mut().component.as_any_mut().downcast_mut::<C>() {
                    Some(r) => e.into_mut(),
                    None => {
                        // type is different, overwrite the component and clear the nested ones.
                        let node = e.into_mut();
                        node.component = Box::new(f());
                        node.nested.clear();
                        // TODO: can we avoid that additional downcast, after all we had
                        //       the concrete type before boxing.
                        node
                    }
                }
            }
            Entry::Vacant(e) => e.insert(Node::new(f())),
        }
    }

    /*
    pub fn try_resolve<C>(&mut self) -> Option<&mut C>
    where
        C: 'static,
    {
        match self.nested.entry(self.current_index.into()) {
            Entry::Occupied(mut e) => {
                match e.get_mut().component.as_any_mut().downcast_mut() {
                    Some(r) => Some(r),
                    None => {
                        // type is different, must remove the whole tree.
                        e.remove();
                        None
                    }
                }
            }
            Entry::Vacant(e) => None,
        }
    }
    */
}

pub struct ComponentContext<'a, Msg>
where
    Msg: 'static,
{
    node: &'a mut Node<Msg>,
}

impl<'a, Msg> ComponentContext<'a, Msg> {
    pub fn new(node: &'a mut Node<Msg>) -> Self {
        Self { node }
    }

    pub fn with(&mut self, f: impl FnOnce(&mut RenderContext<Msg>)) {
        // swap out the map and create a new RenderContext.
        let map = mem::replace(&mut self.node.nested, HashMap::new());
        let mut nested_rc = RenderContext {
            nested: map,
            current_index: 0,
        };
        f(&mut nested_rc);
        // and swap it back in
        mem::swap(&mut self.node.nested, &mut nested_rc.nested);
    }
}

pub struct Node<Msg>
where
    Msg: 'static,
{
    pub component: Box<dyn ViewComponent<Msg>>,
    pub nested: HashMap<ScopeFragment, Node<Msg>>,
}

impl<Msg> Node<Msg> {
    pub fn new(component: impl ViewComponent<Msg>) -> Self {
        Self {
            component: Box::new(component),
            nested: HashMap::new(),
        }
    }
}

struct EmptyComp;

impl<Msg: 'static> ViewComponent<Msg> for EmptyComp {
    fn update(&mut self, msg: WindowMsg) {
        unimplemented!()
    }

    fn render(&self) -> Presentation<Msg> {
        unimplemented!()
    }
}
