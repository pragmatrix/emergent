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
            Bound::Bounded(length) => place_bounded(self, start, length, Alignment::Start).1,
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LayoutMode {
    ZeroToMin,
    MinToPreferred,
    PreferredToBalanced,
    BalancedToMax,
    BalancedToInfinite,
}

fn place_bounded(
    constraints: &[Linear],
    start: finite,
    bound: length,
    _alignment: Alignment,
) -> (LayoutMode, Vec<Span>) {
    let min: length = constraints.iter().map(|c| c.min).sum();
    if bound <= min {
        // bound is below or at the minimum size
        // -> size all elements to their minimum risking a layout overflow.
        // TODO: implement wrapping.
        return (
            LayoutMode::ZeroToMin,
            to_spans(start, constraints.iter().map(|c| c.min)).collect(),
        );
    }

    // bound > min

    let preferred_length: length = constraints.iter().map(|c| c.preferred_effective()).sum();
    if bound <= preferred_length {
        // bound is over min, but below preferred.
        // so we distribute the remaining space over min.
        // weight is preferred (delta from min to effective_preferred)
        let weights: Vec<length> = constraints.iter().map(|c| c.preferred).collect();
        let mut lengths: Vec<length> = constraints.iter().map(|c| c.min).collect();
        lengths
            .as_mut_slice()
            .distribute(bound - min, weights.as_slice());
        return (
            LayoutMode::MinToPreferred,
            to_spans(start, lengths.into_iter()).collect(),
        );
    }

    // bound > preferred

    let balanced = compute_smallest_balanced_layout(constraints);
    let balanced_length: length = balanced.iter().cloned().sum();

    {
        if bound <= balanced_length {
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
                .distribute(bound - preferred_length, weights.as_slice());
            return (
                LayoutMode::PreferredToBalanced,
                to_spans(start, lengths.into_iter()).collect(),
            );
        }
    }

    // bound > smallest balanced layout

    if let Max::Length(max_length) = constraints.iter().map(|c| c.max_effective()).max().unwrap() {
        if bound <= max_length {
            // bound is below the maximum layout possible.
            // So we need to compute a (balanced) layout with the remaining space available.
            let lengths = distribute_over_smallest_balanced(&constraints, bound - balanced_length);
            return (
                LayoutMode::BalancedToMax,
                to_spans(start, lengths.into_iter()).collect(),
            );
        }

        // bound is > max size
        // TODO: use alignment to place the elements.
        unimplemented!();
    }

    let lengths = distribute_over_smallest_balanced(&constraints, bound - balanced_length);
    return (
        LayoutMode::BalancedToInfinite,
        to_spans(start, lengths.into_iter()).collect(),
    );
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
        .map(|c| c.max_effective().limit_to(max_preferred))
        .collect()
}

/// Distribute space over the smallest balanced layout.
fn distribute_over_smallest_balanced(
    constraints: &[Linear],
    mut to_distribute: length,
) -> Vec<length> {
    let lengths = constraints.len();
    // the elements that cannot get any larger.
    let mut at_max = vec![false; lengths];
    let mut resizable = lengths;

    // The current base length is the length we apply to each element.
    let mut current_base_length = constraints
        .iter()
        .map(|c| c.preferred_effective())
        .max()
        .unwrap();

    // The current layout.
    let mut layout = vec![length::default(); lengths];

    // compute the smallest balanced layout (TODO: we already computed that, recycle!).
    for (i, c) in constraints.iter().enumerate() {
        if c.max_effective() < Max::Length(current_base_length) {
            at_max[i] = true;
            resizable -= 1;
            layout[i] = c.max_effective().limit_to(current_base_length)
        } else {
            layout[i] = current_base_length;
        }
    }

    // build a list of max values that are next to consider.
    let max_limits: Vec<(length, usize)> = {
        let mut max: Vec<(length, usize)> = constraints
            .iter()
            .enumerate()
            .filter(|(i, _)| !at_max[*i])
            .filter_map(|(i, c)| c.max_effective().length().map(|l| (l, i)))
            .collect();
        max.sort();
        max
    };

    let mut current_limit_index = 0;
    while to_distribute > 0.0.into() {
        if current_limit_index == max_limits.len() {
            if resizable != 0 {
                // distribute the rest to the ones resizable which can grow infinitely.
                distribute_equally(&mut layout, &at_max, to_distribute / resizable.into());
            }
            break;
        }

        let (next_max, next_max_i) = max_limits[current_limit_index];
        debug_assert!(!at_max[next_max_i]);
        debug_assert!(next_max >= current_base_length);

        if next_max > current_base_length {
            let distribute_now =
                to_distribute.min((next_max - current_base_length) * resizable.into());
            distribute_equally(&mut layout, &at_max, distribute_now / resizable.into());
            to_distribute -= distribute_now;
            current_base_length += distribute_now;
        }

        at_max[next_max_i] = true;
        resizable -= 1;
        current_limit_index += 1;
    }

    return layout;

    fn distribute_equally(layout: &mut Vec<length>, at_max: &[bool], length: length) {
        debug_assert_eq!(layout.len(), at_max.len());
        for (i, at_max) in at_max.iter().enumerate() {
            if !at_max {
                layout[i] += length
            }
        }
    }
}

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
    pub fn limit_to(&self, l: length) -> length {
        match *self {
            Max::Length(s) => l.min(s),
            Max::Infinite => l,
        }
    }

    pub fn length(&self) -> Option<length> {
        match *self {
            Max::Length(l) => Some(l),
            Max::Infinite => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::constraints::{place_bounded, Alignment, Linear, Max};
    use crate::finite;
    use emergent_drawing::{functions::*, paint, scalar, Canvas, DrawingCanvas};

    #[test]
    fn visualized_constraints() {
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
        let blue = paint().color(0xff0000ff).clone();
        let green = paint().color(0xff00ff00).clone();
        let red = paint().color(0xffff0000).clone();
        let light_grey = paint().color(0xffd0d0d0).clone();

        let left = 512.0;

        let grey = paint().color(0xff808080).clone();
        let black = paint().clone();

        let v_spacing = 8.0;
        let box_height = 16.0;
        let constraint_marker_height = box_height / 2.0;
        let mut previous_positions: Option<Vec<finite>> = None;
        let font = font("", 12.0);

        for (layout_index, bound) in (0..=120).step_by(5).enumerate() {
            let (mode, spans) = place_bounded(
                &constraints,
                0.0.into(),
                (bound as scalar).into(),
                Alignment::SpaceBetween,
            );

            let top = layout_index as scalar * (v_spacing + box_height);
            let bottom = top + box_height;

            dbg!(&spans);
            let span = crate::spans::span(&spans).unwrap();

            // draw mode as text to the right.
            {
                let spacing = 8.0;
                let mode_str = format!("{} -> {}, {:?}", bound, *span.length(), mode);
                let pos = (left + *span.end() + spacing, bottom);
                let text = text(pos, mode_str, &font);
                canvas.draw(text, &black);
            }

            // draw the top and bottom lines

            {
                let range = (left + *span.begin(), left + *span.end());
                let top_line = line_h(top, range);
                let bottom_line = line_h(bottom, range);
                canvas.draw(top_line, &grey);
                canvas.draw(bottom_line, &grey);
            }

            let positions: Vec<finite> = crate::spans::positions(&spans).collect();

            // draw the connector lines from the previous layout.

            if let Some(previous_positions) = previous_positions {
                let previous_top = top - v_spacing;
                for (i, pos) in previous_positions.iter().enumerate() {
                    let current_position = positions[i];
                    let line = line(
                        (left + **pos, previous_top),
                        (left + *current_position, top),
                    );
                    canvas.draw(line, &light_grey);
                }
            };

            // draw the vertical separators of all spans.
            {
                for position in &positions {
                    let left = left + **position;
                    let line = line_v(left, (top, bottom));
                    canvas.draw(line, &grey);
                }
            }

            previous_positions = Some(positions);

            // draw the constraint markers
            for (i, span) in spans.iter().enumerate() {
                dbg!((top, bottom));
                let left = left + *span.begin();

                let constraint = constraints[i];
                let length = span.length();
                let constraint_marker_vrange = (top, top + constraint_marker_height);

                if length >= constraint.min {
                    let value = *constraint.min;
                    let marker = line_v(left + value, constraint_marker_vrange);
                    canvas.draw(marker, &blue);
                }

                if length >= constraint.preferred_effective() {
                    let value = *constraint.preferred_effective();
                    let marker = line_v(left + value, constraint_marker_vrange);
                    canvas.draw(marker, &green);
                }

                if let Max::Length(max) = constraint.max_effective() {
                    if length >= max {
                        let value = *max;
                        let marker = line_v(left + value, constraint_marker_vrange);
                        canvas.draw(marker, &red);
                    }
                }
            }
        }

        canvas.render();
    }
}
