//! A DSL to create user interface views based on data.

use crate::{Direction, ScopedView, View, ViewBuilder};
use emergent_drawing::{DrawingBounds, Point, Transformed, Vector};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem;
use std::ops::Range;

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
    pub fn map<F, Msg>(self, map_f: F) -> ItemMap<'a, F, Msg, I>
    where
        F: Fn(ViewBuilder<Msg>, &I) -> View<Msg>,
    {
        ItemMap {
            item: self,
            map_f,
            pd: PhantomData,
        }
    }
}

pub struct ItemMap<'a, F, Msg, I> {
    item: Item<'a, I>,
    map_f: F,
    pd: PhantomData<*const Msg>,
}

//
// Data (TODO: rename this to projection?)
//

pub struct Data<'a, E> {
    data: &'a [E],
}

impl<'a, E> Data<'a, E> {
    pub fn new(data: &'a [E]) -> Self {
        Self { data }
    }
}

impl<'a, E> IndexAccessible<E> for Data<'a, E> {
    fn as_slice(&self) -> &[E] {
        self.data
    }
}

//
// AsData / TODO: rename to Project / as_projection()?
//

pub trait AsData<'a, E> {
    fn as_data(&'a self) -> Data<'a, E>;
}

impl<'a, E> AsData<'a, E> for Vec<E> {
    fn as_data(&'a self) -> Data<'a, E> {
        Data::new(&self)
    }
}

impl<'a, E> AsData<'a, E> for &'a [E] {
    fn as_data(&'a self) -> Data<'a, E> {
        Data::new(self)
    }
}

//
// IndexAccessible
//

pub trait IndexAccessible<E> {
    fn as_slice(&self) -> &[E];

    fn map_view<F, Msg>(self, map_f: F) -> DataMap<Self, F, Msg, E>
    where
        F: Fn(ViewBuilder<Msg>, &E) -> View<Msg>,
        Self: Sized,
    {
        DataMap {
            data: self,
            map_f,
            pd: PhantomData,
        }
    }

    fn partition<F>(self, partition_f: F) -> Partition<Self, F, E>
    where
        E: Clone,
        F: Fn(&E) -> bool,
        Self: Sized,
    {
        let (a, b): (Vec<_>, Vec<_>) = self
            .as_slice()
            .iter()
            .cloned()
            .partition(|e| partition_f(e));

        Partition {
            data: self,
            partition_f,
            result: (a, b),
        }
    }

    fn order_by<F>(self, order_f: F) -> OrderBy<Self, F, E>
    where
        E: Clone,
        F: Fn(&E, &E) -> Ordering,
        Self: Sized,
    {
        let mut projection: Vec<_> = self.as_slice().iter().cloned().collect();
        projection.sort_by(&order_f);

        OrderBy {
            data: self,
            projection,
            order_f,
        }
    }
}

//
// OrderBy
//

pub struct OrderBy<D, F, E> {
    data: D,
    projection: Vec<E>,
    order_f: F,
}

// TODO: may use AsRef<[E]> for that.

impl<'b, D, E, F> IndexAccessible<E> for OrderBy<D, F, E>
where
    D: IndexAccessible<E>,
{
    fn as_slice(&self) -> &[E] {
        &self.projection
    }
}

//
// Partition
//

pub struct Partition<D, F, E> {
    data: D,
    partition_f: F,
    pub result: (Vec<E>, Vec<E>),
}

//
// DataMap
//

pub struct DataMap<D, F, Msg, E> {
    data: D,
    map_f: F,
    pd: PhantomData<(*const Msg, *const E)>,
}

pub trait IndexMappable<Msg> {
    fn len(&self) -> usize;
    fn map_i(&self, builder: ViewBuilder<Msg>, i: usize) -> View<Msg>;

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

    fn map_i(&self, builder: ViewBuilder<Msg>, i: usize) -> View<Msg> {
        let ll = self.left.len();
        if i < ll {
            self.left.map_i(builder, i)
        } else {
            self.right.map_i(builder, i - ll)
        }
    }
}

impl<'a, F, Msg, I> IndexMappable<Msg> for ItemMap<'a, F, Msg, I>
where
    F: Fn(ViewBuilder<Msg>, &I) -> View<Msg>,
{
    fn len(&self) -> usize {
        1
    }

    fn map_i(&self, builder: ViewBuilder<Msg>, i: usize) -> View<Msg> {
        debug_assert_eq!(i, 0);
        let map_f = &self.map_f;
        let item = &self.item.item;
        (map_f)(builder, item)
    }
}

impl<D, F, Msg, E> IndexMappable<Msg> for DataMap<D, F, Msg, E>
where
    D: IndexAccessible<E>,
    F: Fn(ViewBuilder<Msg>, &E) -> View<Msg>,
{
    fn len(&self) -> usize {
        self.data.as_slice().len()
    }

    fn map_i(&self, builder: ViewBuilder<Msg>, i: usize) -> View<Msg> {
        let map_f = &self.map_f;
        let data = &self.data.as_slice()[i];

        (map_f)(builder, data)
    }
}

pub trait Reducible<Msg> {
    fn reduce(
        self,
        builder: ViewBuilder<Msg>,
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
        mut builder: ViewBuilder<Msg>,
        reducer: impl ViewReducer<Msg> + 'static,
    ) -> View<Msg> {
        let reduced = reducer.reduce(builder, 0..self.len(), |builder, i| {
            builder.scoped(i, |b| self.map_i(b, i))
        });
        reduced
    }
}

pub trait ViewReducer<Msg> {
    fn reduce(
        &self,
        container: ViewBuilder<Msg>,
        views: Range<usize>,
        builder: impl Fn(&mut ViewBuilder<Msg>, usize) -> ScopedView<Msg>,
    ) -> View<Msg>;

    // TODO: this is temporary until we might find a better signature for reduce, that allows both, a set of
    // pre-generated ScopedViews and lazy resolvement.
    fn reduce_immediate(&self, container: ViewBuilder<Msg>, v: Vec<ScopedView<Msg>>) -> View<Msg> {
        let v: Vec<_> = v.into_iter().map(Some).collect();
        let len = v.len();
        // TODO: avoid the RefCell here?
        let v = RefCell::new(v);
        self.reduce(container, 0..len, |_, i| {
            let v = mem::replace(v.borrow_mut().get_mut(i).unwrap(), None).unwrap();
            v
        })
    }
}

/// TODO: find a better type for the identity reducer.

impl<Msg> ViewReducer<Msg> for () {
    fn reduce(
        &self,
        mut container: ViewBuilder<Msg>,
        views: Range<usize>,
        builder: impl Fn(&mut ViewBuilder<Msg>, usize) -> ScopedView<Msg>,
    ) -> View<Msg> {
        let nested: Vec<_> = views
            .into_iter()
            .map(|i| builder(&mut container, i))
            .collect();
        container.combined(nested)
    }
}

impl<Msg> ViewReducer<Msg> for Direction {
    fn reduce(
        &self,
        mut container: ViewBuilder<Msg>,
        views: Range<usize>,
        builder: impl Fn(&mut ViewBuilder<Msg>, usize) -> ScopedView<Msg>,
    ) -> View<Msg> {
        let mut p = Point::default();
        let direction = self.to_vector();

        // TODO: recycle container?
        // TODO: only display elements that are visible.
        // TODO: Use a generic layout manager here.

        let views: Vec<_> = views
            .into_iter()
            .filter_map(|i| {
                let nested = builder(&mut container, i).presentation_scoped(i);
                let drawing_bounds = nested.fast_bounds(&container);
                if let Some(bounds) = drawing_bounds.as_bounds() {
                    // * direction.abs() makes sure that only alignment in the layout direction is considered, and
                    // the alignment of the cross axis is left as it is.
                    let align = -bounds.point.to_vector() * direction.abs();
                    let nested = nested.transformed((p + align).to_vector());
                    p += Vector::from(bounds.extent) * direction;
                    Some(nested)
                } else {
                    None
                }
            })
            .collect();

        container.combined(views)
    }
}

pub trait SimpleLayout {
    fn layout(&self, bounds: impl Iterator<Item = DrawingBounds>) -> Vec<Vector>;

    fn layout_bounds(&self, bounds: impl Iterator<Item = DrawingBounds>) -> Vec<DrawingBounds> {
        let bounds: Vec<_> = bounds.collect();
        let vecs = self.layout(bounds.iter().cloned());
        vecs.into_iter()
            .enumerate()
            .map(|(i, v)| bounds[i].transformed(v))
            .collect()
    }
}

impl SimpleLayout for Direction {
    fn layout(&self, bounds: impl Iterator<Item = DrawingBounds>) -> Vec<Vector> {
        let mut p = Point::default();
        let direction = self.to_vector();

        bounds
            .map(|drawing_bounds| {
                if let Some(bounds) = drawing_bounds.as_bounds() {
                    let align = -bounds.point.to_vector();
                    let nested = (p + align).to_vector();
                    p += Vector::from(bounds.extent) * direction;
                    nested
                } else {
                    Vector::default()
                }
            })
            .collect()
    }
}
