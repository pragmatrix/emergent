use crate::length;
use crate::Bound;

/// Area constraints.
pub type Area = [Linear; 2];

/// Volume constraint.
pub type Volume = [Linear; 3];

/// Linear constraint.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Linear {
    min: length,
    /// The additional length preferred added to min.
    preferred: length,
    /// The additional length the defines the maximum added to min + preferred.
    /// If not set, there is no maximum size.
    max: Max,
}

impl Linear {
    /// A linear constraint that just specifies a minimum size.
    pub fn min(min: length) -> Linear {
        Linear {
            min,
            preferred: 0.0.into(),
            max: Max::Infinite,
        }
    }

    /// A fixed size constraint.
    /// TODO: may rename to tight?
    pub fn fixed(value: length) -> Linear {
        Linear {
            min: value,
            preferred: 0.0.into(),
            max: Max::Finite(0.0.into()),
        }
    }

    /// The effective preferred size.
    ///
    /// Equals to min + preferred.
    pub fn effective_preferred(&self) -> length {
        self.min + self.preferred
    }

    /// The effective maximum size.
    pub fn effective_max(&self) -> Max {
        self.max.map(|m| self.min + self.preferred + m)
    }

    /// Combine the constraints of one axis to create a constraint that represents
    /// the elements of that axis layouted one after each other.
    pub fn directional(a: &[Linear]) -> Linear {
        match a.len() {
            0 => panic!("internal error: zero directional constraints"),
            1 => *a.first().unwrap(),
            _ => a[1..].iter().fold(a[0], |a, b| Linear {
                min: a.min + b.min,
                preferred: a.preferred + b.preferred,
                max: match (a.max, b.max) {
                    (Max::Finite(_), Max::Infinite) => Max::Infinite,
                    (Max::Infinite, Max::Finite(_)) => Max::Infinite,
                    (Max::Finite(a), Max::Finite(b)) => Max::Finite(a + b),
                    (Max::Infinite, Max::Infinite) => Max::Infinite,
                },
            }),
        }
    }

    pub fn orthogonal(a: &[Linear]) -> Linear {
        match a.len() {
            0 => panic!("internal error: zero orthogonal constraints"),
            1 => *a.first().unwrap(),
            _ => a[1..].iter().fold(a[0], |a, b| Linear {
                min: a.min.max(b.min),
                // try to give every element the preferred size, so we
                // use max here and not average.
                preferred: a.preferred.max(b.preferred),
                max: match (a.max, b.max) {
                    (Max::Finite(a), Max::Infinite) => Max::Finite(a),
                    (Max::Infinite, Max::Finite(b)) => Max::Finite(b),
                    // note: the maximum of an element can be always exceeded
                    // when the element gets sized, which means that is must be
                    // aligned inside its box, which the element decides how.
                    (Max::Finite(a), Max::Finite(b)) => Max::Finite(a.max(b)),
                    (Max::Infinite, Max::Infinite) => Max::Infinite,
                },
            }),
        }
    }

    /// Layouts the linear constraint.
    ///
    /// For an unbounded Bound, this uses the preferred size.
    ///
    /// For an bounded Bound, this _always_ returns the bound's finite size.
    ///
    /// The rationale behind that is that the Bound is considered unchangeable,
    /// meaning the element _must_ fit into, even if it gets overconstrained,
    /// and this leaves the element a final say in the positioning, for examnple, it
    /// might decide to comply to it by sizing itself below its minimum size, or
    /// it might show only a part of itself.
    pub fn layout(&self, bound: Bound) -> length {
        match bound {
            Bound::Unbounded => self.effective_preferred(),
            Bound::Bounded(length) => length,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Max {
    Finite(length),
    Infinite,
}

impl Max {
    pub fn map<F>(&self, f: F) -> Max
    where
        F: Fn(length) -> length,
    {
        match *self {
            Max::Finite(v) => Max::Finite(f(v)),
            Max::Infinite => Max::Infinite,
        }
    }
}
