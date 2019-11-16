use crate::WindowMsg;
use emergent_presentation::Presentation;
use std::any::Any;
use std::mem;

pub trait ViewComponent<Msg>
where
    Self: AsAny + 'static,
    Msg: 'static,
{
    /// Process the window message.
    fn update(&mut self, msg: WindowMsg);

    /// Try to reconcile a new parameterization for that component.
    ///
    /// The default implementation overwrites the component with the newer one.
    /// Implement `reconcile` when it's necessary to preserve state over time.
    fn reconcile(&mut self, newer: Self)
    where
        Self: Sized,
    {
        mem::replace(self, newer);
    }

    /// Render a presentation.
    fn render(&self) -> Presentation;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// https://stackoverflow.com/questions/33687447/how-to-get-a-reference-to-a-concrete-type-from-a-trait-object

// TODO: if that works, try adding the functions to the ViewComponent<Msg> directly.

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
