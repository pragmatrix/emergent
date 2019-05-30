use crate::fps;

/// Two-dimensional constraints.
pub type Rect = [Dim; 2];

/// One-dimensional hard constraints.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Dim {
    min: fps,
    // The additional size element prefers,
    // if 0, min _must_ be the size of the element, if
    // set min+range is the preferred _and_ the maximum size.
    // If not set, everything >= min is tolerated.
    range: Option<fps>,
}

impl Dim {
    pub fn identity() -> Dim {
        Dim {
            min: 0.0.into(),
            range: None,
        }
    }

    /// A constraint that just specifies a minimum size.
    pub fn min(min: fps) -> Dim {
        Dim { min, range: None }
    }

    /// Combine the constraints of one axis to create a combined constraint for all
    /// the elements of that axis.
    pub fn combine(a: &[Dim]) -> Dim {
        a.iter().fold(Self::identity(), |a, b| Dim {
            min: a.min + b.min,
            range: match (a.range, b.range) {
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (Some(a), Some(b)) => Some(a + b),
                (None, None) => None,
            },
        })
    }
}
