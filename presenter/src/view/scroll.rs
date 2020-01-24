use crate::recognizer::pan;
use crate::recognizer::PanRecognizer;
use crate::{Context, GestureRecognizer, View};
use emergent_drawing::{Transformed, Vector};

struct State {
    /// The transformation vector of the content.
    content_transform: Vector,
}

// Experiment: create a scroll view around a content view.
/// TODO: this must be somehow be lazy, and perhaps something that that can be bound to the elements that generate the content views?
pub fn view<Msg: 'static>(
    context: Context,
    build_content: impl FnOnce(Context) -> View<Msg>,
) -> View<Msg> {
    let view = context.with_state(
        || {
            info!("scrollview: resetting state");
            State {
                content_transform: Vector::new(100.0, 100.0),
            }
        },
        |ctx, s| {
            info!("scrollivew at: {:?}", s.content_transform);
            build_content(ctx).transformed(s.content_transform)
        },
    );

    view.in_area()
        .with_recognizer(PanRecognizer::new().apply(|mut s: State, e| {
            match e {
                pan::Event::Pressed(_) => {
                    info!("scrollview: pressed");
                    s.content_transform += Vector::new(10.0, 10.0);
                }
                pan::Event::Moved(_, v) => {
                    info!("scrollview: moved: {:?}", v);
                    s.content_transform += Vector::new(1.0, 1.0);
                }
                pan::Event::Released(_, _) => {}
            };

            (s, None)
        }))
}
