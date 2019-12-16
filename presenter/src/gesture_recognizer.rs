use emergent_ui::WindowMsg;

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
    fn update(&mut self, msg: WindowMsg);
}
