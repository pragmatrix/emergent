use crate::{ChangeSet, IndexedSource, IndexedTarget};

pub struct LazyMap<E, R> {
    generator: Box<dyn Fn(&E) -> R>,
    elements: Vec<E>,
    generated: Vec<Option<R>>,
}

impl<E, R> LazyMap<E, R> {
    pub fn new(generator: impl Fn(&E) -> R + 'static) -> Self {
        LazyMap {
            generator: Box::new(generator),
            elements: Vec::new(),
            generated: Vec::new(),
        }
    }

    fn iter_at(&mut self, start: usize) -> impl Iterator<Item = &R> {
        let ref elements = self.elements;
        let ref generator = self.generator;
        let ref mut generated = self.generated;

        generated[start..]
            .iter_mut()
            .enumerate()
            .map(move |(i, r)| match r {
                Some(r) => r,
                None => {
                    *r = Some((generator)(&elements[i + start]));
                    r.as_ref().unwrap()
                }
            })
    }
}

impl<E, R> IndexedSource<R> for LazyMap<E, R> {
    fn len(&self) -> usize {
        self.generated.len()
    }

    fn iter_at(&mut self, start: usize) -> Box<dyn Iterator<Item = &R> + '_> {
        Box::new(self.iter_at(start))
    }
}

impl<E, R> IndexedTarget<E> for LazyMap<E, R> {
    fn apply(&mut self, elements: &[E]) -> ChangeSet<E>
    where
        E: Clone,
        E: PartialEq,
    {
        let cs = self.elements.apply(elements);
        let len = cs.len_applied();
        let ref mut generated = self.generated;
        generated.resize_with(len, || None);
        cs.updated_indices().for_each(|i| generated[i] = None);
        cs
    }
}

#[cfg(test)]
mod tests {
    use crate::{IndexedTarget, LazyMap};

    #[test]
    fn test_random_access_laziness() {
        let mut map: LazyMap<i32, i32> = LazyMap::new(|a| a * a);
        map.apply(&[2, 3, 4]);
        let v = *map.iter_at(1).nth(0).unwrap();
        assert_eq!(v, 9);
        assert_eq!(map.generated[0], None);
        assert_eq!(map.generated[2], None);
    }
}
