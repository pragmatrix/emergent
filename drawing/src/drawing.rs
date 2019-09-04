use crate::{
    DrawingBounds, DrawingFastBounds, DrawingTarget, LinkChain, MeasureText, Point, Ref, Shape,
    Vector,
};
use serde::{Deserialize, Serialize};

mod blend_mode;
pub use blend_mode::*;

mod clip;
pub use clip::*;

mod color;
pub use color::*;

pub mod font;
pub use font::Font;

pub mod paint;
pub use paint::Paint;

mod transform;
pub use transform::*;

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Drawing {
    /// A transform chain the commands refer to.
    transforms: LinkChain<Transform>,
    /// A clip chain that derives the clip graph that is used.
    clips: LinkChain<Clip>,

    commands: Vec<(Ref<Clip>, Ref<Transform>, Draw)>,

    /// Optional drawing bounds that resulted from a fast_bounds() invocation.
    #[serde(skip)]
    bounds: Option<DrawingBounds>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Draw {
    /// Fill the current clipping area with the given paint and blend mode.
    Paint(Paint, BlendMode),

    /// Draw a number of shapes with the same paint.
    Shapes(Vec<Shape>, Paint),

    /// Draw a nested drawing.
    Drawing(Drawing),
}

/*
impl<I: IntoIterator<Item = Draw>> From<I> for Drawing {
    fn from(v: I) -> Self {
        Drawing(v.into_iter().collect())
    }
}
*/

/*
// TODO: is this appropriate?
impl Deref for Drawing {
    type Target = Vec<Draw>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Drawing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

*/

impl Drawing {
    pub fn new() -> Drawing {
        Default::default()
    }

    /// Return the most recent Draw
    pub fn last_mut(&mut self) -> Option<&mut Draw> {
        self.commands.last_mut().map(|(_, _, d)| d)
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Push a draw with the recent transformation and clip.
    pub fn push(&mut self, draw: Draw) {
        let (clip, transform) = self.state();
        self.commands.push((clip, transform, draw));
    }

    /// Returns the current state.
    ///
    /// This is the most recent clip and transformation.
    pub fn state(&self) -> (Ref<Clip>, Ref<Transform>) {
        match self.commands.last() {
            Some((c, t, _)) => (*c, *t),
            None => (Ref::Identity, Ref::Identity),
        }
    }

    /*
    pub fn take(&mut self) -> Vec<Draw> {
        mem::replace(&mut self.0, Vec::new())
    }
    */

    pub fn stack_h(drawings: Vec<Drawing>, measure_text: &dyn MeasureText) -> Drawing {
        Self::stack(drawings, measure_text, Vector::new(1.0, 0.0))
    }

    pub fn stack_v(drawings: Vec<Drawing>, measure_text: &dyn MeasureText) -> Drawing {
        Self::stack(drawings, measure_text, Vector::new(0.0, 1.0))
    }

    /// Stack a number of drawings vertically based on fast_bounds() computation.
    ///
    /// The direction to stack the drawings is computed from the delta vector
    /// which is multiplied with the bounds before to compute the location of
    /// the next drawing.
    pub fn stack(
        drawings: Vec<Drawing>,
        measure_text: &dyn MeasureText,
        d: impl Into<Vector>,
    ) -> Drawing {
        let d = d.into();
        let mut p = Point::default();
        let mut r = Drawing::new();
        for drawing in drawings {
            match drawing.fast_bounds(measure_text) {
                DrawingBounds::Bounded(b) => {
                    let align = -b.point.to_vector();
                    let transform = Transform::Translate((p + align).to_vector());
                    r.transform(&transform, |d| d.draw_drawing(&drawing));
                    p += Vector::from(b.extent) * d
                }
                _ => {}
            }
        }
        r
    }
}
