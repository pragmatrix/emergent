use downcast_rs::Downcast;
use emergent_ui::WindowMsg;
use std::any::Any;

/// A trait to define gesture recognizers.
///
/// Gesture recognizers are persisting in the presentation and are updated with
/// each WindowMsg. Their lifetime is bound to the scope they are rendered at first.
///
/// This is the first attempt to introduce presentation persisting state.
///
/// Ultimately, this should generalize into a component system covering all view state, that
/// can be specified in the rendering process, must persist and update between presentation runs.

pub trait GestureRecognizer {
    type Msg: 'static;
    fn update(&mut self, msg: WindowMsg) -> Option<Self::Msg>;
}

pub trait UntypedGestureRecognizer {
    fn update(&mut self, msg: WindowMsg) -> Option<Box<dyn Any + 'static>>;
}

impl<T, Msg: 'static> UntypedGestureRecognizer for T
where
    T: GestureRecognizer<Msg = Msg> + 'static,
{
    fn update(&mut self, msg: WindowMsg) -> Option<Box<dyn Any + 'static>> {
        let cmd = GestureRecognizer::update(self, msg);
        cmd.map(|msg| Box::new(msg).into_any())
    }
}
