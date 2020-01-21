use crate::{Context, View};

struct State {}

// Experiment: create a scroll view around a content view.
/// TODO: this must be somehow be lazy, and perhaps something that that can be bound to the elements that generate the content views?
pub fn view<Msg>(context: &mut Context, content: View<Msg>) -> View<Msg> {
    // get state, do we need state to be mutable, or not?

    context.with_state(|| State {}, |s: State| s);

    content
}
