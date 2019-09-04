//! Functions related to mapping chains.

use crate::{Chain, Identity, Ref};

struct MapChain<'s, F, Source: Sized, Target>
where
    F: Fn(&Target, &Source) -> Target,
{
    f: F,
    source: &'s dyn Chain<Source>,
    mapped: Vec<Option<Target>>,
}

impl<'s, F, Source, Target> Chain<Target> for MapChain<'s, F, Source, Target>
where
    F: Fn(&Target, &Source) -> Target,
{
    fn len(&self) -> usize {
        self.mapped.len()
    }

    fn resolve(&mut self, r: Ref<Target>) -> Option<&Target> {
        let mut target = &mut self.mapped[r.index()?];
        match target {
            Some(t) => Some(t),
            None => {
                let (parent_ref, source) = self.source.resolve(r.transmute());
                let (_, parent) = self.resolve(parent_ref.transmute());
                *target = Some((self.f)(parent, source));
                (parent_ref.transmute(), target.as_ref().unwrap())
            }
        }
    }
}

/// Create another chain by mapping from element to another.
pub fn lazy_map<'a, F: 'a, Source, Target: Identity + Clone + 'static>(
    source: &'a dyn Chain<Source>,
    f: F,
) -> impl Chain<Target> + 'a
where
    for<'r, 's> F: Fn(&'r Target, &'s Source) -> Target,
{
    assert!(!source.is_empty());
    let mut v = vec![None; source.len()];
    v[0] = Some(Target::IDENTITY);
    MapChain {
        f,
        source,
        mapped: v,
    }
}
