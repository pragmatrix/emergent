use crate::{finite, length, span, Bound, Span};
use std::cmp::Ordering;

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
    /// The additional length the defines the maximum added to min + preferred, or Infinite
    /// when this element can be stretched to arbitrary lengths.
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
            max: Max::Length(0.0.into()),
        }
    }

    /// The effective preferred size.
    ///
    /// Equals to min + preferred.
    pub fn preferred_effective(&self) -> length {
        self.min + self.preferred
    }

    /// The effective maximum size.
    pub fn max_effective(&self) -> Max {
        self.max.map(|m| self.min + self.preferred + m)
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
            Bound::Unbounded => self.preferred_effective(),
            Bound::Bounded(length) => length,
        }
    }
}

pub trait Combine<T> {
    fn combine_directional(&self) -> T;
    fn combine_orthogonal(&self) -> T;
}

impl Combine<Linear> for [Linear] {
    /// Combine the constraints of one axis to create a constraint that represents
    /// the elements of that axis layouted one after each other.
    fn combine_directional(&self) -> Linear {
        match self.len() {
            0 => panic!("internal error: zero directional constraints"),
            1 => *self.first().unwrap(),
            _ => self[1..].iter().fold(self[0], |a, b| Linear {
                min: a.min + b.min,
                preferred: a.preferred + b.preferred,
                max: match (a.max, b.max) {
                    (Max::Length(_), Max::Infinite) => Max::Infinite,
                    (Max::Infinite, Max::Length(_)) => Max::Infinite,
                    (Max::Length(a), Max::Length(b)) => Max::Length(a + b),
                    (Max::Infinite, Max::Infinite) => Max::Infinite,
                },
            }),
        }
    }

    fn combine_orthogonal(&self) -> Linear {
        match self.len() {
            0 => panic!("internal error: zero orthogonal constraints"),
            1 => *self.first().unwrap(),
            _ => self[1..].iter().fold(self[0], |a, b| Linear {
                min: a.min.max(b.min),
                // try to give every element the preferred size, so we
                // use max here and not average.
                preferred: a.preferred.max(b.preferred),
                max: match (a.max, b.max) {
                    (Max::Length(a), Max::Infinite) => Max::Length(a),
                    (Max::Infinite, Max::Length(b)) => Max::Length(b),
                    // note: the maximum of an element can be always exceeded
                    // when the element gets sized, which means that is must be
                    // aligned inside its box, which the element decides how.
                    (Max::Length(a), Max::Length(b)) => Max::Length(a.max(b)),
                    (Max::Infinite, Max::Infinite) => Max::Infinite,
                },
            }),
        }
    }
}

pub trait Place<T> {
    fn place(&self, start: finite, bound: Bound) -> Vec<Span>;
}

impl Place<Linear> for [Linear] {
    fn place(&self, start: finite, bound: Bound) -> Vec<Span> {
        match bound {
            // bounded, use minimum sizes.
            Bound::Unbounded => self
                .iter()
                .scan(start, |cur, l| {
                    let c = *cur;
                    *cur = *cur + l.min;
                    Some(span(c, l.min))
                })
                .collect(),
            // TODO: support alignment.
            Bound::Bounded(length) => place_bounded(self, start, length, Alignment::Start),
        }
    }
}

pub trait Distribute<T> {
    fn distribute(&mut self, space: T, weights: &[T]);
}

impl Distribute<length> for [length] {
    /// Distribute space relatively defined by some weights.
    fn distribute(&mut self, space: length, weights: &[length]) {
        assert_eq!(self.len(), weights.len());
        let all: length = weights.iter().cloned().sum();
        for i in 0..self.len() {
            self[i] += weights[i] * space / all
        }
    }
}

pub enum Alignment {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
}

fn place_bounded(
    constraints: &[Linear],
    start: finite,
    bound: length,
    alignment: Alignment,
) -> Vec<Span> {
    let min: length = constraints.iter().map(|c| c.min).sum();
    if bound <= min {
        // bound is below or at the minimum size
        // -> size all elements to their minimum risking a layout overflow.
        // TODO: implement wrapping.
        return to_spans(start, constraints.iter().map(|c| c.min)).collect();
    }

    // bound > min

    let preferred: length = constraints.iter().map(|c| c.preferred_effective()).sum();
    if bound <= preferred {
        // bound is over min, but below preferred.
        // so we distribute the remaining space over min.
        // weight is preferred (delta from min to effective_preferred)
        let weights: Vec<length> = constraints.iter().map(|c| c.preferred).collect();
        let mut lengths: Vec<length> = constraints.iter().map(|c| c.min).collect();
        lengths
            .as_mut_slice()
            .distribute(bound - min, weights.as_slice());
        return to_spans(start, lengths.into_iter()).collect();
    }

    // bound > preferred

    {
        let balanced = compute_smallest_balanced_layout(constraints);
        let balanced_all: length = balanced.iter().cloned().sum();
        if bound <= balanced_all {
            // distribution weights are the balanced layout lengths minus the preferred effective.
            let mut lengths: Vec<length> = constraints
                .iter()
                .map(|c| c.preferred_effective())
                .collect();
            let weights: Vec<length> = (0..constraints.len())
                .map(|i| balanced[i] - lengths[i])
                .collect();
            lengths
                .as_mut_slice()
                .distribute(bound - balanced_all, weights.as_slice());
            return to_spans(start, lengths.into_iter()).collect();
        }
    }
    // bound > smallest balanced layout

    if let Max::Length(max_length) = constraints.iter().map(|c| c.max_effective()).max().unwrap() {
        if bound <= max_length {
            // bound is below the maximum layout possible.
            // So we need to compute a (balanced) layout with the remaining space available.
            // TODO
            return unimplemented!();
        }

        // bound is > max size
        // TODO: use alignment to place the elements.
        return unimplemented!();
    }

    // there is no maximum size, compute a balanced layout with the remaining space available.
    return unimplemented!();
}

/// Convert a starting point and a number of length's to spans.
fn to_spans(start: finite, it: impl Iterator<Item = length>) -> impl Iterator<Item = Span> {
    it.scan(start, |cur, l| {
        let c = *cur;
        *cur = *cur + l;
        Some(span(c, l))
    })
}

/// Compute the smallest possible balanced layout.
///
/// The balanced layout is a layout that sizes all elements to the preferred
/// size of the largest element while also keeping their size below their max.
fn compute_smallest_balanced_layout(constraints: &[Linear]) -> Vec<length> {
    let max_preferred = constraints
        .iter()
        .map(|c| c.preferred_effective())
        .max()
        .unwrap();
    constraints
        .iter()
        .map(|c| c.max_effective().limit(max_preferred))
        .collect()
}

/// A segment that describes a length range that can be interpolated linearly.
struct InterpolationSegment<'a> {
    /// Beginning length of all the elements.
    begin: length,
    /// Ending length of all the elements, this is where the next segment starts.
    ///
    /// The last segment may be infinite.
    end: Max,
    /// The maximums of the elements for this range only.
    ///
    /// This does include the elements that are smaller in a layout with start length,
    /// and does not include the elements that max is larger in a layout with end length.
    max_constraints: &'a Vec<Max>,
}

impl<'a> InterpolationSegment<'a> {
    pub fn layout(&self, l: length) -> Vec<length> {
        assert!(l >= self.begin);
        assert!(Max::Length(l) <= self.end);
        unimplemented!()
    }
}

struct InterpolationSegmentIterator<'a> {
    constraints: &'a [Linear],
    base: Vec<length>,
    max_sorted: Vec<(Max, usize)>,
    current_max_index: usize,
}

/*
impl<'a> Iterator for InterpolationSegmentIterator<'a> {
    type Item = InterpolationSegment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_max_index == self.max_sorted.len() {
            return None;
        }
    }
}

impl<'a> InterpolationSegmentIterator<'a> {
    pub fn from_constraints(constraints: &'a [Linear]) -> Self {
        let mut sorted_max = {
            let mut v: Vec<(Max, usize)> = constraints
                .iter()
                .enumerate()
                .map(|(i, c)| (c.max_effective(), i))
                .collect();
            v.sort();
            v
        };

        Self {
            constraints,
            sorted_max,
            current: 0,
        }
    }
}
*/

/// An interator, that returns linear interpolation steps over the contraints.
///
/// This is used to keep the layout blanced while repecting their max sizes.

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Max {
    Length(length),
    Infinite,
}

impl PartialOrd for Max {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (*self, *other) {
            (Max::Length(s), Max::Length(o)) => s.partial_cmp(&o),
            (Max::Infinite, Max::Length(_)) => Some(Ordering::Greater),
            (Max::Length(_), Max::Infinite) => Some(Ordering::Less),
            (Max::Infinite, Max::Infinite) => Some(Ordering::Equal),
        }
    }
}

impl Ord for Max {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Max {
    pub fn map<F>(&self, f: F) -> Max
    where
        F: Fn(length) -> length,
    {
        match *self {
            Max::Length(v) => Max::Length(f(v)),
            Max::Infinite => Max::Infinite,
        }
    }

    /// Limit a length to the maximum.
    pub fn limit(&self, l: length) -> length {
        match *self {
            Max::Length(s) => l.min(s),
            Max::Infinite => l,
        }
    }
}

#[cfg(test)]

mod tests {
    use crate::constraints::{Linear, Max};
    use emergent_drawing::{scalar, Canvas, DrawingCanvas, Line, Paint, PaintStyle, Rect};

    #[test]
    fn visualized_constraints() {
        let height = 256.0;

        let constraints = [
            Linear {
                min: 10.0.into(),
                preferred: 15.0.into(),
                max: Max::Infinite,
            },
            Linear {
                min: 20.0.into(),
                preferred: 15.0.into(),
                max: Max::Length(20.0.into()),
            },
        ];

        let mut canvas = DrawingCanvas::new();
        let blue = Paint::new().color(0xff0000ff).clone();
        let green = Paint::new().color(0xff00ff00).clone();
        let red = Paint::new().color(0xffff0000).clone();

        let width = 64.0;
        let mut left = 0.0;

        for constraint in &constraints {
            let min = *constraint.min;
            let preferred = *constraint.preferred_effective();

            let right = left + width;

            let min_line = Line::from(((left, height - min).into(), (right, height - min).into()));
            canvas.draw(min_line, &blue);

            let preferred_line = Line::from((
                (left, height - preferred).into(),
                (right, height - preferred).into(),
            ));
            canvas.draw(preferred_line, &green);

            if let Max::Length(max) = constraint.max_effective() {
                let max = *max;
                let max_line =
                    Line::from(((left, height - max).into(), (right, height - max).into()));
                canvas.draw(max_line, &red);
            }

            left += width;
        }

        {
            let width_box = width * constraints.len() as scalar;
            let r = Rect::from(((0, 0).into(), (width_box, height).into()));

            let black = Paint::new()
                .color(0xff808080)
                .style(PaintStyle::Stroke)
                .clone();

            canvas.draw(r, &black);
        }
        canvas.render();
    }
}
