//! A declarative DSL to create user interface views.

use crate::{Context, Direction, View};
use emergent_drawing::{DrawingFastBounds, Point, Transformed, Vector};
use emergent_presentation::Scoped;

// TODO: View<Msg> and Context<Msg> could be made into a generic V and C?
// TODO: combine Item and Data somehow, or can we use a trait to make them both mappable?

pub struct Item<'a, I> {
    item: &'a I,
}

impl<'a, I> Item<'a, I> {
    pub fn new(item: &'a I) -> Self {
        Self { item }
    }
}

impl<'a, I> Item<'a, I> {
    pub fn map<Msg>(
        self,
        map_f: impl Fn(&mut Context<Msg>, &I) -> View<Msg> + 'a,
    ) -> ItemMap<'a, Msg, I> {
        ItemMap {
            item: self,
            map_f: Box::new(map_f),
        }
    }
}

pub struct ItemMap<'a, Msg, I> {
    item: Item<'a, I>,
    map_f: Box<dyn Fn(&mut Context<Msg>, &I) -> View<Msg> + 'a>,
}

pub struct Data<'a, E> {
    data: &'a [E],
}

impl<'a, E> Data<'a, E> {
    pub fn new(data: &'a [E]) -> Self {
        Self { data }
    }
}

impl<'a, E> Data<'a, E> {
    pub fn map<Msg>(
        self,
        map_f: impl Fn(&mut Context<Msg>, &E) -> View<Msg> + 'a,
    ) -> DataMap<'a, Msg, E> {
        DataMap {
            data: self,
            map_f: Box::new(map_f),
        }
    }
}

pub struct DataMap<'a, Msg, E> {
    data: Data<'a, E>,
    map_f: Box<dyn Fn(&mut Context<Msg>, &E) -> View<Msg> + 'a>,
}

pub trait IndexMappable<Msg> {
    fn len(&self) -> usize;
    fn map_i(&self, context: &mut Context<Msg>, i: usize) -> View<Msg>;

    fn extend<'a>(&'a self, other: &'a dyn IndexMappable<Msg>) -> ExtendedIndexMappable<'a, Msg>
    where
        Self: Sized,
    {
        ExtendedIndexMappable {
            left: (self as &dyn IndexMappable<Msg>),
            right: other,
        }
    }
}

pub struct ExtendedIndexMappable<'a, Msg> {
    left: &'a dyn IndexMappable<Msg>,
    right: &'a dyn IndexMappable<Msg>,
}

impl<'a, Msg> IndexMappable<Msg> for ExtendedIndexMappable<'a, Msg> {
    fn len(&self) -> usize {
        self.left.len() + self.right.len()
    }

    fn map_i(&self, context: &mut Context<Msg>, i: usize) -> View<Msg> {
        let ll = self.left.len();
        if i < ll {
            self.left.map_i(context, i)
        } else {
            self.right.map_i(context, i - ll)
        }
    }
}

impl<'a, Msg, I> IndexMappable<Msg> for ItemMap<'a, Msg, I> {
    fn len(&self) -> usize {
        1
    }

    fn map_i(&self, context: &mut Context<Msg>, i: usize) -> View<Msg> {
        debug_assert_eq!(i, 0);
        let map_f = &self.map_f;
        let item = &self.item.item;
        (map_f)(context, item)
    }
}

impl<'a, Msg, E> IndexMappable<Msg> for DataMap<'a, Msg, E> {
    fn len(&self) -> usize {
        self.data.data.len()
    }

    fn map_i(&self, context: &mut Context<Msg>, i: usize) -> View<Msg> {
        let map_f = &self.map_f;
        let data = &self.data.data[i];

        (map_f)(context, data)
    }
}

pub trait Reducible<Msg> {
    fn reduce(
        self,
        context: &mut Context<Msg>,
        reducer: impl ViewReducer<Msg> + 'static,
    ) -> View<Msg>;
}

impl<Msg, T> Reducible<Msg> for T
where
    T: IndexMappable<Msg>,
{
    // TODO: this is not the whole story here, how can we reduce incrementally?
    fn reduce(
        self,
        context: &mut Context<Msg>,
        reducer: impl ViewReducer<Msg> + 'static,
    ) -> View<Msg> {
        let len = self.len();

        let views: Vec<View<Msg>> = (0..len)
            .map(|i| context.nested(i, |c| self.map_i(c, i)))
            .collect();
        let reduced = reducer.reduce(context, views);
        reduced
    }
}

// TODO: I think this trait provides the wrong functionality, we need to support to pull
//       view elements lazily (probably by index?).
//       If so, this interface may be replacible by IndexMappable?
pub trait ViewReducer<Msg> {
    fn reduce(&self, context: &mut Context<Msg>, views: Vec<View<Msg>>) -> View<Msg>;
}

/// TODO: find a better type for the identity reducer.

impl<Msg> ViewReducer<Msg> for () {
    fn reduce(&self, _context: &mut Context<Msg>, views: Vec<View<Msg>>) -> View<Msg> {
        let views = views
            .into_iter()
            .enumerate()
            .map(|(i, view)| view.scoped(i));
        View::new_combined(views)
    }
}

impl<Msg> ViewReducer<Msg> for Direction {
    fn reduce(&self, context: &mut Context<Msg>, views: Vec<View<Msg>>) -> View<Msg> {
        // TODO: recycle container?
        // TODO: only display elements that are visible.
        // TODO: Use a generic layout manager here.
        let mut p = Point::default();
        let direction = self.to_vector();

        let views = views.into_iter().enumerate().map(|(i, view)| {
            let view = view.scoped(i);
            let drawing_bounds = view.fast_bounds(context);
            if let Some(bounds) = drawing_bounds.as_bounds() {
                let align = -bounds.point.to_vector();
                let nested = view.transformed((p + align).to_vector());
                p += Vector::from(bounds.extent) * direction;
                nested
            } else {
                View::new()
            }
        });

        View::new_combined(views)
    }
}
