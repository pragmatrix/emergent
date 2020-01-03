//! Experimental referential presentations.

use emergent_drawing::{BlendMode, Clip, Paint, Point, Rect, Shape, Transform};
use std::marker::PhantomData;

#[derive(Debug)]
struct Table<T>(Vec<T>);

impl<T> Table<T> {
    fn new() -> Table<T> {
        Table(Vec::new())
    }
}

#[derive(Copy, Clone, Debug)]
struct Ref<T>(usize, PhantomData<T>);

impl<T> Ref<T> {
    fn new(i: usize) -> Ref<T> {
        Ref(i, PhantomData)
    }
}

impl<T> Table<T> {
    pub fn append(&mut self, t: T) -> Ref<T> {
        let r = Ref::new(self.0.len());
        self.0.push(t);
        r
    }
}

struct Transformation {
    pub left: Option<Ref<Transformation>>,
    pub right: Ref<Transform>,
}

struct Clipping {
    pub prev: Option<Ref<Clipping>>,
    pub transformation: Option<Ref<Transformation>>,
    pub clip: Ref<Clip>,
}

enum Draw {
    Shape(
        Ref<Transformation>,
        Option<Ref<Clipping>>,
        Ref<Paint>,
        Ref<Shape>,
    ),
    Fill(Ref<Clipping>, BlendMode),
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Table::new()
    }
}

#[derive(Default)]
struct Presentation {
    pub paints: Table<Paint>,
    pub clips: Table<Clip>,
    pub transforms: Table<Transform>,
    pub shapes: Table<Shape>,

    pub transformations: Table<Transformation>,
    pub clippings: Table<Clipping>,

    pub draws: Table<Draw>,
}

impl Presentation {
    pub fn new_transformation(&mut self, transform: Transform) -> Ref<Transformation> {
        let transform = self.transforms.append(transform);
        self.transformations.append(Transformation {
            left: None,
            right: transform,
        })
    }
}

impl Presentation {
    pub fn draw(
        &mut self,
        transformation: impl IntoRef<Presentation, Transformation>,
        clipping: impl IntoRefOpt<Presentation, Clipping>,
        paint: impl IntoRef<Table<Paint>, Paint>,
        shape: impl IntoRef<Table<Shape>, Shape>,
    ) -> Ref<Draw> {
        let transformation = self.resolve(transformation);
        let clipping = clipping.into_ref_opt(self);
        let paint = self.paints.resolve(paint);
        let shape = self.shapes.resolve(shape);
        self.draws
            .append(Draw::Shape(transformation, clipping, paint, shape))
    }
}

trait IntoRefOpt<Resolver, T> {
    fn into_ref_opt(self, resolver: &mut Resolver) -> Option<Ref<T>>;
}

impl<Resolver, Target, T> IntoRefOpt<Resolver, Target> for Option<T>
where
    T: IntoRef<Resolver, Target>,
{
    fn into_ref_opt(self, resolver: &mut Resolver) -> Option<Ref<Target>> {
        match self {
            Some(v) => Some(v.into_ref(resolver)),
            None => None,
        }
    }
}

trait IntoRef<Resolver, T> {
    fn into_ref(self, resolver: &mut Resolver) -> Ref<T>;
}

impl IntoRef<Presentation, Transformation> for Transform {
    fn into_ref(self, resolver: &mut Presentation) -> Ref<Transformation> {
        let transform = resolver.transforms.append(self);
        resolver.transformations.append(Transformation {
            left: None,
            right: transform,
        })
    }
}

impl IntoRef<Presentation, Clipping> for Clip {
    fn into_ref(self, resolver: &mut Presentation) -> Ref<Clipping> {
        let clip = resolver.clips.append(self);
        resolver.clippings.append(Clipping {
            prev: None,
            transformation: None,
            clip,
        })
    }
}

/*

trait IntoRef<T> {
    fn into_ref(self, table: &mut Table<T>) -> Ref<T>;
}

impl<T> IntoRef<T> for Ref<T> {
    fn into_ref(self, _table: &mut Table<T>) -> Ref<T> {
        self
    }
}

impl<T> IntoRef<T> for T {
    fn into_ref(self, table: &mut Table<T>) -> Ref<T> {
        table.append(self)
    }
}

*/

impl<T> IntoRef<Table<T>, T> for T {
    fn into_ref(self, resolver: &mut Table<T>) -> Ref<T> {
        resolver.append(self)
    }
}

impl Presentation {
    fn resolve<T>(&mut self, ir: impl IntoRef<Presentation, T>) -> Ref<T> {
        ir.into_ref(self)
    }
}

impl<T> Table<T> {
    fn resolve(&mut self, rr: impl IntoRef<Table<T>, T>) -> Ref<T> {
        rr.into_ref(self)
    }
}

#[test]
pub fn test() {
    let mut p = Presentation::default();
    p.draw(
        Transform::Identity,
        Option::<Clip>::None,
        Paint::new(),
        Shape::Point(Point::new(1.0, 1.0)),
    );
}
