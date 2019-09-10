//! Gesture handler specification, Event generation, and serialization.
use emergent_drawing::Point;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

/// A gesture.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub enum Gesture {
    /// A single tap, either a touch or a mouse button click.
    Tap(PointEvent),
}

impl Gesture {
    pub fn tap<Msg: Serialize>(f: impl FnOnce(Point) -> Msg) -> Self {
        Gesture::Tap(PointEvent::from_fn(f))
    }
}

/// A serialized event with placeholders for Point x / y coordinates.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct PointEvent(String);

impl PointEvent {
    pub fn from_fn<Msg: Serialize>(f: impl FnOnce(Point) -> Msg) -> Self {
        let msg = f(Self::PLACEHOLDER);
        let ron = ron::ser::to_string(&msg)
            .unwrap()
            .replace(&Self::placeholders().0, "{x}")
            .replace(&Self::placeholders().1, "{y}");
        PointEvent(ron)
    }

    pub const PLACEHOLDER: Point = Point::new(21624837.0, 39048042.0);

    fn placeholders() -> &'static (String, String) {
        PLACEHOLDERS.get_or_init(|| {
            (
                Self::PLACEHOLDER.x.to_string(),
                Self::PLACEHOLDER.y.to_string(),
            )
        })
    }
}

static PLACEHOLDERS: OnceCell<(String, String)> = OnceCell::new();

#[cfg(test)]
mod tests {
    use crate::PointEvent;
    use emergent_drawing::{scalar, Point};
    use serde::Serialize;

    #[derive(Serialize)]
    enum Msg {
        Clicked(Point),
        Clicked2 { p: Point },
        ClickedTuple((scalar, scalar)),
        ClickedInt(i32, i32),
    }

    #[test]
    fn test_point_event() {
        assert_eq!(
            PointEvent::from_fn(|p| Msg::Clicked(p)).0,
            "Clicked(({x},{y},))"
        );

        assert_eq!(
            PointEvent::from_fn(|p| Msg::Clicked2 { p }).0,
            "Clicked2(p:({x},{y},),)"
        );

        assert_eq!(
            PointEvent::from_fn(|p| Msg::ClickedTuple((p.x, p.y))).0,
            "ClickedTuple(({x},{y},))"
        );

        assert_eq!(
            PointEvent::from_fn(|p| Msg::ClickedInt(p.x as _, p.y as _)).0,
            "ClickedInt({x},{y},)"
        );
    }
}
