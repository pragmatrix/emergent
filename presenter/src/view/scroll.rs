use crate::recognizer::mover::MoveTransaction;
use crate::recognizer::{animator, easing, Animator, MoverRecognizer, Subscription};
use crate::{Context, InputProcessor, View};
use emergent_drawing::{scalar, DrawingFastBounds, Rect, Transformed, Vector};
use std::ops::Deref;
use std::time::Duration;

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
    let create_content = |context: Context, state: &State| {
        trace!("scrollview at: {:?}", state.content_transform);
        // TODO, we must consume the context, but then we need to get support and view_bounds out of it, this is ugly.
        let support = context.support();
        let view_bounds = context.view_bounds();
        trace!("view_bounds: {:?}", view_bounds);

        let view = build_content(context);

        let content_bounds = view.fast_bounds(support.deref());
        let (alignment_transform, transform) = match content_bounds.as_bounds() {
            Some(content_bounds) => {
                let content_bounds = content_bounds.to_rect();
                trace!("content_bounds: {:?}", content_bounds);

                let aligned_bounds = align_in_container(
                    &content_bounds,
                    (Alignment::Center, Alignment::Begin),
                    &view_bounds,
                );
                let alignment_transform = aligned_bounds.left_top() - content_bounds.left_top();
                trace!("alignment_transform: {:?}", alignment_transform);

                let preferred_bounds = aligned_bounds + state.content_transform;
                trace!("preferred_bounds: {:?}", preferred_bounds);

                let constrained_bounds = constrain_in_container(&preferred_bounds, &view_bounds);
                trace!("perfect_place: {:?}", constrained_bounds);

                let resistance = (preferred_bounds.center().to_vector()
                    - constrained_bounds.center().to_vector())
                    / 2.0;

                (
                    alignment_transform,
                    alignment_transform + state.content_transform - resistance,
                )
            }
            None => Default::default(),
        };
        trace!("final_transform: {:?}", transform);

        (alignment_transform, view.transformed(transform))
    };

    let (alignment_transform, view) = context.with_state_r(
        || {
            info!("scrollview: resetting state");

            State {
                content_transform: Vector::new(0.0, 0.0),
            }
        },
        create_content,
    );

    let mut view = view.in_area();
    let mover = view.attach_recognizer(&mut context, || {
        info!("creating new recognizer");
        MoverRecognizer::new(|state: &State, _| {
            Some(Mover {
                start: state.content_transform,
            })
        })
    });

    // TODO: support Deref to be able to access `is_active()` on `mover`?
    if !mover.recognizer.is_active() {
        let state: &State = view.get_state().unwrap();
        let initial = state.content_transform;
        if state.content_transform != alignment_transform {
            view.attach_recognizer(&mut context, || {
                Animator::new(Duration::from_millis(200), easing::ease_out_cubic).apply_mut(
                    move |e: animator::Event, s: &mut State| {
                        s.content_transform = e.interpolate(&initial, &alignment_transform);
                        None
                    },
                )
            })
            // TODO: subscribe should be able to be applied directly on the return recognizer.
            // TODO: subscription of ticks should be implicitly done by the recognizer.
            .subscriptions
            .subscribe(Subscription::Ticks);
        }
    }

    view
}

struct Mover {
    start: Vector,
}

impl<Msg> MoveTransaction<Msg> for Mover {
    type State = State;

    fn update(&mut self, pos: Vector, s: &mut Self::State) -> Option<Msg> {
        s.content_transform = self.start + pos;
        None
    }

    fn commit(&mut self, pos: Vector, s: &mut Self::State) -> Option<Msg> {
        s.content_transform = self.start + pos;
        None
    }

    fn rollback(&mut self, s: &mut Self::State) -> Option<Msg> {
        s.content_transform = self.start;
        None
    }
}

fn align_in_container(to_center: &Rect, align: (Alignment, Alignment), container: &Rect) -> Rect {
    align_rect(to_center, align, container)
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Alignment {
    Begin,
    Center,
    End,
}

fn align_span(
    (begin, end): (scalar, scalar),
    alignment: Alignment,
    (cb, ce): (scalar, scalar),
) -> (scalar, scalar) {
    let length = end - begin;
    match alignment {
        Alignment::Begin => (cb, cb + length),
        Alignment::Center => {
            let begin = (cb + ce - length) / 2.0;
            (begin, begin + length)
        }
        Alignment::End => (ce - length, ce),
    }
}

fn align_rect(r: &Rect, align: (Alignment, Alignment), container: &Rect) -> Rect {
    let (h, v) = align;
    let h = align_span(r.h_span(), h, container.h_span());
    let v = align_span(r.v_span(), v, container.v_span());
    Rect::from_spans(h, v)
}

trait RectSpans {
    fn h_span(&self) -> (scalar, scalar);
    fn v_span(&self) -> (scalar, scalar);
    fn from_spans(h: (scalar, scalar), v: (scalar, scalar)) -> Self;
}

impl RectSpans for Rect {
    fn h_span(&self) -> (scalar, scalar) {
        (self.left, self.right)
    }

    fn v_span(&self) -> (scalar, scalar) {
        (self.top, self.bottom)
    }

    fn from_spans((left, right): (scalar, scalar), (top, bottom): (scalar, scalar)) -> Self {
        Rect::new((left, top).into(), (right, bottom).into())
    }
}

fn constrain_in_container(preferred: &Rect, bounds: &Rect) -> Rect {
    let h = constrain(preferred.h_span(), bounds.h_span());
    let v = constrain(preferred.v_span(), bounds.v_span());
    Rect::from_spans(h, v)
}

fn constrain((pb, pe): (scalar, scalar), (bb, be): (scalar, scalar)) -> (scalar, scalar) {
    let pw = pe - pb;
    let bw = be - bb;
    let b = {
        if pw <= bw {
            bb + (bw - pw) / 2.0
        } else {
            if pb > bb {
                bb
            } else if pe < be {
                be - pw
            } else {
                pb
            }
        }
    };

    (b, b + pw)
}
