use crate::{Context, View};
use emergent_drawing::{Transformed, Vector};

struct State {
    /// The transformation vector of the content.
    content_transform: Vector,
}

// Experiment: create a scroll view around a content view.
/// TODO: this must be somehow be lazy, and perhaps something that that can be bound to the elements that generate the content views?
pub fn view<Msg>(context: Context, content: impl FnOnce(Context) -> View<Msg>) -> View<Msg> {
    // get state, do we need state to be mutable, or not?

    context.with_state(
        || State {
            content_transform: Vector::new(100.0, 100.0),
        },
        |ctx, s: State| {
            let content = content(ctx).transformed(s.content_transform);
            (s, content)
        },
    )
}
