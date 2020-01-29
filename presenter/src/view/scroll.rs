use crate::recognizer::MoverRecognizer;
use crate::{Context, View};
use emergent_drawing::{Transformed, Vector};

#[derive(Clone)]
struct State {
    /// The transformation vector of the content.
    content_transform: Vector,
}

// Experiment: create a scroll view around a content view.
/// TODO: this must be somehow be lazy, and perhaps something that can be bound to the elements that generate the content views?
pub fn view<Msg: 'static>(
    mut context: Context,
    build_content: impl FnOnce(Context) -> View<Msg>,
) -> View<Msg> {
    let view = context
        .with_state(
            || {
                info!("scrollview: resetting state");
                State {
                    content_transform: Vector::new(0.0, 0.0),
                }
            },
            |ctx, s| {
                info!("scrollview at: {:?}", s.content_transform);
                build_content(ctx).transformed(s.content_transform)
            },
        )
        .in_area();

    context.attach_recognizer(view, || {
        info!("creating new recognizer");
        MoverRecognizer::new(|state: &mut State| &mut state.content_transform)
    })
}
