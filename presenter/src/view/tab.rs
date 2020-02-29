//! A view that presents data in multiple areas / tabs

use crate::{Context, Direction, IndexAccessible, SimpleLayout, View, ViewReducer};
use emergent_drawing::{DrawingFastBounds, ReplaceWith, Transformed, Vector};
use std::{f32, mem};

struct State {
    focused_index: usize,
    nested_transform: Vector,
}

pub fn view<Msg: 'static>(
    mut context: Context,
    build_content: impl FnOnce(&mut Context) -> Vec<View<Msg>>,
) -> View<Msg> {
    // TODO: we should be able to define states separately from builders (returning a ContextWithState<State> like value).
    let view = context.with_state(
        || State {
            focused_index: 0,
            nested_transform: Vector::default(),
        },
        |mut c, s: &State| {
            let views = build_content(&mut c);
            assert!(!views.is_empty());
            let focused = s.focused_index.min(views.len() - 1);
            let bounds = Direction::Row.layout_bounds(views.iter().map(|v| v.fast_bounds(&c)));
            let focused_bounds = bounds[focused].as_bounds();
            let view = Direction::Row.reduce(c, views);
            match focused_bounds {
                Some(bounds) => view.transformed(bounds.point.to_vector()),
                None => view,
            }
        },
    );

    view
}
