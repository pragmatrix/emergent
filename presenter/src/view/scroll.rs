use crate::{Context, View};
use emergent_drawing::{Transformed, Vector};

struct State {
    /// The transformation vector of the content.
    content_transform: Vector,
}

// Experiment: create a scroll view around a content view.
/// TODO: this must be somehow be lazy, and perhaps something that that can be bound to the elements that generate the content views?
pub fn view<Msg>(context: &mut Context, content: View<Msg>) -> View<Msg> {
    // get state, do we need state to be mutable, or not?

    context.with_state(
        || State {
            content_transform: Vector::new(100.0, 100.0),
        },
        |s: State| {
            let content = content.transformed(s.content_transform);
            (s, content)
        },
    )
}
