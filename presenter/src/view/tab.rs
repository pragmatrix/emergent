//! A view that presents data in multiple areas / tabs

use crate::{Direction, ScopedView, SimpleLayout, View, ViewBuilder, ViewReducer};
use emergent_drawing::{DrawingFastBounds, Transformed, Vector};

struct State {
    focused_index: usize,
    nested_transform: Vector,
}

pub fn view<Msg: 'static>(
    mut builder: ViewBuilder<Msg>,
    build_content: impl FnOnce(&mut ViewBuilder<Msg>) -> Vec<ScopedView<Msg>>,
) -> View<Msg> {
    let views = build_content(&mut builder);
    assert!(!views.is_empty());
    let state = builder.use_state(|| State {
        focused_index: 0,
        nested_transform: Vector::default(),
    });
    let focused = state.focused_index.min(views.len() - 1);
    let bounds = Direction::Row.layout_bounds(views.iter().map(|v| v.fast_bounds(&builder)));
    let focused_bounds = bounds[focused].as_bounds();
    let view = Direction::Row.reduce_immediate(builder, views);
    match focused_bounds {
        Some(bounds) => view.transformed(bounds.point.to_vector()),
        None => view,
    }
}
