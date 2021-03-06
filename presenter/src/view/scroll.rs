use crate::input_processor::WithResistance;
use crate::input_processor::{animator, ConvergeTo};
use crate::input_processor::{easing, Animator};
use crate::input_processor::{Pan, PreserveMomentum};
use crate::{InputProcessor, View, ViewBuilder};
use emergent_drawing::{scalar, Point, Rect, Vector};
use std::time::Duration;

#[derive(Clone)]
struct State {
    /// The transformation vector of the content.
    content_transform: Vector,
    /// Current movement?
    movement_active: bool,
}

#[derive(Copy, Clone, Debug)]
struct ConstrainedContentTransform(Vector);

// Experiment: create a scroll view around a content view.
/// TODO: this must be somehow be lazy, and perhaps something that can be bound to the elements that generate the content views?
pub fn view<Msg: 'static>(
    mut builder: ViewBuilder<Msg>,
    build_content: impl FnOnce(ViewBuilder<Msg>) -> View<Msg>,
) -> View<Msg> {
    // TODO, we must consume the context, but then we need to get support and view_bounds out of it, this is ugly.
    let container_bounds = builder.view_bounds();
    trace!("view_bounds: {:?}", container_bounds);

    let view = builder.scoped(0, build_content);

    let content_bounds = view.fast_bounds(&builder);

    let state = builder.use_state(|| {
        info!("scrollview: resetting state");
        State {
            content_transform: Vector::new(0.0, 0.0),
            movement_active: false,
        }
    });

    trace!("scrollview at: {:?}", state.content_transform);

    let (constrained_content_transform, transform) = match content_bounds.as_bounds() {
        Some(content_bounds) => {
            let content_bounds = content_bounds.to_rect();
            trace!("content_bounds: {:?}", content_bounds);

            let aligned_bounds = align_in_container(
                &content_bounds,
                (Alignment::Center, Alignment::Begin),
                &container_bounds,
            );
            let alignment_transform = aligned_bounds.left_top() - content_bounds.left_top();
            trace!("alignment_transform: {:?}", alignment_transform);

            let preferred_bounds = aligned_bounds + state.content_transform;
            trace!("preferred_bounds: {:?}", preferred_bounds);

            let constrained_bounds = constrain_in_container(&preferred_bounds, &container_bounds);
            trace!("perfect_place: {:?}", constrained_bounds);

            let constrained_content_transform =
                state.content_transform + (constrained_bounds.center() - preferred_bounds.center());

            (
                constrained_content_transform,
                alignment_transform + state.content_transform,
            )
        }
        None => Default::default(),
    };
    trace!("final_transform: {:?}", transform);

    let view = view.transformed(transform).in_area();

    builder.set_state(ConstrainedContentTransform(constrained_content_transform));
    builder.use_input_processor(|| {
        info!("creating new processor");
        let drift_duration = Duration::from_millis(500);
        Pan::new()
            .map_begin(|p: Point, state: &State| {
                let d = state.content_transform - p.to_vector();
                Some(move |p: Point| p + d)
            })
            // TODO: this can be just a map_with_state()?
            .with_resistance(|p, constrained: &ConstrainedContentTransform| {
                ((p + constrained.0).to_vector() / 2.0) - p.to_vector()
            })
            .preserve_momentum(100.0, easing::ease_out_cubic, drift_duration)
            // TODO: can we split up phases here, and implement converge_to with map_with_state() then?
            .converge_to(
                |constrained: &ConstrainedContentTransform| Point::from(constrained.0),
                drift_duration,
                easing::ease_out_cubic,
            )
            // TODO: insert a map to remove the phase before apply?
            .apply(|e, s: &mut State| {
                let (p, _) = e.data();
                s.content_transform = p.to_vector();
                s.movement_active = e.is_active();
                None
            })
    });

    // the bounce-back logic, which is inelegant, but I found no better solution yet.
    // There are two situations that need to trigger it:
    // - When a move ended without a drifting phase (without activating momentum preserving mode).
    // - When the content was changed _and_ the previously installed input processor is not active.

    // TODO: support Deref to be able to access `is_active()` on `mover`?
    let state: &State = builder.get_state().unwrap();
    if !state.movement_active && state.content_transform != constrained_content_transform {
        let initial = state.content_transform;
        builder.use_input_processor(|| {
            Animator::new(Duration::from_millis(200), easing::ease_out_cubic).apply(
                move |e: animator::Event, s: &mut State| {
                    s.content_transform = e.interpolate(&initial, &constrained_content_transform);
                    None
                },
            )
        })
    }

    builder.wrapped(view)
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
